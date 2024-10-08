use std::{collections::HashMap, sync::{Arc, RwLock}};

#[derive(Debug, Clone)]
pub struct InMemoryDB {
    store: Arc<RwLock<HashMap<String, String>>>
}

impl InMemoryDB {
    pub fn new() -> Self {
        InMemoryDB {
            store: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub fn set(&self, key: String, value: String) -> bool {
        let mut store = self.store.write().unwrap();
        store.insert(key, value);
        true
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let store = self.store.read().unwrap();
        store.get(key).cloned()
    }

    pub fn delete(&self, key: &str) -> bool {
        let mut store = self.store.write().unwrap();
        store.remove(key).is_some()
    }

    pub fn clone_state(&self) -> HashMap<String, String> {
        self.clone().store.read().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get_del() {
        let db = InMemoryDB::new();

        //Test set and get
        db.set("key1".to_string(), "val1".to_string());
        assert_eq!(db.get("key1"), Some("val1".to_string()));

        //Test getting not set key
        assert_eq!(db.get("key2"), None);

        //Test getting deleted key
        db.delete("key1");
        assert_eq!(db.get("key1"), None);
    }
}