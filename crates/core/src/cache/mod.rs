use std::sync::Arc;

use parking_lot::Mutex;
use store::{
  constant::{CacheStoreFactory, CacheStoreTrait},
  memory::MemoryCacheFactory,
  DiskCacheFactory,
};

use crate::config::Mode;

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
  _store: Box<dyn CacheStoreTrait>,
  context: Arc<CacheContext>,
}

pub enum CacheType {
  Memory,
  Disk {
    cache_dir: String,
    namespace: String,
    mode: Mode,
  },
}

impl CacheType {
  pub fn create_store_factory(&self) -> Box<dyn CacheStoreFactory> {
    match self {
      CacheType::Memory => Box::new(MemoryCacheFactory::new()),
      CacheType::Disk {
        cache_dir,
        namespace,
        mode,
      } => Box::new(DiskCacheFactory::new(cache_dir, namespace, *mode)),
    }
  }
}

pub struct CacheOption {
  pub cache_enable: bool,
  pub option: CacheType,
}

pub struct CacheContext {
  pub cache_enable: bool,
  pub option: CacheType,
  pub store_factory: Box<dyn CacheStoreFactory>,
}

impl CacheManager {
  pub fn new(option: CacheOption) -> Self {
    let store_factory: Box<dyn CacheStoreFactory> = option.option.create_store_factory();
    let context = Arc::new(CacheContext {
      cache_enable: option.cache_enable,
      option: option.option,
      store_factory,
    });

    let module_cache = module_cache::ModuleCacheManager::new(context.clone());
    let resource_cache = resource_cache::ResourceCacheManager::new(context.clone());

    Self {
      module_cache,
      resource_cache,
      // plugin cache is not initialized here. it will be initialized when compile starts.
      plugin_cache: PluginCacheManager::new(context.clone()),
      custom: context.store_factory.create_cache_store("custom"),
      lazy_compile_store: context.store_factory.create_cache_store("lazy-compilation"),
      lock: Mutex::new(false),
      _store: context.store_factory.create_cache_store("_"),
      context,
    }
  }

  #[inline]
  pub fn enable(&self) -> bool {
    self.context.cache_enable
  }

  pub fn showdown(&self) {
    if !self.context.cache_enable {
      return;
    }
    self._store.shutdown();
  }

  pub fn write_cache(&self) {
    if !self.context.cache_enable {
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
