use crate::config::Mode;

pub mod cache_store;
pub mod module_cache;
pub mod resource_cache;

/// All cache related operation are charged by [CacheManager]
pub struct CacheManager {
  pub module_cache: module_cache::ModuleCacheManager,
}

impl CacheManager {
  pub fn new(cache_dir: &str, namespace: &str, mode: Mode) -> Self {
    Self {
      module_cache: module_cache::ModuleCacheManager::new(cache_dir, namespace, mode),
    }
  }

  pub fn has_module_cache(&self, code_hash: &str) -> bool {
    self.module_cache.has_module_cache(code_hash)
  }
}
