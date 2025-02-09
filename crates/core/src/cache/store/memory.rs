use dashmap::{mapref::multiple::RefMulti, DashMap};

use crate::HashMap;

use super::{constant::{CacheStoreFactory, CacheStoreTrait}, error::CacheError, CacheStoreKey};

#[derive(Default)]
pub struct MemoryCacheStore {
  cache: DashMap<String, Vec<u8>>,
  manifest: DashMap<String, String>,
}

impl MemoryCacheStore {
  pub fn new() -> Self {
    Default::default()
  }
}

impl CacheStoreTrait for MemoryCacheStore {
  fn has_cache(&self, name: &str) -> bool {
    self.manifest.contains_key(name)
  }

  fn get_store_keys(&self) -> Vec<RefMulti<String, String>> {
    self.manifest.iter().collect()
  }

  fn is_cache_changed(&self, store_key: &CacheStoreKey) -> bool {
    if let Some(guard) = self.manifest.get(&store_key.name) {
      if guard.value() == &store_key.key {
        return false;
      }
    }

    true
  }

  fn write_single_cache(&self, store_key: CacheStoreKey, bytes: Vec<u8>) -> Result<(), CacheError> {
    if self.is_cache_changed(&store_key) {
      self
        .manifest
        .insert(store_key.name.clone(), store_key.key.clone());
      self.cache.insert(store_key.key.clone(), bytes);
    }

    Ok(())
  }

  fn write_manifest(&self) {}

  fn write_cache(&self, cache_map: HashMap<CacheStoreKey, Vec<u8>>) {
    for (store_key, bytes) in cache_map {
      self.write_single_cache(store_key, bytes).unwrap();
    }
  }

  fn read_cache(&self, name: &str) -> Option<Vec<u8>> {
    if let Some(key) = self.manifest.get(name) {
      return self.cache.get(key.value()).map(|v| v.value().clone());
    }

    None
  }

  fn remove_cache(&self, name: &str) {
    self.manifest.remove(name);
    self.cache.remove(name);
  }
}

#[derive(Default)]
pub struct MemoryCacheFactory {}

impl MemoryCacheFactory {
  pub fn new() -> Self {
    Default::default()
  }
}

// TODO: namespace by name for memory
impl CacheStoreFactory for MemoryCacheFactory {
  fn create_cache_store(&self, _name: &str) -> Box<dyn CacheStoreTrait> {
    Box::new(MemoryCacheStore::new())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn t1() {
    let store = MemoryCacheStore::new();

    let bytes = vec![1, 2, 3];

    let name = "namespace".to_string();

    store
      .write_single_cache(
        CacheStoreKey {
          name: name.clone(),
          key: "hash".to_string(),
        },
        bytes.clone(),
      )
      .unwrap();

    assert_eq!(store.read_cache(&name), Some(bytes));
  }
}
