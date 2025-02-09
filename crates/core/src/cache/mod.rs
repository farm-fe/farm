use std::rc::Rc;

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
}

impl CacheManager {
  pub fn new(cache_dir: &str, namespace: &str, mode: Mode) -> Self {
    let store_factory: Rc<Box<dyn CacheStoreFactory>> =
      Rc::new(Box::new(DiskCacheFactory::new(cache_dir, namespace, mode)));

    let module_cache = module_cache::ModuleCacheManager::new(cache_dir, store_factory.clone());
    let resource_cache = resource_cache::ResourceCacheManager::new(store_factory.clone());

    Self {
      module_cache,
      resource_cache,
      // plugin cache is not initialized here. it will be initialized when compile starts.
      plugin_cache: PluginCacheManager::new(store_factory.clone()),
      custom: store_factory.create_cache_store("custom"),
      lazy_compile_store: store_factory.create_cache_store("lazy-compilation"),
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

  pub fn write<V: Cacheable>(&self, name: &str, value: V) -> Result<()> {
    let bytes = value.serialize_bytes().unwrap();
    self
      .custom
      .write_single_cache(
        CacheStoreKey {
          name: name.to_string(),
          key: sha256(&bytes, 8),
        },
        bytes,
      )
      .unwrap();

    Ok(())
  }

  pub fn read<V: Cacheable>(&self, name: &str) -> Result<Option<Box<V>>> {
    let bytes = self.custom.read_cache(name).unwrap();

    let v = V::deserialize_bytes(bytes).unwrap();

    Ok(v.downcast::<V>().ok())
  }
}
