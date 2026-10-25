use std::{path::PathBuf, sync::Arc};

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
pub use module_cache::ModuleMetadataStore;
mod scope;
pub use scope::ScopeRef;

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

#[derive(Debug, Default)]
pub enum CacheType {
  #[default]
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
  pub thread_pool: Arc<rayon::ThreadPool>,
}

pub struct CacheContext {
  pub cache_enable: bool,
  pub option: CacheType,
  pub store_factory: Box<dyn CacheStoreFactory>,
  pub thread_pool: Arc<rayon::ThreadPool>,
}

impl Default for CacheContext {
  fn default() -> Self {
    let cache_type = CacheType::default();
    Self {
      cache_enable: Default::default(),
      store_factory: cache_type.create_store_factory(),
      option: cache_type,
      thread_pool: Arc::new(
        rayon::ThreadPoolBuilder::new()
          .num_threads(num_cpus::get())
          .build()
          .unwrap(),
      ),
    }
  }
}

impl CacheManager {
  /// Physical storage of this manager, including disk format, namespace and
  /// mode. Its captured options remain authoritative after plugin config hooks.
  pub fn disk_cache_directory(&self) -> Option<PathBuf> {
    match &self.context.option {
      CacheType::Memory => None,
      CacheType::Disk {
        cache_dir,
        namespace,
        mode,
      } => Some(store::CacheStore::directory_for(
        cache_dir, namespace, *mode,
      )),
    }
  }

  pub fn new(option: CacheOption) -> Self {
    let store_factory: Box<dyn CacheStoreFactory> = option.option.create_store_factory();
    let context = Arc::new(CacheContext {
      cache_enable: option.cache_enable,
      option: option.option,
      store_factory,
      thread_pool: option.thread_pool,
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
    if !self.enable() {
      return;
    }

    self._store.shutdown();
  }

  pub fn write_cache(&self) {
    self.write_cache_with(|| {});
  }

  fn write_cache_with(&self, on_acquired: impl FnOnce()) {
    if !self.enable() {
      return;
    }
    // discard write if cannot get lock
    let Some(mut lock) = self.lock.try_lock() else {
      return;
    };

    on_acquired();
    *lock = true;
    // write cache in parallel

    self.context.thread_pool.install(|| {
      rayon::join(
        || self.module_cache.write_cache(),
        || {
          rayon::join(
            || self.resource_cache.write_cache(),
            || self.plugin_cache.write_cache_to_disk(),
          )
        },
      );

      self.showdown();
    });
    *lock = false;
  }
}

#[cfg(test)]
mod tests {
  use std::{
    sync::{
      atomic::{AtomicBool, Ordering},
      mpsc, Arc,
    },
    thread,
    time::Duration,
  };

  use super::{CacheManager, CacheOption, CacheType};

  #[test]
  fn concurrent_writer_is_discarded_until_active_write_finishes() {
    let manager = Arc::new(CacheManager::new(CacheOption {
      cache_enable: true,
      option: CacheType::Memory,
      thread_pool: Arc::new(
        rayon::ThreadPoolBuilder::new()
          .num_threads(1)
          .build()
          .expect("test thread pool"),
      ),
    }));
    let timeout = Duration::from_secs(20);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (resume_tx, resume_rx) = mpsc::channel();
    let first_manager = Arc::clone(&manager);
    let first = thread::spawn(move || {
      first_manager.write_cache_with(|| {
        entered_tx
          .send(())
          .expect("notify first writer eligibility");
        resume_rx
          .recv_timeout(timeout)
          .expect("release first writer");
      });
    });
    let first_entered = entered_rx.recv_timeout(timeout);
    let second_entered = Arc::new(AtomicBool::new(false));
    let second_probe = Arc::clone(&second_entered);
    let second_manager = Arc::clone(&manager);
    let (completed_tx, completed_rx) = mpsc::channel();
    let second = thread::spawn(move || {
      second_manager.write_cache_with(|| {
        second_probe.store(true, Ordering::SeqCst);
      });
      completed_tx.send(()).expect("notify second writer return");
    });
    let second_completed = completed_rx.recv_timeout(timeout);
    let second_entered_before_release = second_entered.load(Ordering::SeqCst);
    let released = resume_tx.send(());
    // Join both writers before assertions, including the old implementation's
    // failure path, so the test never leaves a paused writer behind.
    let first_result = first.join();
    let second_result = second.join();
    assert!(first_entered.is_ok(), "first writer must reach acquisition");
    assert!(
      released.is_ok(),
      "first writer must remain paused until release"
    );
    assert!(first_result.is_ok());
    assert!(second_result.is_ok());
    assert!(
      second_completed.is_ok(),
      "contending writer must return without waiting"
    );
    assert!(
      !second_entered_before_release,
      "contending writer must not enter the cache write after eligibility"
    );
    let mut third_entered = false;
    manager.write_cache_with(|| third_entered = true);
    assert!(
      third_entered,
      "a later writer must acquire the released lock"
    );
    assert!(!*manager.lock.lock());
  }
}
