use std::{collections::HashMap, sync::{Arc, RwLock}};

pub struct InMemoryDB {
    store: Arc<RwLock<HashMap<String, String>>>
}

impl InMemoryDB {
    pub fn new() -> Self {
        InMemoryDB {
            store: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub fn set(&self, key: String, value: String) {
        let mut store = self.store.write().unwrap();
        store.insert(key, value);
    }

    pub fn get(&self, key: String) -> Option<String> {
        let store = self.store.read().unwrap();
        store.get(&key).cloned()
    }

    pub fn delete(&self, key: String) {
        let mut store = self.store.write().unwrap();
        store.remove(&key);
    }
}