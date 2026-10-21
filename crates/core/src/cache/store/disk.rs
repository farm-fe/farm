use std::{
  fmt::Debug,
  hash::Hash,
  io::Write,
  path::{Path, PathBuf},
  sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
  },
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
use crate::{cache::store::constant::CacheStoreItemRef, config::Mode, serialize, HashMap};

#[derive(Debug, Default)]
pub struct ResourceLock<T: Debug + Eq + Hash + PartialEq> {
  locks: DashMap<T, Arc<Mutex<bool>>>,
}

impl<T: Eq + Hash + PartialEq + Clone + Debug> ResourceLock<T> {
  pub fn lock(&self, key: T) -> ArcMutexGuard<parking_lot::RawMutex, bool> {
    // Release the map shard before waiting for the resource mutex. A holder
    // may need another resource from that shard while completing its read.
    let mutex = Arc::clone(
      self
        .locks
        .entry(key)
        .or_insert_with(|| Arc::new(Mutex::new(true)))
        .value(),
    );
    mutex.lock_arc()
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
  restored: DashSet<u128>,
  resource_lock: ResourceLock<String>,
  dirty_generation: DashMap<u128, u64>,
  flushed_generation: DashMap<u128, u64>,
  flush_lock: Mutex<()>,
}

static CACHE_WRITE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn write_atomic(path: &Path, data: &[u8]) -> std::io::Result<()> {
  // Keep the previous published file intact if the process exits during a
  // write. The temporary file is on the same filesystem as its destination.
  let (temporary, mut file) = loop {
    let sequence = CACHE_WRITE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temporary = path.with_extension(format!("{}-{sequence}.tmp", std::process::id()));
    match std::fs::OpenOptions::new()
      .write(true)
      .create_new(true)
      .open(&temporary)
    {
      Ok(file) => break (temporary, file),
      Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
      Err(error) => return Err(error),
    }
  };
  let result = (|| {
    file.write_all(data)?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(&temporary, path)
  })();
  if result.is_err() {
    let _ = std::fs::remove_file(&temporary);
  }
  result
}

impl CacheStore {
  pub(crate) fn directory_for(cache_dir_str: &str, namespace: &str, mode: Mode) -> PathBuf {
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

    cache_dir
  }

  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
    let cache_dir = Self::directory_for(cache_dir_str, namespace, mode);
    let manifest_file_path = cache_dir.join(FARM_CACHE_MANIFEST_FILE);

