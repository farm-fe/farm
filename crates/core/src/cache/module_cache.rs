use dashmap::{DashMap, DashSet};
use rkyv::Deserialize;
use std::collections::HashMap;

use farmfe_macro_cache_item::cache_item;

use crate::config::Mode;
use crate::module::{Module, ModuleId};
use crate::plugin::PluginAnalyzeDepsHookResultEntry;
use crate::{deserialize, serialize};

use super::cache_store::CacheStore;

pub struct ModuleCacheManager {
  store: CacheStore,
  cache: DashMap<String, Vec<u8>>,
  initial_cache_modules: DashSet<ModuleId>,
  new_cache_modules: DashSet<ModuleId>,
}

#[cache_item]
pub struct CachedModule {
  pub module: Module,
  pub deps: Vec<PluginAnalyzeDepsHookResultEntry>,
}

impl ModuleCacheManager {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
    let store = CacheStore::new(cache_dir_str, namespace, mode);
    let cache = store.read_cache("module");
    let start = std::time::Instant::now();
    let cache = cache
      .into_iter()
      .map(|(key, value)| (key, value.to_vec()))
      .collect::<DashMap<String, Vec<u8>>>();
    println!("read cache time: {:?}", start.elapsed());

    Self {
      store,
      cache,
      initial_cache_modules: DashSet::new(),
      new_cache_modules: DashSet::new(),
    }
  }

  pub fn cache(&self) -> &DashMap<String, Vec<u8>> {
    &self.cache
  }

  pub fn initial_cache_modules(&self) -> &DashSet<ModuleId> {
    &self.initial_cache_modules
  }

  pub fn new_cache_modules(&self) -> &DashSet<ModuleId> {
    &self.new_cache_modules
  }

  pub fn has_cache(&self, key: &str) -> bool {
    self.cache.contains_key(key)
  }

  pub fn set_cache(&self, key: &str, module: &CachedModule) {
    self.new_cache_modules.insert(module.module.id.clone());
    let bytes = serialize!(module);
    self.cache.insert(key.to_string(), bytes);
  }

  pub fn get_cache(&self, key: &str) -> CachedModule {
    let start = std::time::Instant::now();
    if let Some(bytes) = self.cache.get(key) {
      let cached_module = deserialize!(&bytes, CachedModule);
      if cached_module
        .module
        .id
        .to_string()
        .contains("node_modules/react-dom/")
      {
        println!(
          "deserialize time: {:?}, size: {} -> {:?}",
          cached_module.module.id,
          cached_module.module.size,
          start.elapsed()
        );
      }
      if !self.new_cache_modules.contains(&cached_module.module.id) {
        self
          .initial_cache_modules
          .insert(cached_module.module.id.clone());
      }

      cached_module
    } else {
      panic!("Module cache not found: {}", key);
    }
  }

  /// Write the cache map to the disk.
  pub fn write_cache(&self) {
    let mut cache_map = HashMap::new();

    for item in self.cache.iter() {
      cache_map.insert(item.key().to_string(), item.value().to_vec());
    }

    self.store.write_cache(cache_map, "module");
  }

  pub fn is_initial_cache(&self, module_id: &ModuleId) -> bool {
    self.initial_cache_modules.contains(module_id)
  }
}
