use std::collections::HashMap;

use dashmap::DashMap;

use crate::config::Mode;

use super::cache_store::CacheStore;

#[derive(Default)]
pub struct PluginCacheManager {
  store: CacheStore,
  cache: DashMap<String, Vec<u8>>,
}

impl PluginCacheManager {
  pub fn new(cache_dir: &str, namespace: &str, mode: Mode) -> Self {
    let store = CacheStore::new(cache_dir, namespace, mode);
    Self {
      store,
      cache: DashMap::new(),
    }
  }

  pub fn read_cache(&self) -> HashMap<String, Vec<u8>> {
    self.store.read_cache("plugin")
  }

  pub fn set_cache(&self, cache: HashMap<String, Vec<u8>>) {
    for (key, value) in cache {
      self.cache.insert(key, value);
    }
  }

  pub fn write_cache_to_disk(&self) {
    let cache = self
      .cache
      .iter()
      .map(|entry| (entry.key().clone(), entry.value().clone()))
      .collect::<HashMap<_, _>>();
    self.store.write_cache(cache, "plugin");
  }
}