    let manifest = if manifest_file_path.exists() && manifest_file_path.is_file() {
      std::fs::read_to_string(manifest_file_path)
        .ok()
        .and_then(|content| serde_json::from_str::<HashMap<String, String>>(&content).ok())
        .unwrap_or_default()
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

  fn hash_index_from_name(&self, name: &str) -> u128 {
    let bucket = sha256(name.as_bytes(), 32)
      .chars()
      .fold(0u8, |r, i| (r + i as u8) % 16);
    // NamespaceStore prefixes each name with its logical namespace. Keep
    // resource/plugin updates out of the much larger module cache shards.
    // A hexadecimal digest keeps arbitrary namespace text out of filenames.
    let namespace = name.split_once('|').map_or(0, |(namespace, _)| {
      u128::from_str_radix(&sha256(namespace.as_bytes(), 28), 16)
        .expect("SHA-256 prefixes contain only hexadecimal digits")
    });
    (namespace << 8) | u128::from(bucket)
  }

  fn join_hash(&self, hash: u128) -> PathBuf {
    self.cache_dir.join(format!("cache-{hash}"))
  }

  fn real_cache_path(&self, name: &str) -> (u128, PathBuf) {
    let index = self.hash_index_from_name(name);

    (index, self.join_hash(index))
  }

  fn invalidate_missing_entries(&self, hash: u128) {
    let invalid_names = self
      .manifest
      .iter()
      .filter(|entry| {
        self.hash_index_from_name(entry.key()) == hash && !self.data.contains_key(entry.value())
      })
      .map(|entry| (entry.key().clone(), entry.value().clone()))
      .collect::<Vec<_>>();
    for (name, key) in invalid_names {
      self.remove_observed_missing_entry(hash, &name, &key);
    }
  }

  fn remove_observed_missing_entry(&self, hash: u128, name: &str, observed_key: &str) {
    if self
      .manifest
      .remove_if(name, |_, key| {
        key == observed_key && !self.data.contains_key(key)
      })
      .is_some()
    {
      self.mark_shard_changed(hash);
    }
  }

  fn restore_cache_by_hash(&self, hash: u128, cache_path: PathBuf) {
    let lock = self.resource_lock.lock(format!("shard:{hash}"));

    if self.restored.contains(&hash) {
      return;
    }

    let value = std::fs::read(&cache_path)
      .ok()
      .and_then(|data| rkyv::from_bytes::<CombineCacheData, rkyv::rancor::Error>(&data).ok());
    let Some(value) = value else {
      // Cache writes may be interrupted or damaged. Never dereference an
      // unchecked archive from disk, and allow the same inputs to rebuild it.
      self.invalidate_missing_entries(hash);
      self.restored.insert(hash);
      return;
    };

    value.into_par_iter().for_each(|(item_key, value)| {
      if !self.manifest.contains_key(&item_key.name) || self.data.contains_key(&item_key.key) {
        return;
      }
      self.data.insert(item_key.key, value);
    });

    // A stop between publishing a shard and its manifest can leave a valid
    // archive whose keys no longer match the manifest. Rebuild those misses.
    self.invalidate_missing_entries(hash);

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

  fn write_content_to_disk(&self, cache_dir_str: PathBuf, data: Vec<u8>) -> bool {
    if let Err(error) = write_atomic(&cache_dir_str, &data) {
      eprintln!(
        "Could not write Farm cache {}: {error}",
        cache_dir_str.display()
      );
      return false;
    }
    true
  }

  fn mark_shard_changed(&self, hash: u128) {
    let mut generation = self.dirty_generation.entry(hash).or_default();
    *generation = generation.wrapping_add(1);
  }

  fn pending_shards(&self) -> HashMap<u128, u64> {
    self
      .dirty_generation
      .iter()
      .filter_map(|entry| {
        let hash = *entry.key();
        let generation = *entry.value();
        (generation != self.flushed_generation.get(&hash).map_or(0, |value| *value))
          .then_some((hash, generation))
      })
      .collect()
  }

  fn publish_shard(&self, hash: u128, generation: u64, data: &CombineCacheData) {
    if self.write_content_to_disk(self.join_hash(hash), serialize!(data)) {
      // A mutation after this snapshot increments dirty_generation again.
      // Acknowledge only this published generation, including on retry.
      self.flushed_generation.insert(hash, generation);
    }
  }

  fn write_disk(&self) {
    let pending = self.pending_shards();
    if pending.is_empty() {
      return;
    }
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
      .filter(|name| {
        pending.contains_key(&self.hash_index_from_name(name))
          && self
            .manifest
            .get(name)
            .is_some_and(|entry| !self.data.contains_key(entry.value()))
      })
      .map(|v| self.real_cache_path(&v))
      .unique()
      .par_bridge()
      .for_each(|(hash, real_cache_path)| {
        self.restore_cache_by_hash(hash, real_cache_path);
      });

    let combined = self
      .manifest
      .iter()
      .par_bridge()
      .filter(|item| pending.contains_key(&self.hash_index_from_name(item.key())))
      .fold(
        HashMap::<u128, CombineCacheData>::default,
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
      });
    pending.into_par_iter().for_each(|(hash, generation)| {
      let empty = CombineCacheData::default();
      // Publish an empty archive too when the last item was removed.
      self.publish_shard(hash, generation, combined.get(&hash).unwrap_or(&empty));
    });
  }

  fn _remove_cache(&self, name: &str, should_restore: bool) -> Option<Vec<u8>> {
    let lock = self.resource_lock.lock(format!("entry:{name}"));
    let item = self.manifest.get(name)?;
    let key = item.value().clone();
    drop(item);

    if should_restore && !self.data.contains_key(&key) {
      self.restore_cache(name);
    }

    let (_, item) = self.manifest.remove(name)?;
    let v = self.data.remove(&item).map(|(_, v)| v);
    self.mark_shard_changed(self.hash_index_from_name(name));

    drop(lock);

    v
  }

  fn insert_cache(&self, name: &str, key: &str, data: Vec<u8>) {
    let _entry = self.resource_lock.lock(format!("entry:{name}"));
    // Readers must never observe a newly published key before its data.
    self.data.insert(key.to_string(), data);
    self.manifest.insert(name.to_string(), key.to_string());
    self.mark_shard_changed(self.hash_index_from_name(name));
  }

  fn write_manifest(&self) {
    let manifest = self.manifest.clone().into_iter().collect::<HashMap<_, _>>();

    if !self.cache_dir.exists() {
      std::fs::create_dir_all(&self.cache_dir).unwrap();
    }

    let manifest_file_path = &self.cache_dir.join(FARM_CACHE_MANIFEST_FILE);
    if let Err(error) = write_atomic(
      manifest_file_path,
      serde_json::to_string(&manifest).unwrap().as_bytes(),
    ) {
      eprintln!("Could not write Farm cache manifest: {error}");
    }
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
    let _flush = self.flush_lock.lock();
    self.write_disk();
    self.write_manifest();
  }
}

pub struct DiskCacheFactory {
  store: Arc<Box<dyn CacheStoreTrait>>,
}

#[cfg(test)]
mod cache_format_tests {
  use super::*;

