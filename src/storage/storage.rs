use super::{in_memory_db::InMemoryDB, persistent_db::PersistentDB};

pub enum StorageMode {
    InMemory,
    Persistent
}

pub struct Storage {
    in_memory_db: InMemoryDB,
    persistent_db: PersistentDB
}

impl Storage {
    pub fn new(db_path: &str) -> Self {
        Storage {
            in_memory_db: InMemoryDB::new(),
            persistent_db: PersistentDB::new(db_path)
        }
    }

    pub fn set_in_memory(&self, key: String, value: String) -> bool {
        self.in_memory_db.set(key, value)
    }

    pub fn get_in_memory(&self, key: String) -> Option<String> {
        self.in_memory_db.get(key)
    }

    pub fn delete_in_memory(&self, key: String) -> bool {
        self.in_memory_db.delete(key)
    }

    pub fn set_persistent(&self, key: String, value: String) -> bool {
        self.set_persistent(key, value)
    }

    pub fn get_persistent(&self, key: &str) -> Option<String> {
        self.persistent_db.get(key)
    }

    pub fn delete_persistent(&self, key: &str) -> bool {
        self.persistent_db.delete(key)
    }
}