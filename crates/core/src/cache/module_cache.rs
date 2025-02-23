use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;

use dashmap::mapref::one::{MappedRef, MappedRefMut, Ref, RefMut};

use dashmap::DashMap;
use farmfe_macro_cache_item::cache_item;
pub use module_metadata::ModuleMatedataStore;

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

type ModuleMatedataMap = Arc<DashMap<String, ModuleMatedataStore>>;

pub struct ModuleCacheManager {
  /// Store is responsible for how to read and load cache from disk.
  pub mutable_modules_store: MutableModulesMemoryStore,
  pub immutable_modules_store: ImmutableModulesMemoryStore,
  pub module_matedata: ModuleMatedataMap,
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

// #[cache_item]
// #[derive(Clone)]
// pub struct CachedModuleBasis {
//   pub module: Module,
//   pub dependencies: Vec<CachedModuleDependency>,
//   pub watch_dependencies: Vec<CachedWatchDependency>,
// }

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

  pub matedata: Option<HashMap<String, CustomMetaDataMap>>,
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
  pub fn new(
    cache_dir_str: &str,
    store: Rc<Box<dyn CacheStoreFactory>>,
    matedata: ModuleMatedataMap,
  ) -> Self {
    Self {
      mutable_modules_store: MutableModulesMemoryStore::new(store.clone()),
      immutable_modules_store: ImmutableModulesMemoryStore::new(cache_dir_str, store.clone()),
      module_matedata: matedata,
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

  pub fn set_cache(&self, key: ModuleId, mut module: CachedModule) {
    let map = self
      .module_matedata
      .iter()
      .fold(HashMap::new(), |mut res, v| {
        let plugin_name = v.key();

        if let Some(map) = v.get_map(&key) {
          res.insert(plugin_name.to_string(), map);
        }

        res
      });

    module.matedata = Some(map);

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
    let thread_pool = rayon::ThreadPoolBuilder::new()
      .num_threads(2)
      .build()
      .unwrap();

    thread_pool.install(|| {
      rayon::join(
        || self.mutable_modules_store.write_cache(),
        || self.immutable_modules_store.write_cache(),
      );
    });
  }

  pub fn invalidate_cache(&self, key: &ModuleId) {
    self.mutable_modules_store.invalidate_cache(key);
    self.immutable_modules_store.invalidate_cache(key);

    self.module_matedata.iter().for_each(|v| {
      v.invalidate(key);
    });
  }

  pub fn cache_outdated(&self, key: &ModuleId) -> bool {
    self.mutable_modules_store.cache_outdated(key)
      || self.immutable_modules_store.cache_outdated(key)
  }

  pub fn read_metadata<V: Cacheable>(
    &self,
    plugin_name: &str,
    key: &ModuleId,
    name: &str,
  ) -> Option<Box<V>> {
    // read from cached module
    if let Some(mut v) = self.get_cache_mut_option_ref(key) {
      return v
        .matedata
        .as_mut()
        .and_then(|v| v.get_mut(plugin_name).and_then(|v| v.get_cache(name)));
    }

    // read from matedata
    self
      .module_matedata
      .get(plugin_name)
      .and_then(|v: Ref<'_, String, ModuleMatedataStore>| v.get_matedata(key, name))
  }

  pub fn write_metadata<V: Cacheable>(
    &self,
    plugin_name: &str,
    key: ModuleId,
    name: String,
    value: V,
  ) {
    // write to cached module
    if let Some(mut v) = self.get_cache_mut_option_ref(&key) {
      let cached_module = v.value_mut();

      if cached_module.matedata.is_none() {
        cached_module.matedata = Default::default();
      }

      if let Some(ref mut matedata) = cached_module.matedata {
        if !matedata.contains_key(plugin_name) {
          matedata.insert(plugin_name.to_string(), Default::default());
        }

        if let Some(ref mut matedata) = matedata.get_mut(plugin_name) {
          matedata.insert(name, Box::new(value));
        }
      }

      return;
    }

    // write to matedata
    if !self.module_matedata.contains_key(plugin_name) {
      self
        .module_matedata
        .insert(plugin_name.to_string(), Default::default());
    }

    self.module_matedata.get(plugin_name).map(|v| {
      v.write_metadata(key, name, Box::new(value));
    });
  }

  // pub fn read_metadata_ref<V: Cacheable>(
  //   &self,
  //   key: &ModuleId,
  //   name: &str,
  // ) -> Option<MatedataMapRef<'_, V>> {
  //   let v = self.get_cache_option_ref(key).and_then(|v| {
  //     v.try_map(|v| v.metadata.as_ref().map(|v| v.get_ref(name)).flatten())
  //       .ok()
  //   });

  //   if v.is_some() {
  //     return v.map(MatedataMapRef::CachedMetadataRef);
  //   }

  //   self
  //     .module_matedata
  //     .read_ref(key, name)
  //     .map(MatedataMapRef::ModuleMetadata)
  // }

  // pub fn read_metadata_ref_mut<V: Cacheable>(
  //   &self,
  //   key: &ModuleId,
  //   name: &str,
  // ) -> Option<MatedataMapRefMut<'_, V>> {
  //   let v = self.get_cache_mut_option_ref(key).and_then(|v| {
  //     v.try_map(|v| v.metadata.as_mut().map(|v| v.get_mut(name)).flatten())
  //       .ok()
  //   });

  //   if v.is_some() {
  //     return v.map(MatedataMapRefMut::CachedMetadataRef);
  //   }

  //   self
  //     .module_matedata
  //     .read_ref_mut(key, name)
  //     .map(MatedataMapRefMut::ModuleMetadata)
  // }

  // pub fn metadata(&self, key: &ModuleId) -> Option<CachedMetadataRef> {
  //   self
  //     .get_cache_mut_option_ref(key)
  //     .map(MatedataRefMut::CachedMetadataRef)
  //     .or_else(|| {
  //       self
  //         .module_matedata
  //         .get_map_mut_ref(key)
  //         .map(MatedataRefMut::ModuleMetadata)
  //     })
  //     .map(CachedMetadataRef::new)
  // }
}

// pub enum MatedataMapRef<'a, V> {
//   CachedMetadataRef(MappedRef<'a, ModuleId, CachedModule, V>),
//   ModuleMetadata(MappedRef<'a, ModuleId, CustomMetaDataMap, V>),
// }

// impl<'a, V> Deref for MatedataMapRef<'a, V> {
//   type Target = V;

//   fn deref(&self) -> &Self::Target {
//     match self {
//       MatedataMapRef::CachedMetadataRef(v) => v.value(),
//       MatedataMapRef::ModuleMetadata(v) => v.value(),
//     }
//   }
// }

// pub enum MatedataMapRefMut<'a, V> {
//   CachedMetadataRef(MappedRefMut<'a, ModuleId, CachedModule, V>),
//   ModuleMetadata(MappedRefMut<'a, ModuleId, CustomMetaDataMap, V>),
// }

// impl<'a, V> Deref for MatedataMapRefMut<'a, V> {
//   type Target = V;

//   fn deref(&self) -> &Self::Target {
//     match self {
//       MatedataMapRefMut::CachedMetadataRef(v) => v.value(),
//       MatedataMapRefMut::ModuleMetadata(v) => v.value(),
//     }
//   }
// }

// impl<'a, V> DerefMut for MatedataMapRefMut<'a, V> {
//   fn deref_mut(&mut self) -> &mut V {
//     match self {
//       MatedataMapRefMut::CachedMetadataRef(v) => v.value_mut(),
//       MatedataMapRefMut::ModuleMetadata(v) => v.value_mut(),
//     }
//   }
// }

// pub enum MatedataRefMut<'a> {
//   CachedMetadataRef(RefMut<'a, ModuleId, CachedModule>),
//   ModuleMetadata(RefMut<'a, ModuleId, CustomMetaDataMap>),
// }

// pub struct CachedMetadataRef<'a> {
//   value: MatedataRefMut<'a>,
// }

// impl<'a> CachedMetadataRef<'a> {
//   pub fn new(v: MatedataRefMut<'a>) -> Self {
//     Self { value: v }
//   }

//   pub fn read<V: Cacheable>(&mut self, name: &str) -> Option<&mut V> {
//     match &mut self.value {
//       MatedataRefMut::CachedMetadataRef(ref_mut) => {
//         ref_mut.metadata.as_mut().and_then(|v| v.get_mut(name))
//       }
//       MatedataRefMut::ModuleMetadata(ref_mut) => ref_mut.get_mut::<V>(name),
//     }
//   }

//   /// Get some data that does not need to be stored in memory after serialization
//   pub fn get_cache<V: Cacheable>(&mut self, name: &str) -> Option<Box<V>> {
//     match &mut self.value {
//       MatedataRefMut::CachedMetadataRef(ref_mut) => {
//         ref_mut.metadata.as_mut().and_then(|v| v.get_cache(name))
//       }
//       MatedataRefMut::ModuleMetadata(ref_mut) => ref_mut.get_cache::<V>(name),
//     }
//   }

//   pub fn write<V: Cacheable, N: ToString>(&mut self, name: N, value: V) {
//     match &mut self.value {
//       MatedataRefMut::CachedMetadataRef(ref_mut) => {
//         if ref_mut.metadata.is_none() {
//           ref_mut.metadata = Some(CustomMetaDataMap::default());
//         }

//         if let Some(ref mut metadata) = ref_mut.metadata {
//           metadata.insert(name.to_string(), Box::new(value));
//         }
//       }
//       MatedataRefMut::ModuleMetadata(ref_mut) => {
//         ref_mut.insert(name.to_string(), Box::new(value));
//       }
//     }
//   }
// }
