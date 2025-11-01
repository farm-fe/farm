use std::{
  fmt::Debug,
  hash::Hash,
  path::{Path, PathBuf},
  sync::Arc,
};

use dashmap::{DashMap, DashSet};
use farmfe_utils::hash::sha256;
use itertools::Itertools;
use parking_lot::{lock_api::ArcMutexGuard, Mutex};
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

use super::{
  constant::{CacheStoreFactory, CacheStoreTrait, FARM_CACHE_MANIFEST_FILE, FARM_CACHE_VERSION},
  error::CacheError,
  namespace::NamespaceStore,
  CacheStoreKey,
};
use crate::{
  cache::store::constant::CacheStoreItemRef, config::Mode, deserialize, serialize, HashMap,
};

#[derive(Debug, Default)]
pub struct ResourceLock<T: Debug + Eq + Hash + PartialEq> {
  locks: DashMap<T, Arc<Mutex<bool>>>,
}

impl<T: Eq + Hash + PartialEq + Clone + Debug> ResourceLock<T> {
  pub fn lock(&self, key: T) -> ArcMutexGuard<parking_lot::RawMutex, bool> {
    self
      .locks
      .entry(key.clone())
      .or_insert_with(|| Arc::new(Mutex::new(true)))
      .lock_arc()
  }
}

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
  restored: DashSet<u8>,
  resource_lock: ResourceLock<String>,
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

  fn join_hash(&self, hash: u8) -> PathBuf {
    self.cache_dir.join(format!("cache-{hash}"))
  }

  fn real_cache_path(&self, name: &str) -> (u8, PathBuf) {
    let index = self.hash_index_from_name(name);

    (index, self.join_hash(index))
  }

  fn restore_cache_by_hash(&self, hash: u8, cache_path: PathBuf) {
    if self.restored.contains(&hash) {
      return;
    }

    let lock = self.resource_lock.lock(hash.to_string());

    if self.restored.contains(&hash) {
      return;
    }

    if !cache_path.metadata().ok().is_some_and(|v| v.is_file()) {
      self.restored.insert(hash);
    }

    let data = std::fs::read(cache_path.clone()).unwrap();

    let value = deserialize!(&data, CombineCacheData);

    value.into_par_iter().for_each(|(item_key, value)| {
      if !self.manifest.contains_key(&item_key.name) || self.data.contains_key(&item_key.key) {
        return;
      }
      self.data.insert(item_key.key, value);
    });

    // should drop when restore done, because same hash should wait restore done
    self.restored.insert(hash);

    drop(lock);
  }

  fn restore_cache(&self, name: &str) {
    let (hash, cache_path) = self.real_cache_path(name);
    self.restore_cache_by_hash(hash, cache_path);
  }

  fn try_read_content_ref(&self, name: &str) -> Option<CacheStoreItemRef<'_>> {
    let key = self.manifest.get(name)?;
    let has_data = self.data.contains_key(key.value());

    drop(key);
    if !has_data {
      self.restore_cache(name);
    }

    let manifest_item = self.manifest.get(name)?;

    self.data.get(manifest_item.value()).map(|v| v.map(|v| v))
  }

  fn try_read_content(&self, name: &str) -> Option<Vec<u8>> {
    self.try_read_content_ref(name).map(|v| v.to_vec())
  }

  fn write_content_to_disk(&self, cache_dir_str: PathBuf, data: Vec<u8>) {
    std::fs::write(cache_dir_str, data).unwrap();
  }

  fn write_disk(&self) {
    let cache_dir = &self.cache_dir;

    if !cache_dir.exists() {
      std::fs::create_dir_all(cache_dir).unwrap();
    }

    let manifest_keys = self
      .manifest
      .iter()
      .map(|v| v.key().clone())
      .collect::<Vec<_>>();

    manifest_keys
      .into_iter()
      .filter(|v| !self.data.contains_key(v))
      .map(|v| self.real_cache_path(&v))
      .unique()
      .par_bridge()
      .for_each(|(hash, real_cache_path)| {
        self.restore_cache_by_hash(hash, real_cache_path);
      });

    self
      .manifest
      .iter()
      .par_bridge()
      .fold(
        HashMap::<u8, CombineCacheData>::default,
        |mut combine_data, item| {
          let name = item.key();
          let key = item.value();

          let Some(value) = self.try_read_content(name) else {
            return combine_data;
          };

          let (hash, _) = self.real_cache_path(name);

          combine_data
            .entry(hash)
            .or_default()
            .insert((name, key).into(), value);

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
        self.write_content_to_disk(self.join_hash(cache_file_path), data);
      });
  }

  fn _remove_cache(&self, name: &str, should_restore: bool) -> Option<Vec<u8>> {
    let lock = self.resource_lock.lock(name.to_string());
    let item = self.manifest.get(name)?;
    let key = item.value().clone();
    drop(item);

    if !self.data.contains_key(&key) {
      if !should_restore {
        return None;
      }

      self.restore_cache(name);
    }

    let (_, item) = self.manifest.remove(name)?;
    let v = self.data.remove(&item).map(|(_, v)| v);

    drop(lock);

    v
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
    !matches!(self.manifest.get(&store_key.name), Some(guard) if guard.value() == &store_key.key)
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

  fn read_cache_ref(&self, name: &str) -> Option<CacheStoreItemRef<'_>> {
    self.try_read_content_ref(name)
  }

  fn remove_cache(&self, name: &str) -> Option<Vec<u8>> {
    self._remove_cache(name, true)
  }

  fn remove_cache_only(&self, name: &str) {
    let _ = self._remove_cache(name, false);
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

#[cfg(feature = "profile")]
mod profile {

  pub struct TimeCacheFactory {
    store: Arc<Box<dyn CacheStoreTrait>>,
  }

  macro_rules! time {
    ($f_name:literal, $name:expr, $code:expr) => {{
      let start = std::time::Instant::now();
      let result = { $code };
      let duration = start.elapsed();
      println!(
        "Task {} {} Time taken: {:?}",
        $f_name,
        $name.to_string(),
        duration
      );
      result
    }};
  }

  impl TimeCacheFactory {
    pub fn new(store: Arc<Box<dyn CacheStoreTrait>>) -> Self {
      Self { store }
    }
  }

  impl CacheStoreTrait for TimeCacheFactory {
    fn has_cache(&self, name: &str) -> bool {
      time!("has_cache", name, self.store.has_cache(name))
    }

    fn is_cache_changed(&self, store_key: &CacheStoreKey) -> bool {
      let v1 = store_key.name.clone();
      time!(
        "is_cache_changed",
        v1,
        self.store.is_cache_changed(store_key)
      )
    }

    fn write_single_cache(
      &self,
      store_key: CacheStoreKey,
      bytes: Vec<u8>,
    ) -> Result<(), CacheError> {
      let v1 = store_key.name.clone();
      time!(
        "write_single_cache",
        v1,
        self.store.write_single_cache(store_key, bytes)
      )
    }

    fn read_cache(&self, name: &str) -> Option<Vec<u8>> {
      time!("read_cache", name, self.store.read_cache(name))
    }

    fn read_cache_ref(&self, name: &str) -> Option<CacheStoreItemRef<'_>> {
      time!("read_cache_ref", name, self.store.read_cache_ref(name))
    }

    fn remove_cache(&self, name: &str) -> Option<Vec<u8>> {
      time!("remove_cache", name, self.store.remove_cache(name))
    }
  }
}

impl CacheStoreFactory for DiskCacheFactory {
  fn create_cache_store(&self, name: &str) -> Box<dyn CacheStoreTrait> {
    #[cfg(feature = "profile")]
    {
      return Box::new(NamespaceStore::new(
        Box::new(profile::TimeCacheFactory::new(self.store.clone())),
        name.to_string(),
      ));
    }
    Box::new(NamespaceStore::new(self.store.clone(), name.to_string()))
  }
}
