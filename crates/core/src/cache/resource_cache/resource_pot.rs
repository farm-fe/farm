use std::collections::HashMap;

use dashmap::DashMap;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rkyv::Deserialize;

use crate::{
  cache::cache_store::{CacheStore, CacheStoreKey},
  config::Mode,
  deserialize, serialize,
};

use super::resource_memory_store::{CachedResourcePot, ResourceMemoryStore};

/// In memory store for Resource Pot
pub struct ResourcePotMemoryStore {
  /// low level cache store
  store: CacheStore,
  /// resource pot id -> Cached Resource Pot
  cached_resources: DashMap<String, CachedResourcePot>,
}

impl ResourcePotMemoryStore {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
    Self {
      store: CacheStore::new(cache_dir_str, namespace, mode, "resource"),
      cached_resources: DashMap::new(),
    }
  }

  pub fn is_cache_changed(&self, name: String, hash: String) -> bool {
    self
      .store
      .is_cache_changed(&CacheStoreKey { name, key: hash })
  }
}

impl ResourceMemoryStore for ResourcePotMemoryStore {
  fn has_cache(&self, name: &str) -> bool {
    if self.cached_resources.contains_key(name) {
      return true;
    }

    self.store.has_cache(name)
  }

  fn set_cache(&self, name: &str, resource: CachedResourcePot) {
    self.cached_resources.insert(name.to_string(), resource);
  }

  fn get_cache(&self, name: &str) -> Option<CachedResourcePot> {
    if let Some(resource) = self.cached_resources.get(name) {
      return Some(resource.value().clone());
    }

    let cache = self.store.read_cache(name);

    if let Some(cache) = cache {
      let resource = deserialize!(&cache, CachedResourcePot);
      self
        .cached_resources
        .insert(name.to_string(), resource.clone());
      return Some(resource);
    }

    None
  }

  fn write_cache(&self) {
    let mut cache_map = HashMap::new();

    for entry in self.cached_resources.iter() {
      let store_key = CacheStoreKey {
        name: entry.key().clone(),
        key: entry.value().hash.clone(),
      };

      if self.store.is_cache_changed(&store_key) {
        cache_map.insert(store_key, entry);
      }
    }

    let cache_map = cache_map
      .into_par_iter()
      .map(|(store_key, resource)| (store_key, serialize!(resource.value())))
      .collect::<HashMap<_, _>>();

    self.store.write_cache(cache_map);
  }
}
