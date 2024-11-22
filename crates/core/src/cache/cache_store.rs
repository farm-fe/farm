//! Cache store of the persistent cache, responsible for reading and writing the cache from the disk.
use dashmap::{mapref::multiple::RefMulti, DashMap};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

use crate::config::Mode;

const FARM_CACHE_VERSION: &str = "0.6.1";
const FARM_CACHE_MANIFEST_FILE: &str = "farm-cache.json";

// TODO make CacheStore a trait and implement DiskCacheStore or RemoteCacheStore or more.
#[derive(Default)]
pub struct CacheStore {
  cache_dir: PathBuf,
  /// name -> cache key manifest of this store.
  /// it will be stored in a separate file
  manifest: DashMap<String, String>,
}

impl CacheStore {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode, name: &str) -> Self {
    let mut cache_dir = Path::new(cache_dir_str).to_path_buf();
    let last = cache_dir
      .file_name()
      .unwrap_or_default()
      .to_string_lossy()
      .to_string();
    cache_dir.pop();

    cache_dir.push(format!("{FARM_CACHE_VERSION}-{last}"));

    if !namespace.is_empty() {
      cache_dir.push(namespace);
    }

    if matches!(mode, Mode::Development) {
      cache_dir.push("development");
    } else {
      cache_dir.push("production");
    }

    if !name.is_empty() {
      cache_dir.push(name);
    }

    let manifest_file_path = cache_dir.join(FARM_CACHE_MANIFEST_FILE);

    let manifest = if manifest_file_path.exists() && manifest_file_path.is_file() {
      let content = std::fs::read_to_string(manifest_file_path).unwrap();
      let map = serde_json::from_str::<HashMap<String, String>>(&content).unwrap();
      let dashmap = DashMap::new();

      for (k, v) in map {
        dashmap.insert(k, v);
      }

      dashmap
    } else {
      DashMap::new()
    };

    Self {
      cache_dir,
      manifest,
    }
  }

  pub fn has_cache(&self, name: &str) -> bool {
    self.manifest.contains_key(name)
  }

  pub fn get_store_keys(&self) -> Vec<RefMulti<String, String>> {
    self.manifest.iter().collect()
  }

  /// return true if the cache changed or it's a cache item
  pub fn is_cache_changed(&self, store_key: &CacheStoreKey) -> bool {
    if let Some(guard) = self.manifest.get(&store_key.name) {
      if guard.value() == &store_key.key {
        // the cache is not changed
        return false;
      }
    }

    true
  }

  pub fn write_single_cache(
    &self,
    store_key: CacheStoreKey,
    bytes: Vec<u8>,
  ) -> std::io::Result<()> {
    let cache_file_dir = &self.cache_dir;

    if !cache_file_dir.exists() {
      std::fs::create_dir_all(cache_file_dir).unwrap();
    }

    if self.is_cache_changed(&store_key) {
      if let Some(guard) = self.manifest.get(&store_key.name) {
        let cache_file_path = cache_file_dir.join(guard.value());

        if cache_file_path.exists() && cache_file_path.is_file() {
          std::fs::remove_file(cache_file_path).ok();
        }
      }

      self
        .manifest
        .insert(store_key.name.clone(), store_key.key.clone());
      let cache_file_path = cache_file_dir.join(store_key.key);
      std::fs::write(&cache_file_path, bytes).map_err(|e| {
        std::io::Error::new(
          e.kind(),
          format!(
            "Failed to write cache file: {} {:?}, error: {:?}",
            store_key.name, cache_file_path, e
          ),
        )
      })?;
    }

    Ok(())
  }

  pub fn write_manifest(&self) {
    let manifest = self.manifest.clone().into_iter().collect::<HashMap<_, _>>();

    if !self.cache_dir.exists() {
      std::fs::create_dir_all(&self.cache_dir).unwrap();
    }

    let manifest_file_path = &self.cache_dir.join(FARM_CACHE_MANIFEST_FILE);
    std::fs::write(
      manifest_file_path,
      serde_json::to_string(&manifest).unwrap(),
    )
    .unwrap();
  }

  /// Write the cache map to the disk.
  pub fn write_cache(&self, cache_map: HashMap<CacheStoreKey, Vec<u8>>) {
    let cache_file_dir = &self.cache_dir;
    if !cache_file_dir.exists() {
      std::fs::create_dir_all(cache_file_dir).unwrap();
    }

    cache_map
      .into_par_iter()
      .try_for_each(|(store_key, bytes)| {
        self.write_single_cache(store_key, bytes)?;
        Ok::<(), std::io::Error>(())
      })
      .unwrap();

    self.write_manifest();
  }

  pub fn read_cache(&self, name: &str) -> Option<Vec<u8>> {
    if !self.manifest.contains_key(name) {
      return None;
    }

    let cache_key = self.manifest.get(name).unwrap().value().clone();
    let cache_file = self.cache_dir.join(cache_key);

    if cache_file.exists() && cache_file.is_file() {
      return Some(std::fs::read(cache_file).unwrap());
    }

    None
  }
}

/// Cache key of the store, it's a pair of (name, cache_key), a name should only be related to one cache key.
/// Previous cache will be cleared if the related cache key changed for a name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheStoreKey {
  pub name: String,
  pub key: String,
}
