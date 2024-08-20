use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

use crate::raft::network::*;
use crate::raft::rpc::AppendEntries;
use crate::raft::state::RaftState;

use super::rpc::RequestVote;
use super::state::LogEntry;

#[derive(Debug, Clone)]
pub enum NodeRole {
    Follower,
    Candidate,
    Leader,
}

pub struct RaftNode {
    id: u64,
    state: Arc<Mutex<RaftState>>,
    election_timeout: Duration,
    role: Arc<Mutex<NodeRole>>,
    peers: Vec<String>,
}

impl RaftNode {
    pub fn new(id: u64, peers: Vec<String>) -> Self {
        Self {
            id,
            state: Arc::new(Mutex::new(RaftState::new())),
            election_timeout: Duration::from_millis(150 + rand::random::<u64>() % 150),
            role: Arc::new(Mutex::new(NodeRole::Follower)),
            peers,
        }
    }

    pub async fn run(&self) {
        let mut interval = time::interval(self.election_timeout);
        loop {
            interval.tick().await;

            let role = self.role.lock().unwrap().clone();
            match role {
                NodeRole::Leader => {
                    self.send_heartbeats().await;
                }
                NodeRole::Follower => {
                    self.check_election_timeout().await;
                }
                NodeRole::Candidate => {
                    self.start_election().await;
                }
            }
        }
    }

    async fn send_heartbeats(&self) {
        let state = self.state.lock().unwrap();
        let term = state.current_term;
        let leader_id = self.id;
        let prev_log_index = state.last_log_index();
        let prev_log_term = state.last_log_term();
        drop(state);

        for peer in &self.peers {
            let peer_address = peer.clone();
            let entries: Vec<LogEntry> = vec![];

            let append_entries = AppendEntries {
                term,
                leader_id,
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit: self.state.lock().unwrap().commit_index,
            };

            // let peer_id = *peer;
            let state = self.state.clone();
            tokio::spawn(async move {
                let response = send_append_entries(&peer_address, append_entries).await;
                let mut state = state.lock().unwrap();
                if response.term > term {
                    state.current_term = response.term;
                }
            });
        }
    }

    async fn start_election(&self) {
        {
            let mut state = self.state.lock().unwrap();
            state.current_term += 1;
            state.voted_for = Some(self.id);
        }

        let term;
        let last_log_term;
        let last_log_index;

        {
            let state = self.state.lock().unwrap();
            term = state.current_term;
            last_log_term = state.last_log_term();
            last_log_index = state.last_log_index();
        }

        let (tx, mut rx) = mpsc::channel(self.peers.len());

        for (peer, peer_address) in self.peers.iter().enumerate() {
            let tx = tx.clone();
            let request_vote = RequestVote {
                term,
                candidate_id: self.id,
                last_log_index,
                last_log_term,
            };
            
            let peer_address = peer_address.clone();
            tokio::spawn(async move {
                let response = send_request_vote(&peer_address, request_vote).await;
                // if let Ok(response) = response {
                    tx.send(response).await.unwrap();
                
            });
        }

        let mut votes = 1;
        while let Some(response) = rx.recv().await {
            if response.vote_granted {
                votes += 1;
                if votes > self.peers.len() / 2 {
                    let mut role = self.role.lock().unwrap();
                    *role = NodeRole::Leader;
                    break;
                }
            }
        }
    }

    async fn check_election_timeout(&self) {
        let mut role = self.role.lock().unwrap();
        *role = NodeRole::Candidate;
    }

    pub async fn start_server(&self, address: &str) {
        let listener = TcpListener::bind(address).await.unwrap();
        println!("Node {} listening on {}", self.id, address);

        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let state = Arc::clone(&self.state);
            tokio::spawn(async move {
                let mut buffer = [0u8; 1024];
                let n = socket.read(&mut buffer).await.unwrap();
                let request_str = String::from_utf8_lossy(&buffer[..n]);

                let response_data = {
                    let mut state = state.lock().unwrap();

                    if let Ok(request) = serde_json::from_str::<RequestVote>(&request_str) {
                        let response = state.handle_vote_request(request);
                        serde_json::to_vec(&response).unwrap()
                    } else if let Ok(request) = serde_json::from_str::<AppendEntries>(&request_str)
                    {
                        let response = state.handle_append_entries(request);
                        serde_json::to_vec(&response).unwrap()
                    } else {
                        eprintln!("Unknown request received: {}", request_str);
                        return;
                    }
                };

                socket.write_all(&response_data).await.unwrap();
            });
        }
    }
}
