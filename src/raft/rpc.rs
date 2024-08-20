use serde::{Deserialize, Serialize};
use super::state::{LogEntry, RaftState};

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct RequestVote {
    pub term: u64,
    pub candidate_id: u64,
    pub last_log_index: u64,
    pub last_log_term: u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResponse {
    pub term: u64,
    pub vote_granted: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntries {
    pub term: u64,
    pub leader_id: u64,
    pub prev_log_index: u64,
    pub prev_log_term: u64,
    pub entries: Vec<LogEntry>,
    pub leader_commit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesResponse {
    pub term: u64,
    pub success: bool,
}

impl RaftState {
    pub fn handle_vote_request(
        &mut self,
        req: RequestVote
    ) -> VoteResponse {
        let mut vote_granted = false;

        if req.term > self.current_term {
            self.current_term = req.term;
            self.voted_for = None;
        }

        if self.voted_for.is_none() || self.voted_for == Some(req.candidate_id) {
            if req.last_log_term > self.last_log_term() ||
            (req.last_log_term == self.last_log_term() && req.last_log_index >= self.last_log_index()) {
                self.voted_for = Some(req.candidate_id);
                vote_granted = true;
            }
        }

        VoteResponse {
            term: self.current_term,
            vote_granted
        }
    }
}