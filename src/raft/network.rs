use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::raft::rpc::*;

pub async fn send_request_vote(peer_address: &str, request: RequestVote) -> VoteResponse {
    let mut stream = TcpStream::connect(peer_address).await.unwrap();
    let request_data = serde_json::to_vec(&request).unwrap();
    stream.write_all(&request_data).await.unwrap();

    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await.unwrap();
    let response: VoteResponse = serde_json::from_slice(&buffer[..n]).unwrap();

    response
}

pub async fn send_append_entries(peer_address: &str, request: AppendEntries) -> AppendEntriesResponse {
    let mut stream = TcpStream::connect(peer_address).await.unwrap();
    let request_data = serde_json::to_vec(&request).unwrap();
    stream.write_all(&request_data).await.unwrap();

    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await.unwrap();
    let response: AppendEntriesResponse = serde_json::from_slice(&buffer[..n]).unwrap();

    response
}