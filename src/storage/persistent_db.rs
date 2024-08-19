use std::sync::{Arc, RwLock};
use rocksdb::{DB, Options};


struct PersistentDB {
    rocksDb: Arc<RwLock<DB>>,
}

impl PersistentDB {
    pub fn new(path: &str) -> Self {
        let mut options = Options::default();
        options.create_if_missing(true);
        let db = DB::open(&options, path).unwrap();
        PersistentDB {
            rocksDb: Arc::new(RwLock::new(db))
        }
    }

    pub fn set(&self, key: String, value: String) -> bool {
        let db = self.rocksDb.write().unwrap();
        db.put(key, value).is_ok()
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let db = self.rocksDb.read().unwrap();
        match db.get(key.as_bytes()) {
            Ok(Some(value)) => Some(String::from_utf8(value).unwrap()),
            Ok(None) => None,
            Err(_) => None
        }
    }

    pub fn delete(&self, key: &str) -> bool {
        let db = self.rocksDb.write().unwrap();
        db.delete(key).is_ok()
    }
}