use dashmap::DashMap;
use rkyv::Deserialize;
use std::collections::HashMap;

use crate::config::Mode;
use crate::resource::resource_pot::ResourcePotMetaData;
use crate::resource::Resource;
use crate::{deserialize, serialize};

use super::cache_store::CacheStore;

pub struct ResourceCacheManager {
  store: CacheStore,
  resource_pot_meta_store: CacheStore,
  cache: DashMap<String, Vec<u8>>,
  resource_pot_meta_cache: DashMap<String, Vec<u8>>,
}

impl ResourceCacheManager {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
    let mut store = CacheStore::new(cache_dir_str, namespace, mode.clone());
    let mut resource_pot_meta_store = CacheStore::new(cache_dir_str, namespace, mode);
    store.set_items_per_cache_file(5);
    resource_pot_meta_store.set_items_per_cache_file(5);

    let cache = store.read_cache("resource");
    let resource_pot_meta_cache = store.read_cache("resource_pot_meta");

    let cache = cache
      .into_iter()
      .map(|(key, value)| (key, value.to_vec()))
      .collect::<DashMap<String, Vec<u8>>>();
    let resource_pot_meta_cache = resource_pot_meta_cache
      .into_iter()
      .map(|(key, value)| (key, value.to_vec()))
      .collect::<DashMap<String, Vec<u8>>>();

    Self {
      store,
      cache,
      resource_pot_meta_store,
      resource_pot_meta_cache,
    }
  }

  pub fn has_resource_cache(&self, key: &str) -> bool {
    self.cache.contains_key(key)
  }

  pub fn has_resource_pot_meta_cache(&self, key: &str) -> bool {
    self.resource_pot_meta_cache.contains_key(key)
  }

  pub fn set_resource_cache(&self, key: &str, resource: &Resource) {
    let bytes = serialize!(resource);
    self.cache.insert(key.to_string(), bytes);
  }

  pub fn set_resource_pot_meta_cache(&self, key: &str, resource: &ResourcePotMetaData) {
    let bytes = serialize!(resource);
    self.resource_pot_meta_cache.insert(key.to_string(), bytes);
  }

  pub fn get_resource_cache(&self, key: &str) -> Resource {
    if let Some(bytes) = self.cache.get(key) {
      deserialize!(&bytes, Resource)
    } else {
      panic!("Resource cache not found: {}", key);
    }
  }

  pub fn get_resource_pot_meta_cache(&self, key: &str) -> ResourcePotMetaData {
    if let Some(bytes) = self.resource_pot_meta_cache.get(key) {
      deserialize!(&bytes, ResourcePotMetaData)
    } else {
      panic!("Resource cache not found: {}", key);
    }
  }

  /// Write the cache map to the disk.
  pub fn write_cache(&self) {
    let mut cache_map = HashMap::new();

    for item in self.cache.iter() {
      cache_map.insert(item.key().to_string(), item.value().to_vec());
    }

    self.store.write_cache(cache_map, "resource");

    let mut resource_pot_meta_cache_map = HashMap::new();

    for item in self.resource_pot_meta_cache.iter() {
      resource_pot_meta_cache_map.insert(item.key().to_string(), item.value().to_vec());
    }

    self
      .resource_pot_meta_store
      .write_cache(resource_pot_meta_cache_map, "resource_pot_meta");
  }
}
