use dashmap::{mapref::one::Ref, DashMap};
use farmfe_utils::hash::sha256;

use crate::HashMap;

use crate::config::Mode;

use super::cache_store::{CacheStore, CacheStoreKey};

#[derive(Default)]
pub struct PluginCacheManager {
  store: CacheStore,
  cache: DashMap<String, Vec<u8>>,
}

impl PluginCacheManager {
  pub fn new(cache_dir: &str, namespace: &str, mode: Mode) -> Self {
    let store = CacheStore::new(cache_dir, namespace, mode, "plugin");
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

    let cache = self
      .store
      .read_cache(&self.normalize_plugin_name(&plugin_name));

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

  pub fn write_cache_to_disk(&self) {
    let cache = self
      .cache
      .iter()
      .map(|entry| {
        (
          CacheStoreKey {
            name: entry.key().clone(),
            key: sha256(entry.value(), 32),
          },
          entry.value().clone(),
        )
      })
      .collect::<HashMap<_, _>>();

    if !cache.is_empty() {
      self.store.write_cache(cache);
    }
  }
}