  fn shard_test_store(tag: &str) -> (PathBuf, CacheStore) {
    let input = std::env::temp_dir()
      .join(format!(
        "farm-cache-shard-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .unwrap()
          .as_nanos()
      ))
      .join("case");
    let store = CacheStore::new(input.to_str().unwrap(), "", Mode::Development);
    (input, store)
  }

  #[test]
  fn discarding_unloaded_payload_preserves_siblings_without_restoring_shard() {
    let (input, store) = shard_test_store("discard-unloaded");
    let name = "resource|changed";
    let sibling = (0..1024)
      .map(|index| format!("resource|sibling-{index}"))
      .find(|candidate| store.hash_index_from_name(candidate) == store.hash_index_from_name(name))
      .unwrap();
    put(&store, name, "changed-v1", b"obsolete payload");
    put(&store, &sibling, "sibling-v1", b"keep payload");
    store.shutdown();

    let reopened = CacheStore::new(input.to_str().unwrap(), "", Mode::Development);
    assert!(reopened.data.is_empty());
    reopened.remove_cache_only(name);
    assert!(
      !reopened.has_cache(name),
      "Discard the manifest even when the payload is still on disk"
    );
    assert!(
      reopened.restored.is_empty(),
      "Invalidation must not read an unused resource shard"
    );
    assert!(
      reopened.data.is_empty(),
      "Invalidation must not materialize unrelated payloads"
    );
    reopened.shutdown();

    let restored = CacheStore::new(input.to_str().unwrap(), "", Mode::Development);
    assert!(
      !restored.has_cache(name),
      "Discarded output must not resurrect on restart"
    );
    assert_eq!(
      restored.read_cache(&sibling),
      Some(b"keep payload".to_vec())
    );
  }

  #[test]
  fn stale_missing_entry_observation_preserves_later_value_and_repair() {
    let (input, store) = shard_test_store("stale-observation");
    let name = "module";
    let hash = store.hash_index_from_name(name);
    store.manifest.insert(name.into(), "missing-v1".into());
    // A restore observed v1 missing; a writer publishes v2 before cleanup.
    put(&store, name, "v2", b"new value");
    store.remove_observed_missing_entry(hash, name, "missing-v1");
    assert_eq!(store.read_cache(name), Some(b"new value".to_vec()));
    // The observed key itself may also have been repaired since the scan.
    store.remove_observed_missing_entry(hash, name, "v2");
    assert_eq!(store.read_cache(name), Some(b"new value".to_vec()));
    store.shutdown();
    let restored = CacheStore::new(input.to_str().unwrap(), "", Mode::Development);
    assert_eq!(restored.read_cache(name), Some(b"new value".to_vec()));
  }

