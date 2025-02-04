use dashmap::{mapref::multiple::RefMulti, DashMap};

use crate::HashMap;

use super::{constant::CacheStoreTrait, error::CacheError, CacheStoreKey};

struct MemoryCacheStore {
  cache: DashMap<String, Vec<u8>>,
  manifest: DashMap<String, String>,
}

impl MemoryCacheStore {
  pub fn new() -> Self {
    Self {
      cache: DashMap::default(),
      manifest: DashMap::default(),
    }
  }
}

impl CacheStoreTrait for MemoryCacheStore {
  fn has_cache(&self, _name: &str) -> bool {
    false
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
    self.cache.get(name).map(|v| v.value().clone())
  }

  fn remove_cache(&self, name: &str) {
    self.manifest.remove(name);
    self.cache.remove(name);
  }
}
