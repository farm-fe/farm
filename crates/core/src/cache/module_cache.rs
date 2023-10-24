use dashmap::mapref::one::{Ref, RefMut};
use dashmap::{DashMap, DashSet};
use rayon::prelude::*;
use rkyv::Deserialize;
use std::collections::HashMap;

use farmfe_macro_cache_item::cache_item;

use crate::config::Mode;
use crate::deserialize;
use crate::module::module_graph::ModuleGraphEdge;
use crate::module::{Module, ModuleId};
use crate::plugin::PluginAnalyzeDepsHookResultEntry;

use super::cache_store::CacheStore;

pub struct ModuleCacheManager {
  /// Store is responsible for how to read and load cache from disk.
  store: CacheStore,
  /// cache map for cache_key -> serialized CachedModule.
  /// It's used for writing or reading data from store. New added cache will be also inserted into it directly.
  cache: DashMap<ModuleId, CachedModule>,
  initial_cached_modules_map: DashMap<ModuleId, CachedModule>,
  invalidated_cached_modules: DashSet<ModuleId>,
}

#[cache_item]
#[derive(Clone)]
pub struct CachedModuleDependency {
  pub dependency: ModuleId,
  pub edge_info: ModuleGraphEdge,
}

#[cache_item]
#[derive(Clone)]
pub struct CachedModule {
  pub module: Module,
  pub dependencies: Vec<CachedModuleDependency>,
  pub last_update_timestamp: u128,
  pub content_hash: String,
  pub package_name: String,
  pub package_version: String,
}

impl CachedModule {
  pub fn dep_sources(
    dependencies: Vec<CachedModuleDependency>,
  ) -> Vec<PluginAnalyzeDepsHookResultEntry> {
    dependencies
      .into_iter()
      .flat_map(|dep| {
        dep
          .edge_info
          .0
          .into_iter()
          .map(|item| PluginAnalyzeDepsHookResultEntry {
            source: item.source,
            kind: item.kind,
          })
      })
      .collect()
  }
}

impl ModuleCacheManager {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
    let start = std::time::Instant::now();

    let store = CacheStore::new(cache_dir_str, namespace, mode);
    let cache = store
      .read_cache("module")
      .into_par_iter()
      .map(|item| {
        let (key, value) = item;
        let cached_module = deserialize!(&value, CachedModule);
        (ModuleId::from(key.as_str()), cached_module)
      })
      .collect::<HashMap<_, _>>();

    let initial_cached_modules_map = cache.clone();

    let cache = cache.into_iter().collect();
    let initial_cached_modules_map = initial_cached_modules_map.into_iter().collect();

    println!("read cache time: {:?}", start.elapsed());

    Self {
      store,
      cache,
      initial_cached_modules_map,
      invalidated_cached_modules: DashSet::new(),
    }
  }

  pub fn has_cache(&self, key: &ModuleId) -> bool {
    self.initial_cached_modules_map.contains_key(key)
  }

  pub fn set_cache(&self, key: ModuleId, module: CachedModule) {
    self.cache.insert(key, module);
  }

  pub fn get_cache(&self, key: &ModuleId) -> CachedModule {
    if let Some((_, cached_module)) = self.initial_cached_modules_map.remove(key) {
      cached_module
    } else {
      panic!("Module cache not found: {:?}", key);
    }
  }

  pub fn get_cache_ref(&self, key: &ModuleId) -> Ref<'_, ModuleId, CachedModule> {
    if let Some(m_ref) = self.initial_cached_modules_map.get(key) {
      m_ref
    } else {
      panic!("Module cache not found: {:?}", key);
    }
  }

  pub fn get_cache_mut_ref(&self, key: &ModuleId) -> RefMut<'_, ModuleId, CachedModule> {
    if let Some(m_ref) = self.initial_cached_modules_map.get_mut(key) {
      m_ref
    } else {
      panic!("Module cache not found: {:?}", key);
    }
  }

  /// Write the cache map to the disk.
  pub fn write_cache(&self) {
    let mut cache_map = HashMap::new();

    for item in self.cache.iter() {
      let bytes = crate::serialize!(item.value());
      cache_map.insert(item.key().to_string(), bytes);
    }

    self.store.write_cache(cache_map, "module");
  }

  pub fn is_initial_cache(&self, module_id: &ModuleId) -> bool {
    self.initial_cached_modules_map.contains_key(module_id)
  }

  pub fn get_initial_cached_modules(&self) -> Vec<ModuleId> {
    self
      .initial_cached_modules_map
      .iter()
      .map(|item| item.key().clone())
      .collect()
  }

  pub fn invalidate_cache(&self, module_id: &ModuleId) {
    self.invalidated_cached_modules.insert(module_id.clone());
  }
}
