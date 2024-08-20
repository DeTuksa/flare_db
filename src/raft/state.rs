use serde::{Deserialize, Serialize};

use crate::raft::rpc::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: u64,
    pub command: String
}

#[derive(Debug, Clone)]
pub struct RaftState {
    pub current_term: u64,
    pub voted_for: Option<u64>,
    pub log: Vec<LogEntry>,
    pub commit_index: u64,
    pub last_applied: u64
}

impl RaftState {
    pub fn new() -> Self {
        Self {
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0
        }
    }

    pub fn append_entry(&mut self, entry: LogEntry) {
        self.log.push(entry)
    }

    pub fn last_log_index(&self) -> u64 {
        self.log.len() as u64
    }

    pub fn last_log_term(&self) -> u64 {
        self.log.last().map_or(0, |entry| entry.term)
    }

    pub fn handle_append_entries(
        &mut self,
        req: AppendEntries
    ) -> AppendEntriesResponse {
        if req.term < self.current_term {
            return AppendEntriesResponse {
                term: self.current_term,
                success: false
            };
        }

        self.current_term = req.term;

        if req.prev_log_index > 0 {
            if let Some(entry) = self.log.get((req.prev_log_index - 1) as usize) {
                if entry.term != req.prev_log_term {
                    return  AppendEntriesResponse {
                        term: self.current_term,
                        success: false
                    };
                }
            } else {
                return AppendEntriesResponse {
                    term: self.current_term,
                    success: false
                };
            }
        }

        let mut index = req.prev_log_index + 1;
        for new_entry in req.entries {
            if let Some(existing_entry) = self.log.get_mut((index - 1) as usize) {
                if existing_entry .term != new_entry.term {
                    *existing_entry = new_entry;
                }
            } else {
                self.log.push(new_entry);
            }
            index += 1;
        }

        if req.leader_commit > self.commit_index {
            self.commit_index = req.leader_commit.min(index - 1);
        }

        AppendEntriesResponse {
            term: self.current_term,
            success: true
        }
    }
}