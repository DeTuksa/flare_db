
#[derive(Debug, Clone)]
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
}