  fn put(store: &CacheStore, name: &str, version: &str, bytes: &[u8]) {
    store
      .write_single_cache(
        CacheStoreKey {
          name: name.into(),
          key: version.into(),
        },
        bytes.to_vec(),
      )
      .unwrap();
  }

  #[test]
  fn namespace_update_preserves_unchanged_large_payload_file() {
    let (input, store) = shard_test_store("namespace-isolation");
    let large_name = "mutable-module|large-module";
    // Match the old 16-bucket index without depending on the new namespace key.
    let old_bucket = |name: &str| {
      sha256(name.as_bytes(), 32)
        .bytes()
        .fold(0u8, |sum, byte| (sum + byte) % 16)
    };
    let small_name = (0..1024)
      .map(|i| format!("resource|small-{i}"))
      .find(|name| old_bucket(name) == old_bucket(large_name))
      .unwrap();
    let large = vec![42u8; 256 * 1024];
    put(&store, large_name, "large-v1", &large);
    put(&store, &small_name, "small-v1", b"a");
    store.shutdown();
    let large_file = store.real_cache_path(large_name).1;
    let original_bytes = std::fs::read(&large_file).unwrap();
    let sentinel = std::time::UNIX_EPOCH + std::time::Duration::from_secs(42);
    std::fs::OpenOptions::new()
      .write(true)
      .open(&large_file)
      .unwrap()
      .set_times(std::fs::FileTimes::new().set_modified(sentinel))
      .unwrap();
    put(&store, &small_name, "small-v2", b"b");
    store.shutdown();
    assert_eq!(std::fs::read(&large_file).unwrap(), original_bytes);
    assert_eq!(
      std::fs::metadata(&large_file).unwrap().modified().unwrap(),
      sentinel
    );
    let reopened = CacheStore::new(input.to_str().unwrap(), "", Mode::Development);
    assert_eq!(reopened.read_cache(large_name), Some(large));
    assert_eq!(reopened.read_cache(&small_name), Some(b"b".to_vec()));
  }

  #[test]
  fn ignores_previous_shared_shards_without_deleting_their_manifest() {
    let (input, _) = shard_test_store("namespace-upgrade");
    let legacy = input.parent().unwrap().join("0.9.3-case/development");
    std::fs::create_dir_all(&legacy).unwrap();
    let manifest = legacy.join(FARM_CACHE_MANIFEST_FILE);
    let original = br#"{"mutable-module|large-module":"old-shared-key"}"#;
    std::fs::write(&manifest, original).unwrap();
    let current = CacheStore::new(input.to_str().unwrap(), "", Mode::Development);
    assert!(!current.has_cache("mutable-module|large-module"));
    assert_ne!(current.cache_dir, legacy);
    assert_eq!(std::fs::read(manifest).unwrap(), original);
  }

  #[test]
  fn clean_reopened_store_does_not_replace_shards() {
    let (input, store) = shard_test_store("clean");
    put(&store, "module", "v1", b"original");
    store.shutdown();
    let file = store.real_cache_path("module").1;
    let sentinel = std::time::UNIX_EPOCH + std::time::Duration::from_secs(42);
    std::fs::OpenOptions::new()
      .write(true)
      .open(&file)
      .unwrap()
      .set_times(std::fs::FileTimes::new().set_modified(sentinel))
      .unwrap();
    let reopened = CacheStore::new(input.to_str().unwrap(), "", Mode::Development);
    assert_eq!(reopened.read_cache("module"), Some(b"original".to_vec()));
    put(&reopened, "module", "v1", b"original");
    reopened.shutdown();
    assert_eq!(
      std::fs::metadata(file).unwrap().modified().unwrap(),
      sentinel
    );
  }

