use std::collections::HashMap;

use crate::config::Mode;

use super::cache_store::CacheStore;

#[derive(Default)]
pub struct PluginCacheManager {
  store: CacheStore,
}

impl PluginCacheManager {
  pub fn new(cache_dir: &str, namespace: &str, mode: Mode) -> Self {
    let store = CacheStore::new(cache_dir, namespace, mode);
    Self { store }
  }

  pub fn read_cache(&self) -> HashMap<String, Vec<u8>> {
    self.store.read_cache("plugin")
  }

  pub fn write_cache(&self, cache: HashMap<String, Vec<u8>>) {
    self.store.write_cache(cache, "plugin");
  }
}
