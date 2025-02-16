use std::rc::Rc;

use dashmap::{mapref::one::Ref, DashMap};
use farmfe_utils::hash::sha256;

use crate::{Cacheable, HashMap};

use super::store::constant::{CacheStoreFactory, CacheStoreTrait};

pub struct PluginCacheManager {
  store: Box<dyn CacheStoreTrait>,
  cache: DashMap<String, Vec<u8>>,
}

impl PluginCacheManager {
  pub fn new(store_factory: Rc<Box<dyn CacheStoreFactory>>) -> Self {
    let store = store_factory.create_cache_store("plugin");
    Self {
      store,
      cache: DashMap::new(),
    }
  }

  fn normalize_plugin_name(&self, plugin_name: &str) -> String {
    // replace all non-alphanumeric characters with _
    plugin_name
      .chars()
      .map(|c| if c.is_alphanumeric() { c } else { '_' })
      .collect::<String>()
  }

  pub fn read_cache(&self, plugin_name: &str) -> Option<Ref<'_, String, Vec<u8>>> {
    let plugin_name = self.normalize_plugin_name(plugin_name);

    if self.cache.contains_key(&plugin_name) {
      return self.cache.get(&plugin_name);
    }

    let cache = self.store.read_cache(&plugin_name);

    if let Some(cache) = cache {
      self.cache.insert(plugin_name.clone(), cache);
      return self.cache.get(&plugin_name);
    }

    None
  }

  pub fn set_cache(&self, plugin_name: &str, cache: Vec<u8>) {
    self
      .cache
      .insert(self.normalize_plugin_name(plugin_name), cache);
  }

  pub fn write_cache_item<V: Cacheable>(&self, plugin_name: &str, cache: V) {
    self.cache.insert(
      self.normalize_plugin_name(plugin_name),
      cache.serialize_bytes().unwrap(),
    );
  }

  pub fn read_cache_item<V: Cacheable>(&self, plugin_name: &str) -> Option<Box<V>> {
    let cache = self.read_cache(plugin_name)?;

    V::deserialize_bytes(cache.value().clone())
      .map(|v| v.downcast::<V>().ok())
      .ok()
      .flatten()
  }

  pub fn write_cache_to_disk(&self) {
    let cache = self
      .cache
      .iter()
      .map(|entry| {
        (
          (entry.key().clone(), sha256(entry.value(), 32)).into(),
          entry.value().clone(),
        )
      })
      .collect::<HashMap<_, _>>();

    if !cache.is_empty() {
      self.store.write_cache(cache);
    }
  }
}
