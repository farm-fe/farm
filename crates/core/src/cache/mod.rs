use std::{rc::Rc, sync::Arc};

use dashmap::DashMap;
use farmfe_utils::hash::sha256;
use parking_lot::Mutex;
use store::{
  constant::{CacheStoreFactory, CacheStoreTrait},
  CacheStoreKey, DiskCacheFactory,
};

use crate::{config::Mode, error::Result, Cacheable};

use self::plugin_cache::PluginCacheManager;

pub mod cacheable;
pub mod module_cache;
pub mod plugin_cache;
pub mod resource_cache;
pub mod store;
pub mod utils;
pub use module_cache::ModuleMatedataStore;

/// All cache related operation are charged by [CacheManager]
/// Note: that you should use CacheManager::new to create a new instance so that the cache can be read from disk.
/// It would do nothing if you create a new instance by CacheManager::default().
pub struct CacheManager {
  pub module_cache: module_cache::ModuleCacheManager,
  pub resource_cache: resource_cache::ResourceCacheManager,
  pub plugin_cache: PluginCacheManager,
  pub lazy_compile_store: Box<dyn CacheStoreTrait>,
  /// cache store for custom caches
  pub custom: Box<dyn CacheStoreTrait>,
  /// lock for cache manager
  pub lock: Mutex<bool>,
  pub enable: bool,
  _store: Box<dyn CacheStoreTrait>,
}

impl CacheManager {
  pub fn new(cache_dir: &str, namespace: &str, mode: Mode, matedata: Arc<DashMap<String, ModuleMatedataStore>>) -> Self {
    let store_factory: Rc<Box<dyn CacheStoreFactory>> =
      Rc::new(Box::new(DiskCacheFactory::new(cache_dir, namespace, mode)));

    let module_cache = module_cache::ModuleCacheManager::new(cache_dir, store_factory.clone(), matedata);
    let resource_cache = resource_cache::ResourceCacheManager::new(store_factory.clone());

    Self {
      module_cache,
      resource_cache,
      // plugin cache is not initialized here. it will be initialized when compile starts.
      plugin_cache: PluginCacheManager::new(store_factory.clone()),
      custom: store_factory.create_cache_store("custom"),
      lazy_compile_store: store_factory.create_cache_store("lazy-compilation"),
      lock: Mutex::new(false),
      enable: false,
      _store: store_factory.create_cache_store("_"),
    }
  }

  pub fn showdown(&self) {
    if !self.enable {
      return;
    }
    self._store.shutdown();
  }

  pub fn cache_enable(mut self, enable: bool) -> Self {
    self.enable = enable;

    self
  }

  pub fn write_cache(&self) {
    if !self.enable {
      return;
    }
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
    self.showdown();
    *lock = false;
  }
}
