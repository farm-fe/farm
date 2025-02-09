use std::rc::Rc;

use dashmap::DashMap;
use farmfe_utils::hash::sha256;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rkyv::Deserialize;

use crate::{
  cache::store::{
    constant::{CacheStoreFactory, CacheStoreTrait},
    CacheStoreKey,
  },
  deserialize,
  module::ModuleId,
  serialize, HashMap,
};

use super::{module_memory_store::ModuleMemoryStore, CachedModule};

/// In memory store for mutable modules
pub struct MutableModulesMemoryStore {
  /// low level cache store
  store: Box<dyn CacheStoreTrait>,
  /// ModuleId -> Cached Module
  cached_modules: DashMap<ModuleId, CachedModule>,
}
// TODO: cache unit test
impl MutableModulesMemoryStore {
  pub fn new(store: Rc<Box<dyn CacheStoreFactory>>) -> Self {
    Self {
      store: store.create_cache_store("mutable-module"),
      cached_modules: DashMap::new(),
    }
  }

  fn gen_cache_store_key(&self, module: &crate::module::Module) -> CacheStoreKey {
    // Fix vue cache timestamp validation. Tools like vue may generate virtual modules which id is ends with .vue?vue,
    // if the original module is changed, but the virtual module's content hash is not changed, in this cache, the cache should be invalidated.
    let timestamp = if module.id.query_string().is_empty() {
      0
    } else {
      module.last_update_timestamp
    };
    let hash_key = sha256(
      format!(
        "{}{}{}",
        module.content_hash,
        module.id.to_string(),
        timestamp
      )
      .as_bytes(),
      32,
    );
    CacheStoreKey {
      name: module.id.to_string(),
      key: hash_key,
    }
  }
}

impl ModuleMemoryStore for MutableModulesMemoryStore {
  fn has_cache(&self, key: &ModuleId) -> bool {
    self.get_cache_ref(key).is_some_and(|m| !m.is_expired)
  }

  fn set_cache(&self, key: ModuleId, module: CachedModule) {
    self.cached_modules.insert(key, module);
  }

  fn get_cache(&self, key: &ModuleId) -> Option<CachedModule> {
    if let Some((_, module)) = self.cached_modules.remove(key) {
      return Some(module);
    }

    let cache = self.store.read_cache(&key.to_string());

    if let Some(cache) = cache {
      let module = deserialize!(&cache, CachedModule);
      // self.cached_modules.insert(key.clone(), module.clone());
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
    let mut cache_map = HashMap::default();
    let mut pending_removed_modules = vec![];

    for entry in self.cached_modules.iter() {
      let module = entry.value();
      if module.is_expired {
        pending_removed_modules.push(module.module.id.clone());
        continue;
      }

      let store_key = self.gen_cache_store_key(&module.module);

      if self.store.is_cache_changed(&store_key) {
        cache_map.insert(store_key, module.clone());
      }
    }

    let cache_map = cache_map
      .into_par_iter()
      .map(|(store_key, module)| (store_key, serialize!(&module)))
      .collect::<HashMap<_, _>>();

    self.store.write_cache(cache_map);

    for module_id in pending_removed_modules {
      self.cached_modules.remove(&module_id);
      self.store.remove_cache(&module_id.to_string());
    }
  }

  fn invalidate_cache(&self, key: &ModuleId) {
    if let Some(mut m) = self.get_cache_mut_ref(key) {
      m.is_expired = true;
    };
  }

  fn is_cache_changed(&self, module: &crate::module::Module) -> bool {
    let store_key = self.gen_cache_store_key(module);
    self.store.is_cache_changed(&store_key)
  }

  fn cache_outdated(&self, key: &ModuleId) -> bool {
    !self.cached_modules.contains_key(key)
  }
}
