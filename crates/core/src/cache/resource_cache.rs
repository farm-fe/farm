use dashmap::{DashMap, DashSet};
use rayon::prelude::*;
use rkyv::Deserialize;
use std::collections::HashMap;

use crate::config::Mode;
use crate::plugin::PluginGenerateResourcesHookResult;
use crate::resource::resource_pot::ResourcePotMetaData;

use crate::{deserialize, serialize};

use super::cache_store::CacheStore;

#[derive(Default)]
pub struct ResourceCacheManager {
  store: CacheStore,
  resource_pot_meta_store: CacheStore,
  cache: DashMap<String, Vec<u8>>,
  resource_pot_meta_cache: DashMap<String, Vec<u8>>,
  used_cached_resources: DashSet<String>,
}

impl ResourceCacheManager {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
    let start = std::time::Instant::now();
    let store = CacheStore::new(cache_dir_str, namespace, mode.clone());
    let resource_pot_meta_store = CacheStore::new(cache_dir_str, namespace, mode);

    let mut res = vec!["resource", "resource_pot_meta"]
      .into_par_iter()
      .map(|key| {
        store
          .read_cache(key)
          .into_iter()
          .map(|(key, value)| (key, value))
          .collect::<DashMap<String, Vec<u8>>>()
      })
      .collect::<Vec<_>>();
    let resource_pot_meta_cache = res.remove(1);
    let cache = res.remove(0);

    let cache = cache
      .into_iter()
      .map(|(key, value)| (key, value))
      .collect::<DashMap<String, Vec<u8>>>();
    let resource_pot_meta_cache = resource_pot_meta_cache
      .into_iter()
      .map(|(key, value)| (key, value))
      .collect::<DashMap<String, Vec<u8>>>();

    println!("read resource cache time: {:?}", start.elapsed());

    Self {
      store,
      cache,
      resource_pot_meta_store,
      resource_pot_meta_cache,
      used_cached_resources: DashSet::new(),
    }
  }

  pub fn has_resource_cache(&self, key: &str) -> bool {
    self.cache.contains_key(key)
  }

  pub fn has_resource_pot_meta_cache(&self, key: &str) -> bool {
    self.resource_pot_meta_cache.contains_key(key)
  }

  pub fn set_resource_cache(&self, key: &str, resource: &PluginGenerateResourcesHookResult) {
    self.add_used_resource(key.to_string());
    let bytes = serialize!(resource);
    self.cache.insert(key.to_string(), bytes);
  }

  pub fn set_resource_pot_meta_cache(&self, key: &str, resource: &ResourcePotMetaData) {
    let bytes = serialize!(resource);
    self.resource_pot_meta_cache.insert(key.to_string(), bytes);
  }

  pub fn get_resource_cache(&self, key: &str) -> PluginGenerateResourcesHookResult {
    if let Some(bytes) = self.cache.get(key) {
      self.add_used_resource(key.to_string());
      deserialize!(&bytes, PluginGenerateResourcesHookResult)
    } else {
      panic!("Resource cache not found: {}", key);
    }
  }

  pub fn get_resource_pot_meta_cache(&self, key: &str) -> ResourcePotMetaData {
    if let Some(bytes) = self.resource_pot_meta_cache.get(key) {
      self.add_used_resource(key.to_string());
      deserialize!(&bytes, ResourcePotMetaData)
    } else {
      panic!("Resource cache not found: {}", key);
    }
  }

  pub fn write_resource_cache(&self) {
    let mut cache_map = HashMap::new();

    for item in self.cache.iter() {
      if !self.used_cached_resources.contains(item.key()) {
        continue;
      }

      cache_map.insert(item.key().to_string(), item.value().to_vec());
    }

    self.store.write_cache(cache_map, "resource");
  }

  pub fn write_resource_pot_meta_cache(&self) {
    let mut resource_pot_meta_cache_map = HashMap::new();

    for item in self.resource_pot_meta_cache.iter() {
      if !self.used_cached_resources.contains(item.key()) {
        continue;
      }

      resource_pot_meta_cache_map.insert(item.key().to_string(), item.value().to_vec());
    }

    self
      .resource_pot_meta_store
      .write_cache(resource_pot_meta_cache_map, "resource_pot_meta");
  }

  /// Write the cache map to the disk.
  pub fn write_cache(&self) {
    // write cache in parallel
    let thread_pool = rayon::ThreadPoolBuilder::new()
      .num_threads(2)
      .build()
      .unwrap();
    thread_pool.install(|| {
      rayon::join(
        || self.write_resource_cache(),
        || self.write_resource_pot_meta_cache(),
      );
    });
  }

  pub fn add_used_resource(&self, key: String) {
    self.used_cached_resources.insert(key);
  }
}
