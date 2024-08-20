use std::{fs::{self, File}, io::{BufReader, BufWriter, Result}, path::PathBuf, sync::{Arc, Mutex}};

use super::{append_only_log::AppendOnlyLog, in_memory_db::InMemoryDB, persistent_db::PersistentDB};

// pub enum StorageMode {
//     InMemory,
//     Persistent
// }

pub struct Storage {
    in_memory_db: Arc<Mutex<InMemoryDB>>,
    persistent_db: Arc<PersistentDB>,
    append_log: Arc<Mutex<AppendOnlyLog>>,
    operation_count: usize,
    compaction_threshold: usize,
    snapshot_path: PathBuf
}

impl Storage {
    pub fn new(project_name: &str) -> Self {

        let db_path = generate_db_path(project_name);
        let log_path = db_path.join("append_only_log.txt");
        let snapshot_path = db_path.join("snapshot.dat");

        Storage {
            in_memory_db: Arc::new(Mutex::new(InMemoryDB::new())),
            persistent_db: Arc::new(PersistentDB::new(db_path.to_str().unwrap())),
            append_log: Arc::new(Mutex::new(AppendOnlyLog::new(&log_path).expect("Failed to create append-only log"))),
            operation_count: 0,
            compaction_threshold: 5,
            snapshot_path
        }
    }

    pub fn set_in_memory(&mut self, key: String, value: String) -> bool {

        if self.append_log.lock().unwrap().append(&format!("SET {} {}", key, value)).is_err() {
            eprintln!("Failed to log SET command");
        }

        let result = {
            let db = self.in_memory_db.lock().unwrap();
            db.set(key, value)
        };

        self.operation_count += 1;
        if self.operation_count  >= self.compaction_threshold {
            self.compact_log();
            self.create_snapshot();
        }

        return  result
    }

    pub fn get_in_memory(&self, key: &str) -> Option<String> {
        let db = self.in_memory_db.lock().unwrap();
        db.get(key)
    }

    pub fn delete_in_memory(&mut self, key: &str) -> bool {
        let result = {
            let db = self.in_memory_db.lock().unwrap();
            db.delete(key)
        };

        self.operation_count += 1;
        if self.operation_count >= self.compaction_threshold {
            self.compact_log();
            self.create_snapshot();
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
            let db = self.in_memory_db.lock().unwrap();
            for command in commands  {
                let parts: Vec<&str> = command.split_whitespace().collect();

                if parts.is_empty() {
                    continue;
                }

                match parts[0] {
                    "SET" if parts.len() == 3 => {
                        let key = parts[1].to_string();
                        let value = parts[2].to_string();
                        db.set(key, value);
                    }
                    "DELETE" if parts.len() == 2 => {
                        let key = parts[1];
                        db.delete(key);
                    }
                    _ => eprintln!("Unrecognized command in log: {}", command)
                }
            }
        }
    }

    fn compact_log(&mut self) {
        eprintln!("Compacting log...");
        let state;
        {
            let db = self.in_memory_db.lock().unwrap();
            state = db.clone_state();
        }
        let append_clone = self.append_log.clone();
        if append_clone.lock().unwrap().compact(&state).is_err() {
            eprintln!("Failed to compact log");
        }
        self.operation_count = 0;
    }

    fn create_snapshot(&mut self) {
        eprintln!("Creating snapshot....");
        let db = self.in_memory_db.lock().unwrap();
        // let db = self.in_memory_db.clone();
        let snapshot_data = &*&db;
        
        if let Ok(_) = self.save_snapshot(snapshot_data) {
            eprintln!("Snapshot created successfully");
            let mut log = self.append_log.lock().unwrap();
            if log.clear().is_err() {
                eprintln!("Failed to clear log after snapshot");
            } else {
                self.operation_count = 0;
            }
        } else {
            eprintln!("Failed to save snapshot");
        }
    }

    fn save_snapshot(&self, snapshot_data: &InMemoryDB) -> Result<()> {
        let file = File::create(&self.snapshot_path)?;
        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, snapshot_data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    pub async fn load_snapshot(&mut self) -> Result<()> {
        if !self.snapshot_path.exists() {
            return Ok(());
        }
        let file = File::open(&self.snapshot_path)?;
        let reader = BufReader::new(file);
        let snapshot_data: InMemoryDB = bincode::deserialize_from(reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let mut db = self.in_memory_db.lock().unwrap();
        *db = snapshot_data;

        Ok(())
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