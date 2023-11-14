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

use self::immutable_modules::ImmutableModulesMemoryStore;
use self::mutable_modules::MutableModulesMemoryStore;

use super::cache_store::CacheStore;

pub mod immutable_modules;
pub mod mutable_modules;

pub struct ModuleCacheManager {
  /// Store is responsible for how to read and load cache from disk.
  pub mutable_modules_store: MutableModulesMemoryStore,
  pub immutable_modules_store: ImmutableModulesMemoryStore,
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
  // pub last_update_timestamp: u128,
  // pub content_hash: String,
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
    Self {
      mutable_modules_store: MutableModulesMemoryStore::new(cache_dir_str, namespace, mode),
      immutable_modules_store: ImmutableModulesMemoryStore::new(cache_dir_str, namespace, mode),
    }
  }

  pub fn has_cache(&self, key: &ModuleId) -> bool {
    self.mutable_modules_store.has_cache(key) || self.immutable_modules_store.has_cache(key)
  }

  pub fn set_cache(&self, key: ModuleId, module: CachedModule) {
    if module.module.immutable {
      self.immutable_modules_store.set_cache(key, module);
    } else {
      self.mutable_modules_store.set_cache(key, module);
    }
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
}