  #[test]
  fn changed_shard_restores_siblings_and_persists_deletions() {
    let (input, store) = shard_test_store("siblings");
    let first = "first";
    let sibling = (0..1000)
      .map(|index| format!("sibling-{index}"))
      .find(|name| store.hash_index_from_name(name) == store.hash_index_from_name(first))
      .unwrap();
    put(&store, first, "first-v1", b"old");
    put(&store, &sibling, "sibling-v1", b"keep");
    store.shutdown();
    let reopened = CacheStore::new(input.to_str().unwrap(), "", Mode::Development);
    put(&reopened, first, "first-v2", b"updated");
    reopened.shutdown();
    let restored = CacheStore::new(input.to_str().unwrap(), "", Mode::Development);
    assert_eq!(restored.read_cache(first), Some(b"updated".to_vec()));
    assert_eq!(restored.read_cache(&sibling), Some(b"keep".to_vec()));
    assert_eq!(restored.remove_cache(&sibling), Some(b"keep".to_vec()));
    restored.shutdown();
    let bytes = std::fs::read(restored.real_cache_path(first).1).unwrap();
    let archive = rkyv::from_bytes::<CombineCacheData, rkyv::rancor::Error>(&bytes).unwrap();
    assert_eq!(archive.len(), 1);
    assert!(archive.keys().all(|key| key.name == first));
    assert_eq!(restored.remove_cache(first), Some(b"updated".to_vec()));
    restored.shutdown();
    let bytes = std::fs::read(restored.real_cache_path(first).1).unwrap();
    assert!(
      rkyv::from_bytes::<CombineCacheData, rkyv::rancor::Error>(&bytes)
        .unwrap()
        .is_empty()
    );
    let empty = CacheStore::new(input.to_str().unwrap(), "", Mode::Development);
    assert!(!empty.has_cache(first));
    assert!(!empty.has_cache(&sibling));
  }

  #[test]
  fn failed_shard_publication_remains_pending_until_retry() {
    let (input, store) = shard_test_store("retry");
    put(&store, "module", "v1", b"retry me");
    let (hash, file) = store.real_cache_path("module");
    std::fs::create_dir_all(&file).unwrap(); // A directory prevents atomic file publication.
    store.shutdown();
    assert!(store.pending_shards().contains_key(&hash));
    std::fs::remove_dir(&file).unwrap();
    store.shutdown();
    assert!(store.pending_shards().is_empty());
    let restored = CacheStore::new(input.to_str().unwrap(), "", Mode::Development);
    assert_eq!(restored.read_cache("module"), Some(b"retry me".to_vec()));
  }

  #[test]
  fn publishing_older_snapshot_does_not_acknowledge_later_mutation() {
    let (input, store) = shard_test_store("generation");
    put(&store, "module", "v1", b"old");
    let hash = store.hash_index_from_name("module");
    let generation = store.pending_shards()[&hash];
    let snapshot = HashMap::from_iter([(
      CacheStoreKey {
        name: "module".into(),
        key: "v1".into(),
      },
      b"old".to_vec(),
    )]);
    put(&store, "module", "v2", b"new");
    std::fs::create_dir_all(&store.cache_dir).unwrap();
    store.publish_shard(hash, generation, &snapshot);
    assert!(store.pending_shards()[&hash] > generation);
    store.shutdown();
    assert!(store.pending_shards().is_empty());
    let restored = CacheStore::new(input.to_str().unwrap(), "", Mode::Development);
    assert_eq!(restored.read_cache("module"), Some(b"new".to_vec()));
  }

