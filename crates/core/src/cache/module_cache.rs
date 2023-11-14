use dashmap::mapref::one::{Ref, RefMut};
use dashmap::{DashMap, DashSet};
use rayon::prelude::*;
use rkyv::Deserialize;
use std::collections::{HashMap, HashSet};

use farmfe_macro_cache_item::cache_item;

use crate::config::Mode;
use crate::deserialize;
use crate::module::module_graph::ModuleGraphEdge;
use crate::module::{Module, ModuleId};
use crate::plugin::PluginAnalyzeDepsHookResultEntry;

use super::cache_store::CacheStore;

#[derive(Default)]
pub struct ModuleCacheManager {
  /// Store is responsible for how to read and load cache from disk.
  store: CacheStore,
  /// cache map for cache_key -> CachedModule.
  /// It's used for writing or reading data from store. New added cache will be also inserted into it directly.
  cache: DashMap<ModuleId, CachedModule>,
  initial_cached_bytes: DashMap<String, Vec<u8>>,
  initial_cached_modules_map: DashMap<ModuleId, CachedModule>,
  initial_cached_modules: HashSet<ModuleId>,
  invalidated_cached_modules: DashSet<ModuleId>,

  used_cached_modules: DashSet<ModuleId>,
}

#[cache_item]
#[derive(Debug, Clone)]
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
    let initial_cached_bytes = store.read_cache("module");

    let cache = initial_cached_bytes
      .par_iter()
      .map(|item| {
        let (key, value) = item;
        let cached_module = deserialize!(&value, CachedModule);
        (ModuleId::from(key.as_str()), cached_module)
      })
      .collect::<HashMap<_, _>>();

    let initial_cached_modules_map = cache.clone();

    let initial_cached_modules = cache
      .iter()
      .map(|item| item.0.clone())
      .collect::<HashSet<_>>();
    let cache = cache.into_iter().collect();
    let initial_cached_modules_map = initial_cached_modules_map.into_iter().collect();
    let initial_cached_bytes = initial_cached_bytes.into_iter().collect();
    println!(
      "read {} modules cache time: {:?}",
      initial_cached_modules.len(),
      start.elapsed()
    );

    Self {
      store,
      cache,
      initial_cached_bytes,
      initial_cached_modules_map,
      initial_cached_modules,
      invalidated_cached_modules: DashSet::new(),
      used_cached_modules: DashSet::new(),
    }
  }

  pub fn add_used_module(&self, module_id: &ModuleId) {
    self.used_cached_modules.insert(module_id.clone());
  }

  pub fn remove_used_module(&self, module_id: &ModuleId) {
    self.used_cached_modules.remove(module_id);
  }

  pub fn has_cache(&self, key: &ModuleId) -> bool {
    self.initial_cached_modules_map.contains_key(key)
  }

  pub fn set_cache(&self, key: ModuleId, module: CachedModule) {
    self.add_used_module(&key);
    self.cache.insert(key, module);
  }

  pub fn get_cache(&self, key: &ModuleId) -> CachedModule {
    if let Some((_, cached_module)) = self.initial_cached_modules_map.remove(key) {
      self.add_used_module(key);
      cached_module
    } else {
      panic!("Module cache not found: {:?}", key);
    }
  }

  pub fn get_cache_ref(&self, key: &ModuleId) -> Ref<'_, ModuleId, CachedModule> {
    if let Some(m_ref) = self.initial_cached_modules_map.get(key) {
      self.add_used_module(key);
      m_ref
    } else {
      panic!("Module cache not found: {:?}", key);
    }
  }

  pub fn get_cache_mut_ref(&self, key: &ModuleId) -> RefMut<'_, ModuleId, CachedModule> {
    if let Some(m_ref) = self.initial_cached_modules_map.get_mut(key) {
      self.add_used_module(key);
      m_ref
    } else {
      panic!("Module cache not found: {:?}", key);
    }
  }

  /// Write the cache map to the disk.
  pub fn write_cache(&self) {
    let mut cache_map = HashMap::new();

    for item in self.cache.iter() {
      if !self.used_cached_modules.contains(&item.key()) {
        continue;
      }

      let key_str = item.key().to_string();

      if let Some(entry) = self.initial_cached_bytes.get(&key_str) {
        if !self.invalidated_cached_modules.contains(item.key()) {
          cache_map.insert(entry.key().clone(), entry.value().to_vec());
          continue;
        }
      }

      let bytes = crate::serialize!(item.value());
      cache_map.insert(key_str, bytes);
    }

    self.store.write_cache(cache_map, "module");
  }

  pub fn is_initial_cache(&self, module_id: &ModuleId) -> bool {
    self.initial_cached_modules.contains(module_id)
      && !self.invalidated_cached_modules.contains(module_id)
  }

  pub fn get_initial_cached_modules(&self) -> Vec<ModuleId> {
    self.initial_cached_modules.iter().cloned().collect()
  }

  pub fn invalidate_cache(&self, module_id: &ModuleId) {
    self.cache.remove(module_id);
    self.invalidated_cached_modules.insert(module_id.clone());
  }
}
