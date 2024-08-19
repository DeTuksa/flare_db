use std::{fs, path::PathBuf, sync::{Arc, Mutex}};

use super::{append_only_log::AppendOnlyLog, in_memory_db::InMemoryDB, persistent_db::PersistentDB};

// pub enum StorageMode {
//     InMemory,
//     Persistent
// }

pub struct Storage {
    in_memory_db: Arc<InMemoryDB>,
    persistent_db: Arc<PersistentDB>,
    append_log: Arc<Mutex<AppendOnlyLog>>,
    operation_count: usize,
    compaction_threshold: usize
}

impl Storage {
    pub fn new(project_name: &str) -> Self {

        let db_path = generate_db_path(project_name);
        let log_path = db_path.join("append_only_log.txt");

        Storage {
            in_memory_db: Arc::new(InMemoryDB::new()),
            persistent_db: Arc::new(PersistentDB::new(db_path.to_str().unwrap())),
            append_log: Arc::new(Mutex::new(AppendOnlyLog::new(&log_path).expect("Failed to create append-only log"))),
            operation_count: 0,
            compaction_threshold: 5
        }
    }

    pub fn set_in_memory(& mut self, key: String, value: String) -> bool {

        if self.append_log.lock().unwrap().append(&format!("SET {} {}", key, value)).is_err() {
            eprintln!("Failed to log SET command");
        }

        let result = self.in_memory_db.set(key, value);

        self.operation_count += 1;
        if self.operation_count  >= self.compaction_threshold {
            self.compact_log();
        }

        return  result
    }

    pub fn get_in_memory(&self, key: &str) -> Option<String> {
        self.in_memory_db.get(key)
    }

    pub fn delete_in_memory(&mut self, key: &str) -> bool {
        let result = self.in_memory_db.delete(key);

        self.operation_count += 1;
        if self.operation_count >= self.compaction_threshold {
            self.compact_log();
        }

        result
    }

    pub fn set_persistent(&self, key: String, value: String) -> bool {
        self.persistent_db.set(key, value)
    }

    pub fn get_persistent(&self, key: &str) -> Option<String> {
        self.persistent_db.get(key)
    }

    pub fn delete_persistent(&self, key: &str) -> bool {
        self.persistent_db.delete(key)
    }

    pub fn replay_log(&mut self) {
        let log = self.append_log.lock().unwrap();
        if let Ok(commands) = log.replay() {
            for command in commands  {
                let parts: Vec<&str> = command.split_whitespace().collect();

                if parts.is_empty() {
                    continue;
                }

                match parts[0] {
                    "SET" if parts.len() == 3 => {
                        let key = parts[1].to_string();
                        let value = parts[2].to_string();
                        self.in_memory_db.set(key, value);
                    }
                    "DELETE" if parts.len() == 2 => {
                        let key = parts[1];
                        self.in_memory_db.delete(key);
                    }
                    _ => eprintln!("Unrecognized command in log: {}", command)
                }
            }
        }
    }

    fn compact_log(&mut self) {
        eprintln!("Compacting log...");
        let state = self.in_memory_db.clone_state();
        let append_clone = self.append_log.clone();
        if append_clone.lock().unwrap().compact(&state).is_err() {
            eprintln!("Failed to compact log");
        }
        self.operation_count = 0;
    }
}

fn generate_db_path(project_name: &str) -> PathBuf {
    let mut base_dir = std::env::current_dir().unwrap();
    base_dir.push("databases");
    base_dir.push(project_name);
    base_dir.push("db");

    // Create the directory if it doesn't exist
    fs::create_dir_all(&base_dir).unwrap();

    base_dir
}