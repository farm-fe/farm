use dashmap::DashMap;

use crate::{cache::cache_store::CacheStore, config::Mode, module::Module};

/// In memory store for mutable modules
pub struct ImmutableModulesMemoryStore {
  /// low level cache store
  store: CacheStore,
  /// CacheKey -> Cached Module
  cached_modules: DashMap<String, Module>,
}

impl ImmutableModulesMemoryStore {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
    Self {
      store: CacheStore::new(cache_dir_str, namespace, mode, "immutable-modules"),
      cached_modules: DashMap::new(),
    }
  }

  pub fn has_cache(&self, key: &str) -> bool {
    if self.cached_modules.contains_key(key) {
      return true;
    }

    return self.store.has_cache(key);
  }

  pub fn set_cache(&self, key: ModuleId, module: CachedModule) {
    self.cached_modules.insert(key, module);
  }
}
