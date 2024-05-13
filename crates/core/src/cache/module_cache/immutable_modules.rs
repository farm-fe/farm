use std::collections::{HashMap, HashSet};

use dashmap::DashMap;
use farmfe_macro_cache_item::cache_item;
use farmfe_utils::hash::sha256;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rkyv::Deserialize;

use crate::{
  cache::{
    cache_store::{CacheStore, CacheStoreKey},
    utils::cache_panic,
  },
  config::Mode,
  module::ModuleId,
};

use super::{module_memory_store::ModuleMemoryStore, CachedModule};

const MANIFEST_KEY: &str = "immutable-modules.json";

#[cache_item]
pub struct CachedPackage {
  pub list: Vec<CachedModule>,
  name: String,
  version: String,
}

impl CachedPackage {
  pub fn gen_key(name: &str, version: &str) -> String {
    format!("{}@{}", name, version)
  }

  pub fn key(&self) -> String {
    Self::gen_key(&self.name, &self.version)
  }
}

/// In memory store for mutable modules
pub struct ImmutableModulesMemoryStore {
  cache_dir: String,
  /// low level cache store
  store: CacheStore,
  /// ModuleId -> Cached Module
  cached_modules: DashMap<ModuleId, CachedModule>,
  /// moduleId -> PackageKey
  manifest: DashMap<ModuleId, String>,
  manifest_reversed: DashMap<String, HashSet<ModuleId>>,
}

impl ImmutableModulesMemoryStore {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
    let store = CacheStore::new(cache_dir_str, namespace, mode, "immutable-modules");

    let manifest_bytes = store.read_cache(MANIFEST_KEY).unwrap_or_default();
    let manifest: HashMap<String, String> =
      serde_json::from_slice(&manifest_bytes).unwrap_or_default();
    let manifest = manifest
      .into_iter()
      .map(|(key, value)| (ModuleId::from(key), value))
      .collect::<HashMap<ModuleId, String>>();

    let manifest_reversed = DashMap::new();

    for (key, value) in manifest.iter() {
      let mut set = manifest_reversed
        .entry(value.clone())
        .or_insert_with(HashSet::new);
      set.insert(key.clone());
    }

    Self {
      store,
      cached_modules: DashMap::new(),
      manifest: manifest.into_iter().collect(),
      manifest_reversed,
      cache_dir: cache_dir_str.to_string(),
    }
  }

  fn read_cached_package(&self, package_key: &str) -> CachedPackage {
    let cache = self
      .store
      .read_cache(package_key)
      .expect("Cache broken, please remove node_modules/.farm and retry.");

    crate::deserialize!(&cache, CachedPackage)
  }

  fn read_package(&self, module_id: &ModuleId) -> Option<()> {
    if let Some(package_key) = self.manifest.get(module_id) {
      let package = self.read_cached_package(package_key.value());

      for module in package.list {
        self.cached_modules.insert(module.module.id.clone(), module);
      }

      return Some(());
    }

    None
  }
}

impl ModuleMemoryStore for ImmutableModulesMemoryStore {
  fn has_cache(&self, key: &crate::module::ModuleId) -> bool {
    if self.cached_modules.contains_key(key) {
      return true;
    }

    if let Some(package_key) = self.manifest.get(key) {
      return self.store.has_cache(package_key.value());
    }

    false
  }

  fn set_cache(&self, key: crate::module::ModuleId, module: super::CachedModule) {
    self.cached_modules.insert(key, module);
  }

  fn get_cache(&self, key: &crate::module::ModuleId) -> Option<super::CachedModule> {
    if let Some(module) = self.cached_modules.remove(key).map(|item| item.1) {
      return Some(module);
    }

    if self.read_package(key).is_some() {
      return Some(
        self
          .cached_modules
          .remove(key)
          .map(|item| item.1)
          .expect("Cache broken, please remove node_modules/.farm and retry."),
      );
    }

    None
  }

  fn get_cache_ref(
    &self,
    key: &crate::module::ModuleId,
  ) -> Option<dashmap::mapref::one::Ref<'_, crate::module::ModuleId, super::CachedModule>> {
    if let Some(module) = self.cached_modules.get(key) {
      return Some(module);
    }

    if self.read_package(key).is_some() {
      return Some(
        self
          .cached_modules
          .get(key)
          .unwrap_or_else(|| cache_panic(&key.to_string(), &self.cache_dir)),
      );
    }

