use std::sync::Arc;

use dashmap::mapref::one::{Ref, RefMut};

use farmfe_macro_cache_item::cache_item;
pub use module_metadata::ModuleMetadataStore;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::cache::scope::{CacheScopeStore, IdType, ScopeRef};
use crate::module::module_graph::ModuleGraphEdge;
use crate::module::{CustomMetaDataMap, Module, ModuleId};
use crate::plugin::PluginAnalyzeDepsHookResultEntry;
use crate::Cacheable;

use immutable_modules::ImmutableModulesMemoryStore;
use module_memory_store::ModuleMemoryStore;
use mutable_modules::MutableModulesMemoryStore;

use super::CacheContext;

pub mod immutable_modules;
pub mod module_memory_store;
mod module_metadata;
pub mod mutable_modules;

pub struct ModuleCacheManager {
  /// Store is responsible for how to read and load cache from disk.
  pub mutable_modules_store: MutableModulesMemoryStore,
  pub immutable_modules_store: ImmutableModulesMemoryStore,
  context: Arc<CacheContext>,
  scope: CacheScopeStore,
}

#[cache_item]
#[derive(Debug, Clone)]
pub struct CachedModuleDependency {
  pub dependency: ModuleId,
  pub edge_info: ModuleGraphEdge,
}

#[cache_item]
#[derive(Debug, Clone)]
pub struct CachedWatchDependency {
  pub dependency: ModuleId,
  pub timestamp: u128,
  pub hash: String,
}

#[cache_item]
#[derive(Clone)]
pub struct CachedModule {
  pub module: Module,
  pub dependencies: Vec<CachedModuleDependency>,
  pub watch_dependencies: Vec<CachedWatchDependency>,
  ///
  /// `default`: false
  ///
  /// true: it makes the cache expire.
  ///
  /// when writing to the cache next time, it will be cleared from memory.
  ///
  pub is_expired: bool,
}

impl CachedModule {
  pub fn dep_sources(
    dependencies: Vec<CachedModuleDependency>,
  ) -> Vec<(PluginAnalyzeDepsHookResultEntry, Option<ModuleId>)> {
    dependencies
      .into_iter()
      .flat_map(|dep| {
        let cloned_dep = dep.dependency;

        let mut sorted_dep = dep
          .edge_info
          .0
          .into_iter()
          .map(|item| (item.source, item.kind, item.order))
          .collect::<Vec<_>>();
        sorted_dep.sort_by(|a, b| a.2.cmp(&b.2));

        sorted_dep.into_iter().map(move |item| {
          (
            PluginAnalyzeDepsHookResultEntry {
              source: item.0,
              kind: item.1,
            },
            Some(cloned_dep.clone()),
          )
        })
      })
      .collect()
  }
}

impl ModuleCacheManager {
  pub fn new(context: Arc<CacheContext>) -> Self {
    Self {
      mutable_modules_store: MutableModulesMemoryStore::new(context.clone()),
      immutable_modules_store: ImmutableModulesMemoryStore::new(context.clone()),
      scope: CacheScopeStore::new(context.clone()),
      context,
    }
  }

  pub fn is_cache_changed(&self, module: &Module) -> bool {
    if module.immutable {
      return self.immutable_modules_store.is_cache_changed(module);
    }

    self.mutable_modules_store.is_cache_changed(module)
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
    if let Some(module) = self.mutable_modules_store.get_cache(key) {
      return module;
    }

    self
      .immutable_modules_store
      .get_cache(key)
      .expect("Cache broken, please remove node_modules/.farm and retry.")
  }

  fn get_cache_option_ref(&self, key: &ModuleId) -> Option<Ref<'_, ModuleId, CachedModule>> {
    self
      .mutable_modules_store
      .get_cache_ref(key)
      .or_else(|| self.immutable_modules_store.get_cache_ref(key))
  }

  pub fn get_cache_ref(&self, key: &ModuleId) -> Ref<'_, ModuleId, CachedModule> {
    self
      .get_cache_option_ref(key)
      .expect("Cache broken, please remove node_modules/.farm and retry.")
  }

  pub fn get_cache_mut_option_ref(
    &self,
    key: &ModuleId,
  ) -> Option<RefMut<'_, ModuleId, CachedModule>> {
    if !self.context.cache_enable {
      return None;
    }

    self
      .mutable_modules_store
      .get_cache_mut_ref(key)
      .or_else(|| self.immutable_modules_store.get_cache_mut_ref(key))
  }

  pub fn get_cache_mut_ref(&self, key: &ModuleId) -> RefMut<'_, ModuleId, CachedModule> {
    self
      .get_cache_mut_option_ref(key)
      .expect("Cache broken, please remove node_modules/.farm and retry.")
  }

  /// Write the cache map to the disk.
  pub fn write_cache(&self) {
    rayon::join(
      || self.mutable_modules_store.write_cache(),
      || {
        rayon::join(
          || self.immutable_modules_store.write_cache(),
          || {
            self.scope.write_cache();
          },
        )
      },
    );
  }

  pub fn invalidate_cache(&self, key: &ModuleId) {
    self.mutable_modules_store.invalidate_cache(key);
    self.immutable_modules_store.invalidate_cache(key);
    self.scope.remove_by_reference(&key.to_string());
  }

  pub fn cache_outdated(&self, key: &ModuleId) -> bool {
    self.mutable_modules_store.cache_outdated(key)
      || self.immutable_modules_store.cache_outdated(key)
  }

  pub fn read_metadata<V: Cacheable>(
    &self,
    name: &str,
    options: Option<MetadataOption>,
  ) -> Option<Box<V>> {
    self.scope.get(name, options.map(|o| o.into()).as_ref())
  }

  pub fn read_scope<V: Cacheable>(&self, name: &str) -> Vec<V> {
    self.scope.get_scope(name)
  }

  pub fn write_metadata<V: Cacheable>(
    &self,
    name: &str,
    value: V,
    options: Option<MetadataOption>,
  ) {
    self.scope.set(name, value, options.map(|o| o.into()));
  }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct MetadataOption {
  pub scope: Option<Vec<String>>,
  pub refer: Option<Vec<String>>,
}

impl MetadataOption {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn scope(mut self, scope: Vec<String>) -> Self {
    self.scope = Some(scope);
    self
  }

  pub fn refer<T>(mut self, refer: Vec<T>) -> Self
  where
    T: ToString,
  {
    self.refer = Some(refer.into_iter().map(|v| v.to_string()).collect());
    self
  }
}

impl Into<Vec<IdType>> for MetadataOption {
  fn into(self) -> Vec<IdType> {
    let mut ids = vec![];
    if let Some(scope) = self.scope {
      for s in scope {
        ids.push(IdType::Scope(s));
      }
    }
    if let Some(refer) = self.refer {
      for r in refer {
        ids.push(IdType::Reference(r));
      }
    }
    ids
  }
}
