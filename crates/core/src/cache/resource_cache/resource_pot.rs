use std::rc::Rc;

use dashmap::DashMap;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rkyv::Deserialize;

use crate::{
  cache::store::{
    constant::{CacheStoreFactory, CacheStoreTrait},
    CacheStoreKey,
  },
  deserialize, serialize, HashMap,
};

use super::resource_memory_store::{CachedResourcePot, ResourceMemoryStore};

/// In memory store for Resource Pot
pub struct ResourcePotMemoryStore {
  /// low level cache store
  store: Box<dyn CacheStoreTrait>,
  /// resource pot id -> Cached Resource Pot
  cached_resources: DashMap<String, CachedResourcePot>,
}

impl ResourcePotMemoryStore {
  pub fn new(store_factory: Rc<Box<dyn CacheStoreFactory>>) -> Self {
    let store = store_factory.create_cache_store("resource");

    Self {
      store,
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
    if let Some((_, resource)) = self.cached_resources.remove(name) {
      return Some(resource);
    }

    let cache = self.store.read_cache(name);

    if let Some(cache) = cache {
      let resource = deserialize!(&cache, CachedResourcePot);
      // self
      //   .cached_resources
      //   .insert(name.to_string(), resource.clone());
      return Some(resource);
    }

    None
  }

  fn write_cache(&self) {
    let mut cache_map = HashMap::default();

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
