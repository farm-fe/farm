use dashmap::DashMap;

use crate::{cache::cache_store::CacheStore, config::Mode, module::ModuleId};

use super::CachedModule;

/// In memory store for mutable modules
pub struct MutableModulesMemoryStore {
  /// low level cache store
  store: CacheStore,
  /// CacheKey -> Cached Module
  cached_modules: DashMap<ModuleId, CachedModule>,
}

impl MutableModulesMemoryStore {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
    Self {
      store: CacheStore::new(cache_dir_str, namespace, mode, "mutable-modules"),
      cached_modules: DashMap::new(),
    }
  }

  pub fn has_cache(&self, key: &ModuleId) -> bool {
    if self.cached_modules.contains_key(key) {
      return true;
    }

    return self.store.has_cache(&key.to_string());
  }

  pub fn set_cache(&self, key: ModuleId, module: CachedModule) {
    self.cached_modules.insert(key, module);
  }

  pub fn get_cache(&self, key: &ModuleId) {}
}
