use parking_lot::Mutex;

use crate::config::Mode;

use self::{cache_store::CacheStore, plugin_cache::PluginCacheManager};

pub mod cache_store;
pub mod cacheable;
pub mod module_cache;
pub mod plugin_cache;
pub mod resource_cache;
pub mod utils;

/// All cache related operation are charged by [CacheManager]
/// Note: that you should use CacheManager::new to create a new instance so that the cache can be read from disk.
/// It would do nothing if you create a new instance by CacheManager::default().
pub struct CacheManager {
  pub module_cache: module_cache::ModuleCacheManager,
  pub resource_cache: resource_cache::ResourceCacheManager,
  pub plugin_cache: PluginCacheManager,
  pub lazy_compile_store: CacheStore,
  /// cache store for custom caches
  pub custom: CacheStore,
  /// lock for cache manager
  pub lock: Mutex<bool>,
}

impl CacheManager {
  pub fn new(cache_dir: &str, namespace: &str, mode: Mode) -> Self {
    let module_cache = module_cache::ModuleCacheManager::new(cache_dir, namespace, mode.clone());
    let resource_cache =
      resource_cache::ResourceCacheManager::new(cache_dir, namespace, mode.clone());

    Self {
      module_cache,
      resource_cache,
      // plugin cache is not initialized here. it will be initialized when compile starts.
      plugin_cache: PluginCacheManager::new(cache_dir, namespace, mode.clone()),
      custom: CacheStore::new(cache_dir, namespace, mode.clone(), "custom"),
      lazy_compile_store: CacheStore::new(cache_dir, namespace, mode, "lazy-compilation"),
      lock: Mutex::new(false),
    }
  }

  pub fn write_cache(&self) {
    // discard write if cannot get lock
    if self.lock.try_lock().is_none() {
      return;
    }

    let mut lock = self.lock.lock();
    *lock = true;
    // write cache in parallel
    let thread_pool = rayon::ThreadPoolBuilder::new()
      .num_threads(4)
      .build()
      .unwrap();
    thread_pool.install(|| {
      rayon::join(
        || self.module_cache.write_cache(),
        || {
          rayon::join(
            || self.resource_cache.write_cache(),
            || self.plugin_cache.write_cache_to_disk(),
          )
        },
      );
    });
    *lock = false;
  }
}
