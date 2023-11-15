use std::collections::HashMap;

use dashmap::DashMap;
use farmfe_macro_cache_item::cache_item;
use farmfe_utils::hash::sha256;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rkyv::Deserialize;

use crate::{
  cache::cache_store::{CacheStore, CacheStoreKey},
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
  /// low level cache store
  store: CacheStore,
  /// ModuleId -> Cached Module
  cached_modules: DashMap<ModuleId, CachedModule>,
  /// moduleId -> PackageKey
  manifest: DashMap<ModuleId, String>,
}

impl ImmutableModulesMemoryStore {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
    let store = CacheStore::new(cache_dir_str, namespace, mode, "immutable-modules");

    let manifest_bytes = store.read_cache(MANIFEST_KEY).unwrap_or_default();
    let manifest: HashMap<String, String> =
      serde_json::from_slice(&manifest_bytes).unwrap_or_default();

    Self {
      store,
      cached_modules: DashMap::new(),
      manifest: manifest
        .into_iter()
        .map(|(key, value)| (ModuleId::from(key), value))
        .collect(),
    }
  }

  /// Get all modules, if not loaded, read from disk
  pub fn get_modules(&self) -> Vec<ModuleId> {
    if self.cached_modules.is_empty() {
      let store_keys = self.store.get_store_keys();

      store_keys.into_par_iter().for_each(|item| {
        if item.key() == MANIFEST_KEY {
          return;
        }

        let cache = self
          .store
          .read_cache(item.key())
          .expect("Cache broken, please remove node_modules/.farm and retry.");

        let package = crate::deserialize!(&cache, CachedPackage);

        for module in package.list {
          self.cached_modules.insert(module.module.id.clone(), module);
        }
      });
    }

    self
      .cached_modules
      .iter()
      .map(|item| item.key().clone())
      .collect()
  }

  fn read_package(&self, module_id: &ModuleId) -> Option<()> {
    if let Some(package_key) = self.manifest.get(module_id) {
      let cache = self
        .store
        .read_cache(package_key.value())
        .expect("Cache broken, please remove node_modules/.farm and retry.");

      let package = crate::deserialize!(&cache, CachedPackage);

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
    if self.cached_modules.contains_key(key) {
      return Some(self.cached_modules.get(key).unwrap().clone());
    }

    if self.read_package(key).is_some() {
      return Some(
        self
          .cached_modules
          .get(key)
          .expect("Cache broken, please remove node_modules/.farm and retry.")
          .clone(),
      );
    }

    None
  }

  fn get_cache_ref(
    &self,
    key: &crate::module::ModuleId,
  ) -> Option<dashmap::mapref::one::Ref<'_, crate::module::ModuleId, super::CachedModule>> {
    if self.cached_modules.contains_key(key) {
      return Some(self.cached_modules.get(key).unwrap());
    }

    if self.read_package(key).is_some() {
      return Some(
        self
          .cached_modules
          .get(key)
          .expect("Cache broken, please remove node_modules/.farm and retry."),
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
          .expect("Cache broken, please remove node_modules/.farm and retry."),
      );
    }

    None
  }

  fn write_cache(&self) {
    let mut packages = HashMap::new();

    for item in self.cached_modules.iter() {
      let module = item.value();
      let package_key = CachedPackage::gen_key(&module.package_name, &module.package_version);

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
      .expect("Cache broken, please remove node_modules/.farm and retry.");

    let mut cache_map = packages
      .into_par_iter()
      .filter_map(|(key, modules)| {
        let module_ids_str = modules
          .iter()
          .map(|item| item.to_string())
          .collect::<Vec<String>>()
          .join(",");
        let package_hash = sha256(module_ids_str.as_bytes(), 32);

        let store_key = CacheStoreKey {
          name: key.clone(),
          key: package_hash,
        };
        // skip clone if the cache is not changed
        if !self.store.is_cache_changed(&store_key) {
          return None;
        }

        let package = CachedPackage {
          list: modules
            .into_par_iter()
            .map(|module_id| {
              self
                .cached_modules
                .get(&module_id)
                .expect("Cache broken, please remove node_modules/.farm and retry.")
                .clone()
            })
            .collect(),
          name: key.split('@').next().unwrap().to_string(),
          version: key.split('@').last().unwrap().to_string(),
        };

        let package_bytes = crate::serialize!(&package);

        Some((store_key, package_bytes))
      })
      .collect::<HashMap<CacheStoreKey, Vec<u8>>>();

    cache_map.insert(
      CacheStoreKey {
        name: MANIFEST_KEY.to_string(),
        key: MANIFEST_KEY.to_string(),
      },
      manifest_bytes,
    );

    self.store.write_cache(cache_map);
  }

  fn invalidate_cache(&self, key: &ModuleId) {
    self.cached_modules.remove(key);
  }
}
