use std::collections::HashMap;

use dashmap::DashMap;
use farmfe_utils::hash::sha256;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rkyv::Deserialize;

use crate::{
  cache::cache_store::{CacheStore, CacheStoreKey},
  config::Mode,
  deserialize,
  module::ModuleId,
  serialize,
};

use super::{module_memory_store::ModuleMemoryStore, CachedModule};

/// In memory store for mutable modules
pub struct MutableModulesMemoryStore {
  /// low level cache store
  store: CacheStore,
  /// ModuleId -> Cached Module
  cached_modules: DashMap<ModuleId, CachedModule>,
}

impl MutableModulesMemoryStore {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
    Self {
      store: CacheStore::new(cache_dir_str, namespace, mode, "mutable-modules"),
      cached_modules: DashMap::new(),
    }
  }
}

impl ModuleMemoryStore for MutableModulesMemoryStore {
  fn has_cache(&self, key: &ModuleId) -> bool {
    if self.cached_modules.contains_key(key) {
      return true;
    }

    self.store.has_cache(&key.to_string())
  }

  fn set_cache(&self, key: ModuleId, module: CachedModule) {
    self.cached_modules.insert(key, module);
  }

  fn get_cache(&self, key: &ModuleId) -> Option<CachedModule> {
    if let Some(module) = self.cached_modules.get(key) {
      return Some(module.value().clone());
    }

    let cache = self.store.read_cache(&key.to_string());

    if let Some(cache) = cache {
      let module = deserialize!(&cache, CachedModule);
      self.cached_modules.insert(key.clone(), module.clone());
      return Some(module);
    }

    None
  }

  fn get_cache_ref(
    &self,
    key: &ModuleId,
  ) -> Option<dashmap::mapref::one::Ref<'_, ModuleId, CachedModule>> {
    if let Some(module) = self.cached_modules.get(key) {
      return Some(module);
    }

    let cache = self.store.read_cache(&key.to_string());

    if let Some(cache) = cache {
      let module = deserialize!(&cache, CachedModule);
      self.cached_modules.insert(key.clone(), module);
      return Some(self.cached_modules.get(key).unwrap());
    }

    None
  }

  fn get_cache_mut_ref(
    &self,
    key: &ModuleId,
  ) -> Option<dashmap::mapref::one::RefMut<'_, ModuleId, CachedModule>> {
    if let Some(module) = self.cached_modules.get_mut(key) {
      return Some(module);
    }

    let cache = self.store.read_cache(&key.to_string());

    if let Some(cache) = cache {
      let module = deserialize!(&cache, CachedModule);
      self.cached_modules.insert(key.clone(), module);
      return Some(self.cached_modules.get_mut(key).unwrap());
    }

    None
  }

  fn write_cache(&self) {
    let mut cache_map = HashMap::new();

    for entry in self.cached_modules.iter() {
      let key = entry.key().clone();
      let module = entry.value();
      let hash_key = sha256(
        format!("{}{}", module.module.content_hash, key.to_string()).as_bytes(),
        32,
      );
      let store_key = CacheStoreKey {
        name: key.to_string(),
        key: hash_key,
      };

      if self.store.is_cache_changed(&store_key) {
        cache_map.insert(store_key, module.clone());
      }
    }

    let cache_map = cache_map
      .into_par_iter()
      .map(|(store_key, module)| (store_key, serialize!(&module)))
      .collect::<HashMap<_, _>>();

    self.store.write_cache(cache_map);
  }

  fn invalidate_cache(&self, key: &ModuleId) {
    self.cached_modules.remove(key);
  }
}
