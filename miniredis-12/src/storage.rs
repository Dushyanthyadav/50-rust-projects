use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Db {
    shared: Arc<RwLock<HashMap<String, Bytes>>>,
}

impl Db {
    pub fn new() -> Db {
        Db {
            shared: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set(&self, key: String, value: Bytes) {
        let mut state = self.shared.write().unwrap();

        state.insert(key, value);

        // Lock is automatically released here when state goes out of scope
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        let state = self.shared.read().unwrap();

        // return a clone of the bytes (Bytes is cheap to clone)
        state.get(key).cloned()
    }

    pub fn del(&self, key: &str) -> bool {
        let mut state = self.shared.write().unwrap();
        state.remove(key).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_basic_crud() {
        let db = Db::new();

        // 1. Set
        db.set("key1".to_string(), Bytes::from("value1"));

        // 2. Get
        let val = db.get("key1").expect("Failed to get key1");
        assert_eq!(val, "value1");

        // 3. Delete
        db.del("key1");
        assert!(db.get("key1").is_none());
    }

    #[test]
    fn test_concurrent_access() {
        // Create the database
        let db = Db::new();

        // Clone the handle for the other thread
        // (This only increments the reference count, it doesn't copy the data)
        let db_clone = db.clone();

        // Spawn a new thread that writes to the DB
        let handle = std::thread::spawn(move || {
            // Write a value from Thread B
            db_clone.set(
                "concurrent_key".to_string(),
                Bytes::from("hello_from_thread"),
            );
        });

        // Wait for the thread to finish writing
        handle.join().unwrap();

        // Read the value from the Main Thread
        // If Arc/RwLock weren't working, this would fail or crash
        let val = db.get("concurrent_key").expect("Key should exist");
        assert_eq!(val, "hello_from_thread");
    }
}
