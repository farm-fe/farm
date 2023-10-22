use crate::config::Mode;

pub mod cache_store;
pub mod module_cache;
pub mod resource_cache;

/// All cache related operation are charged by [CacheManager]
pub struct CacheManager {
  pub module_cache: module_cache::ModuleCacheManager,
  pub resource_cache: resource_cache::ResourceCacheManager,
}

impl CacheManager {
  pub fn new(cache_dir: &str, namespace: &str, mode: Mode) -> Self {
    Self {
      module_cache: module_cache::ModuleCacheManager::new(cache_dir, namespace, mode.clone()),
      resource_cache: resource_cache::ResourceCacheManager::new(cache_dir, namespace, mode),
    }
  }

  pub fn write_cache(&self) {
    self.module_cache.write_cache();
    self.resource_cache.write_cache();
  }
}