  #[test]
  fn single_cache_update_preserves_unaffected_shard() {
    use std::time::{Duration, SystemTime};

    let root = std::env::temp_dir().join(format!(
      "farm-cache-shard-update-{}-{}",
      std::process::id(),
      SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    let input = root.join("case");
    let store = CacheStore::new(input.to_str().unwrap(), "", Mode::Development);
    let changed_name = "new-customer-module";
    let stable_name = (0..100)
      .map(|index| format!("large-existing-component-{index}"))
      .find(|name| store.hash_index_from_name(name) != store.hash_index_from_name(changed_name))
      .unwrap();
    store.write_cache(HashMap::from_iter([
      (
        CacheStoreKey {
          name: changed_name.to_string(),
          key: "customer-v1".into(),
        },
        b"old customer".to_vec(),
      ),
      (
        CacheStoreKey {
          name: stable_name.clone(),
          key: "component-v1".into(),
        },
        b"existing component".to_vec(),
      ),
    ]));
    store.shutdown();
    let stable_file = store.real_cache_path(&stable_name).1;
    let sentinel = SystemTime::UNIX_EPOCH + Duration::from_secs(42);
    std::fs::OpenOptions::new()
      .write(true)
      .open(&stable_file)
      .unwrap()
      .set_times(std::fs::FileTimes::new().set_modified(sentinel))
      .unwrap();
    store.write_cache(HashMap::from_iter([(
      CacheStoreKey {
        name: changed_name.to_string(),
        key: "customer-v2".into(),
      },
      b"new customer".to_vec(),
    )]));
    store.shutdown();
    assert_eq!(
      std::fs::metadata(&stable_file).unwrap().modified().unwrap(),
      sentinel,
      "updating a small route must not replace an unrelated cache shard"
    );
    let restored = CacheStore::new(input.to_str().unwrap(), "", Mode::Development);
    assert_eq!(
      restored.read_cache(changed_name),
      Some(b"new customer".to_vec())
    );
    assert_eq!(
      restored.read_cache(&stable_name),
      Some(b"existing component".to_vec())
    );
  }

  #[test]
  fn resource_waiter_does_not_hold_map_shard() {
    use std::sync::mpsc;
    use std::time::Duration;

    let locks = Arc::new(ResourceLock::<String>::default());
    let owner = locks.lock("package".to_string());
    let (started_tx, started_rx) = mpsc::channel();
    let waiter_locks = Arc::clone(&locks);
    let waiter = std::thread::spawn(move || {
      started_tx.send(()).unwrap();
      let _guard = waiter_locks.lock("package".to_string());
    });
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    // Give the contender time to reach the held resource mutex. The map
    // shard must remain accessible throughout that resource wait.
    std::thread::sleep(Duration::from_millis(100));
    let shard_available = !matches!(
      locks.locks.try_get("package"),
      dashmap::try_result::TryResult::Locked
    );
    // Release and join even on failure, so the regression cannot hang CI.
    drop(owner);
    waiter.join().unwrap();
    assert!(
      shard_available,
      "resource waiter retained the DashMap shard"
    );
  }

  #[test]
  fn ignores_legacy_single_emit_cache_without_removing_it() {
    let root = std::env::temp_dir().join(format!(
      "farm-cache-upgrade-{}-{}",
      std::process::id(),
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    let legacy = root.join("0.9.1-sdk/production");
    std::fs::create_dir_all(&legacy).unwrap();
    let manifest = legacy.join(FARM_CACHE_MANIFEST_FILE);
    std::fs::write(&manifest, r#"{"legacy-single-emit":"old-key"}"#).unwrap();
    let store = CacheStore::new(root.join("sdk").to_str().unwrap(), "", Mode::Production);
    assert!(
      !store.has_cache("legacy-single-emit"),
      "incomplete legacy metadata must trigger a rebuild"
    );
    assert!(
      manifest.exists(),
      "upgrading must preserve the old cache directory"
    );
  }
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
  use std::sync::Arc;

  use crate::cache::{
    store::{constant::CacheStoreItemRef, error::CacheError, CacheStoreKey},
    CacheStoreTrait,
  };

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

    fn remove_cache_only(&self, name: &str) {
      time!(
        "remove_cache_only",
        name,
        self.store.remove_cache_only(name)
      )
    }
  }
}

impl CacheStoreFactory for DiskCacheFactory {
  fn create_cache_store(&self, name: &str) -> Box<dyn CacheStoreTrait> {
    #[cfg(feature = "profile")]
    {
      let profile_store: Arc<Box<dyn CacheStoreTrait>> =
        Arc::new(Box::new(profile::TimeCacheFactory::new(self.store.clone())));
      Box::new(NamespaceStore::new(profile_store, name.to_string()))
    }

    #[cfg(not(feature = "profile"))]
    {
      Box::new(NamespaceStore::new(self.store.clone(), name.to_string()))
    }
  }
}
