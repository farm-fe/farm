use std::{
  path::{Path, PathBuf},
  sync::{Arc, RwLock},
};

use dashmap::DashMap;
use farmfe_utils::hash::sha256;
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

use super::{
  constant::{CacheStoreFactory, CacheStoreTrait, FARM_CACHE_MANIFEST_FILE, FARM_CACHE_VERSION},
  error::CacheError,
  namespace::NamespaceStore,
  CacheStoreKey,
};
use crate::{config::Mode, deserialize, serialize, HashMap, HashSet};

// #[cache_item]
type CombineCacheData = HashMap<CacheStoreKey, Vec<u8>>;
// TODO make CacheStore a trait and implement DiskCacheStore or RemoteCacheStore or more.
#[derive(Default)]
pub struct CacheStore {
  cache_dir: PathBuf,
  /// name -> cache key manifest of this store.
  /// it will be stored in a separate file
  manifest: DashMap<String, String>,
  data: DashMap<String, Vec<u8>>,
  lock: RwLock<HashSet<PathBuf>>,
}

impl CacheStore {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
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

    let manifest_file_path = cache_dir.join(FARM_CACHE_MANIFEST_FILE);

    let manifest = if manifest_file_path.exists() && manifest_file_path.is_file() {
      let content = std::fs::read_to_string(manifest_file_path).unwrap();
      serde_json::from_str::<HashMap<String, String>>(&content)
        .unwrap()
        .into_iter()
        .collect()
    } else {
      DashMap::new()
    };

    Self {
      cache_dir,
      manifest,
      ..Default::default()
    }
  }

  fn hash_index_from_name(&self, name: &str) -> u8 {
    sha256(name.as_bytes(), 32)
      .chars()
      .fold(0u8, |r, i| (r + i as u8) % 16)
  }

  fn real_cache_path(&self, name: &str) -> PathBuf {
    let index = self.hash_index_from_name(name);

    let cache_file_dir = &self.cache_dir;
    cache_file_dir.join(format!("cache-{}", index))
  }

  fn restore_cache(&self, name: &str) {
    let cache_path = self.real_cache_path(name);

    if !(cache_path.exists() && cache_path.is_file()) {
      return;
    }

    if let Ok(map) = self.lock.read() {
      if map.contains(&cache_path) {
        return;
      }
    }

    if let Ok(mut map) = self.lock.write() {
      let data = std::fs::read(cache_path.clone()).unwrap();

      let value = deserialize!(&data, CombineCacheData);

      for (key, value) in value {
        if self.data.contains_key(&key.key) {
          continue;
        }
        self.insert_cache(&key.name, &key.key, value);
      }

      map.insert(cache_path.clone());
    }
  }

  fn try_read_content(&self, name: &str) -> Option<Vec<u8>> {
    let manifest_item = self.manifest.get(name)?;

    if !self.data.contains_key(manifest_item.value()) {
      drop(manifest_item);
      self.restore_cache(name);
    }

    let manifest_item = self.manifest.get(name)?;

    self
      .data
      .get(manifest_item.value())
      .map(|v| v.value().clone())
  }

  fn write_content_to_disk(&self, cache_dir_str: PathBuf, data: Vec<u8>) {
    if let Some(parent) = cache_dir_str.parent() {
      if !parent.exists() {
        std::fs::create_dir_all(parent).unwrap();
      }
    }

    std::fs::write(cache_dir_str, data).unwrap();
  }

  fn write_disk(&self) {
    let cache_dir = &self.cache_dir;

    if !cache_dir.exists() {
      std::fs::create_dir_all(cache_dir).unwrap();
    }

    self
      .manifest
      .iter()
      .par_bridge()
      .fold(
        HashMap::<PathBuf, CombineCacheData>::default,
        |mut combine_data, item| {
          let name = item.key();
          let key = item.value();

          // reenerate the cache if not exists
          let Some(value) = self.try_read_content(name) else {
            return combine_data;
          };

          let cache_file_path = self.real_cache_path(name);

          combine_data
            .entry(cache_file_path)
            .or_default()
            .insert((name.to_string(), key.to_string()).into(), value);

          combine_data
        },
      )
      .reduce(HashMap::default, |mut a, b| {
        for (store_key, map) in b {
          a.entry(store_key).or_default().extend(map);
        }

        a
      })
      .into_par_iter()
      .for_each(|(cache_file_path, data)| {
        let data = serialize!(&data);
        self.write_content_to_disk(cache_file_path, data);
      });
  }

  fn _remove_cache(&self, name: &str) {
    let Some((_, cache_key)) = self.manifest.remove(name) else {
      return;
    };

    self.data.remove(&cache_key);
  }

  fn insert_cache(&self, name: &str, key: &str, data: Vec<u8>) {
    self.manifest.insert(name.to_string(), key.to_string());
    self.data.insert(key.to_string(), data);
  }

  fn write_manifest(&self) {
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
}

impl CacheStoreTrait for CacheStore {
  fn has_cache(&self, name: &str) -> bool {
    self.manifest.contains_key(name)
  }

  /// return true if the cache changed or it's a cache item
  fn is_cache_changed(&self, store_key: &CacheStoreKey) -> bool {
    if let Some(guard) = self.manifest.get(&store_key.name) {
      if guard.value() == &store_key.key {
        // the cache is not changed
        return false;
      }
    }

    true
  }

  fn write_single_cache(&self, store_key: CacheStoreKey, bytes: Vec<u8>) -> Result<(), CacheError> {
    if self.is_cache_changed(&store_key) {
      self.insert_cache(&store_key.name, &store_key.key, bytes);
    }

    Ok(())
  }

  /// Write the cache map to the disk.
  fn write_cache(&self, cache_map: HashMap<CacheStoreKey, Vec<u8>>) {
    cache_map.into_par_iter().for_each(|(store_key, bytes)| {
      self.write_single_cache(store_key, bytes).unwrap();
    });
  }

  fn read_cache(&self, name: &str) -> Option<Vec<u8>> {
    self.try_read_content(name)
  }

  fn remove_cache(&self, name: &str) {
    self._remove_cache(name);
  }

  fn shutdown(&self) {
    self.write_disk();
    self.write_manifest();
  }
}

pub struct DiskCacheFactory {
  store: Arc<Box<dyn CacheStoreTrait>>,
}

impl DiskCacheFactory {
  pub fn new(cache_dir: &str, namespace: &str, mode: Mode) -> Self {
    let store: Arc<Box<dyn CacheStoreTrait>> =
      Arc::new(Box::new(CacheStore::new(cache_dir, namespace, mode)));

    Self { store }
  }
}

impl CacheStoreFactory for DiskCacheFactory {
  fn create_cache_store(&self, name: &str) -> Box<dyn CacheStoreTrait> {
    Box::new(NamespaceStore::new(self.store.clone(), name.to_string()))
  }
}