    None
  }

  fn get_cache_mut_ref(
    &self,
    key: &crate::module::ModuleId,
  ) -> Option<dashmap::mapref::one::RefMut<'_, crate::module::ModuleId, super::CachedModule>> {
    if self.cached_modules.contains_key(key) {
      return Some(self.cached_modules.get_mut(key).unwrap());
    }

    if self.read_package(key).is_some() {
      return Some(
        self
          .cached_modules
          .get_mut(key)
          .unwrap_or_else(|| cache_panic(&key.to_string(), &self.cache_dir)),
      );
    }

    None
  }

  fn write_cache(&self) {
    let mut packages = HashMap::new();

    for item in self.cached_modules.iter() {
      let module = item.value();
      let package_key =
        CachedPackage::gen_key(&module.module.package_name, &module.module.package_version);

      let package = packages.entry(package_key.clone()).or_insert_with(Vec::new);

      package.push(item.key().clone());
      self.manifest.insert(item.key().clone(), package_key);
    }

    let manifest = self
      .manifest
      .iter()
      .map(|item| (item.key().to_string(), item.value().to_string()))
      .collect::<HashMap<String, String>>();

    let manifest_bytes = serde_json::to_vec(&manifest)
      .unwrap_or_else(|e| cache_panic(&e.to_string(), &self.cache_dir));

    let mut cache_map = packages
      .into_par_iter()
      .filter_map(|(key, modules)| {
        let gen_cache_store_key = |mut modules: Vec<String>| {
          modules.sort();

          CacheStoreKey {
            name: key.clone(),
            key: sha256(modules.join(",").as_bytes(), 32),
          }
        };

        // the package is already cached, we only need to update it
        if self.manifest_reversed.contains_key(&key) {
          let modules_in_package = self.manifest_reversed.get(&key).unwrap();
          let mut added_modules = vec![];

          for module_id in modules {
            if modules_in_package.contains(&module_id) {
              continue;
            }
            added_modules.push(module_id);
          }

          // add the new modules to the package
          if !added_modules.is_empty() {
            let mut package = self.read_cached_package(&key);
            package.list.extend(
              added_modules
                .into_par_iter()
                .map(|module_id| {
                  self
                    .cached_modules
                    .get(&module_id)
                    .unwrap_or_else(|| cache_panic(&key.to_string(), &self.cache_dir))
                    .clone()
                })
                .collect::<Vec<_>>(),
            );
            let modules = package
              .list
              .iter()
              .map(|cm| cm.module.id.to_string())
              .collect::<Vec<_>>();
            let package_bytes = crate::serialize!(&package);
            return Some((gen_cache_store_key(modules), package_bytes));
          }
          return None;
        }

        let module_strings = modules.iter().map(|m| m.to_string()).collect::<Vec<_>>();
        let package = CachedPackage {
          list: modules
            .into_par_iter()
            .map(|module_id| {
              self
                .cached_modules
                .get(&module_id)
                .unwrap_or_else(|| cache_panic(&key.to_string(), &self.cache_dir))
                .clone()
            })
            .collect(),
          name: key.split('@').next().unwrap().to_string(),
          version: key.split('@').last().unwrap().to_string(),
        };

        let package_bytes = crate::serialize!(&package);
        Some((gen_cache_store_key(module_strings), package_bytes))
      })
      .collect::<HashMap<CacheStoreKey, Vec<u8>>>();

    cache_map.insert(
      CacheStoreKey {
        name: MANIFEST_KEY.to_string(),
        key: sha256(manifest_bytes.as_slice(), 32),
      },
      manifest_bytes,
    );

    self.store.write_cache(cache_map);
  }

  fn invalidate_cache(&self, key: &ModuleId) {
    self.cached_modules.remove(key);
    self.manifest.remove(key);
    self.manifest_reversed.iter_mut().for_each(|mut item| {
      if item.value_mut().contains(key) {
        item.value_mut().remove(key);
      }
    })
  }

  fn is_cache_changed(&self, module: &crate::module::Module) -> bool {
    // we do not need to check the hash of immutable modules, just check the cache
    !self.has_cache(&module.id)
  }

  fn cache_outdated(&self, key: &ModuleId) -> bool {
    !self.cached_modules.contains_key(key)
  }
}
