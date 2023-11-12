use crate::config::Mode;

use self::plugin_cache::PluginCacheManager;

pub mod cache_store;
pub mod module_cache;
pub mod plugin_cache;
pub mod resource_cache;

/// All cache related operation are charged by [CacheManager]
#[derive(Default)]
pub struct CacheManager {
  pub module_cache: module_cache::ModuleCacheManager,
  pub resource_cache: resource_cache::ResourceCacheManager,
  pub plugin_cache: PluginCacheManager,
}

impl CacheManager {
  pub fn new(cache_dir: &str, namespace: &str, mode: Mode) -> Self {
    let start = std::time::Instant::now();
    // initialize cache store in parallel
    let thread_pool = rayon::ThreadPoolBuilder::new()
      .num_threads(2)
      .build()
      .unwrap();
    let (module_cache, resource_cache) = thread_pool.install(|| {
      let cloned_mode = mode.clone();
      let (module_cache, resource_cache) = rayon::join(
        || {
          let module_cache =
            module_cache::ModuleCacheManager::new(cache_dir, namespace, cloned_mode);
          module_cache
        },
        || {
          let resource_cache =
            resource_cache::ResourceCacheManager::new(cache_dir, namespace, mode.clone());
          resource_cache
        },
      );

      (module_cache, resource_cache)
    });
    println!("read cache time: {:?}", start.elapsed());
    Self {
      module_cache,
      resource_cache,
      plugin_cache: PluginCacheManager::new(cache_dir, namespace, mode),
    }
  }

  pub fn write_cache(&self) {
    // write cache in parallel
    let thread_pool = rayon::ThreadPoolBuilder::new()
      .num_threads(2)
      .build()
      .unwrap();
    thread_pool.install(|| {
      rayon::join(
        || self.module_cache.write_cache(),
        || self.resource_cache.write_cache(),
      );
    });
  }
}
