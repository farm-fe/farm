use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct WorkerCache {
  cache: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl WorkerCache {
  pub fn new() -> Self {
    WorkerCache {
      cache: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub fn get(&self, key: &str) -> Option<Vec<u8>> {
    let cache = self.cache.lock().unwrap();
    cache.get(key).cloned()
  }

  pub fn insert(&self, key: String, value: Vec<u8>) {
    let mut cache = self.cache.lock().unwrap();
    cache.insert(key, value);
  }
}
