use std::rc::Rc;

use dashmap::mapref::one::{MappedRef, Ref, RefMut};

use farmfe_macro_cache_item::cache_item;
use module_metadata::ModuleMetadataStore;

use crate::module::module_graph::ModuleGraphEdge;
use crate::module::{CustomMetaDataMap, Module, ModuleId};
use crate::plugin::PluginAnalyzeDepsHookResultEntry;
use crate::Cacheable;

use immutable_modules::ImmutableModulesMemoryStore;
use module_memory_store::ModuleMemoryStore;
use mutable_modules::MutableModulesMemoryStore;

use super::store::constant::CacheStoreFactory;

pub mod immutable_modules;
pub mod module_memory_store;
mod module_metadata;
pub mod mutable_modules;

pub struct ModuleCacheManager {
  /// Store is responsible for how to read and load cache from disk.
  pub mutable_modules_store: MutableModulesMemoryStore,
  pub immutable_modules_store: ImmutableModulesMemoryStore,
  pub module_metadata: ModuleMetadataStore,
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
  pub fn new(cache_dir_str: &str, store: Rc<Box<dyn CacheStoreFactory>>) -> Self {
    Self {
      mutable_modules_store: MutableModulesMemoryStore::new(store.clone()),
      immutable_modules_store: ImmutableModulesMemoryStore::new(cache_dir_str, store.clone()),
      module_metadata: ModuleMetadataStore::new(store.clone()),
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

  pub fn get_cache_ref(&self, key: &ModuleId) -> Ref<'_, ModuleId, CachedModule> {
    if let Some(module) = self.mutable_modules_store.get_cache_ref(key) {
      return module;
    }

    self
      .immutable_modules_store
      .get_cache_ref(key)
      .expect("Cache broken, please remove node_modules/.farm and retry.")
  }

  pub fn get_cache_mut_ref(&self, key: &ModuleId) -> RefMut<'_, ModuleId, CachedModule> {
    if let Some(module) = self.mutable_modules_store.get_cache_mut_ref(key) {
      return module;
    }

    self
      .immutable_modules_store
      .get_cache_mut_ref(key)
      .expect("Cache broken, please remove node_modules/.farm and retry.")
  }

  /// Write the cache map to the disk.
  pub fn write_cache(&self) {
    let thread_pool = rayon::ThreadPoolBuilder::new()
      .num_threads(2)
      .build()
      .unwrap();

    thread_pool.install(|| {
      rayon::join(
        || self.mutable_modules_store.write_cache(),
        || {
          rayon::join(
            || self.immutable_modules_store.write_cache(),
            || self.module_metadata.write_cache(),
          )
        },
      );
    });
  }

  pub fn invalidate_cache(&self, key: &ModuleId) {
    self.mutable_modules_store.invalidate_cache(key);
    self.immutable_modules_store.invalidate_cache(key);
    self.module_metadata.invalidate(&key.to_string());
  }

  pub fn cache_outdated(&self, key: &ModuleId) -> bool {
    self.mutable_modules_store.cache_outdated(key)
      || self.immutable_modules_store.cache_outdated(key)
  }

  pub fn write_metadata<V: Cacheable>(&self, key: String, name: String, metadata: V) {
    self
      .module_metadata
      .write_metadata(key, name, Box::new(metadata));
  }

  pub fn read_metadata_ref<V: Cacheable>(
    &self,
    key: &str,
    name: &str,
  ) -> Option<MappedRef<'_, String, CustomMetaDataMap, V>> {
    self.module_metadata.read_ref(key, name)
  }

  pub fn metadata(&self, key: &str) -> CachedMetadataRef {
    CachedMetadataRef::new(self.module_metadata.read_mut_or_entry(key))
  }
}

pub struct CachedMetadataRef<'a> {
  value: RefMut<'a, String, CustomMetaDataMap>,
}

impl<'a> CachedMetadataRef<'a> {
  pub fn new(v: RefMut<'a, String, CustomMetaDataMap>) -> Self {
    Self { value: v }
  }

  pub fn read<V: Cacheable>(&mut self, name: &str) -> Option<&mut V> {
    self.value.value_mut().get_mut::<V>(name)
  }

  /// Get some data that does not need to be stored in memory after serialization
  pub fn get_cache<V: Cacheable>(&mut self, name: &str) -> Option<Box<V>> {
    self.value.value_mut().get_cache::<V>(name)
  }

  pub fn write<V: Cacheable, N: ToString>(&mut self, name: N, metadata: V) {
    self
      .value
      .value_mut()
      .insert(name.to_string(), Box::new(metadata));
  }
}
