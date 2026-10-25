use std::sync::Arc;

use dashmap::DashMap;
use farmfe_utils::hash::sha256;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
  HashMap,
  cache::{
    CacheContext,
    store::{CacheStoreKey, constant::CacheStoreTrait},
  },
  deserialize,
  module::ModuleId,
  serialize,
};

use super::{CachedModule, module_memory_store::ModuleMemoryStore};

/// In memory store for mutable modules
pub struct MutableModulesMemoryStore {
  /// low level cache store
  store: Box<dyn CacheStoreTrait>,
  /// ModuleId -> Cached Module
  cached_modules: DashMap<ModuleId, CachedModule>,
}
// TODO: cache unit test
impl MutableModulesMemoryStore {
  pub fn new(context: Arc<CacheContext>) -> Self {
    Self {
      store: context.store_factory.create_cache_store("mutable-module"),
      cached_modules: DashMap::new(),
    }
  }

  fn gen_cache_store_key(&self, module: &crate::module::Module) -> CacheStoreKey {
    // Fix vue cache timestamp validation. Tools like vue may generate virtual modules which id is ends with .vue?vue,
    // if the original module is changed, but the virtual module's content hash is not changed, in this cache, the cache should be invalidated.
    let timestamp = if module.id.query_string().is_empty() {
      0
    } else {
      module.last_update_timestamp
    };
    let hash_key = sha256(
      format!(
        "{}{}{}",
        module.content_hash,
        module.id.to_string(),
        timestamp
      )
      .as_bytes(),
      32,
    );
    CacheStoreKey {
      name: module.id.to_string(),
      key: hash_key,
    }
  }

  fn read_cache(&self, name: String) -> Option<()> {
    // Restoring a module must retain its persisted identity: freezeModule
    // uses that identity to distinguish a real cache hit from a rebuild.
    let v = self.store.read_cache(&name)?;
    let module = deserialize!(&v, CachedModule);

    // A persisted copy must not overwrite a newer in-memory value, including
    // the expired marker installed by invalidate_cache after the last write.
    self
      .cached_modules
      .entry(ModuleId::from(name))
      .or_insert(module);

    Some(())
  }
}

impl ModuleMemoryStore for MutableModulesMemoryStore {
  fn has_cache(&self, key: &ModuleId) -> bool {
    self.get_cache_ref(key).is_some_and(|m| !m.is_expired)
  }

  fn set_cache(&self, key: ModuleId, module: CachedModule) {
    self.cached_modules.insert(key.clone(), module);
    self.store.remove_cache_only(&key.to_string());
  }

  fn get_cache(&self, key: &ModuleId) -> Option<CachedModule> {
    self.get_cache_ref(key).map(|v| v.value().clone())
  }

  fn get_cache_ref(
    &self,
    key: &ModuleId,
  ) -> Option<dashmap::mapref::one::Ref<'_, ModuleId, CachedModule>> {
    if let Some(module) = self.cached_modules.get(key) {
      return Some(module);
    }

    let _ = self.read_cache(key.to_string());

    self.cached_modules.get(key)
  }

  fn get_cache_mut_ref(
    &self,
    key: &ModuleId,
  ) -> Option<dashmap::mapref::one::RefMut<'_, ModuleId, CachedModule>> {
    if let Some(module) = self.cached_modules.get_mut(key) {
      return Some(module);
    }

    let _ = self.read_cache(key.to_string());

    self.cached_modules.get_mut(key)
  }

  fn write_cache(&self) {
    // Retain keys, not read guards for the entire module batch.
    let module_ids = self
      .cached_modules
      .iter()
      .map(|entry| entry.key().clone())
      .collect::<Vec<_>>();

    module_ids.into_par_iter().for_each(|module_id| {
      let Some(entry) = self.cached_modules.get(&module_id) else {
        return;
      };

      if entry.is_expired {
        drop(entry);
        // A newer set_cache may have replaced the expired value meanwhile.
        // Keep its conditional removal and backend invalidation under one lock.
        let _ = self.cached_modules.remove_if(&module_id, |_, current| {
          if current.is_expired {
            self.store.remove_cache_only(&module_id.to_string());
            true
          } else {
            false
          }
        });
        return;
      }

      let store_key = self.gen_cache_store_key(&entry.module);
      if self.store.is_cache_changed(&store_key) {
        // Publish this module before releasing its guard. A concurrent set_cache
        // must invalidate this publication after replacing the in-memory value.
        let cache_map: HashMap<_, _> =
          std::iter::once((store_key, serialize!(entry.value()))).collect();
        self.store.write_cache(cache_map);
      }
    });
  }

  fn invalidate_cache(&self, key: &ModuleId) {
    if let Some(mut m) = self.get_cache_mut_ref(key) {
      m.is_expired = true;
    };
  }

  fn is_cache_changed(&self, module: &crate::module::Module) -> bool {
    // A plugin may force a rebuild without changing transformed bytes. Its
    // expired metadata must still be replaced before the next cache flush.
    self
      .cached_modules
      .get(&module.id)
      .is_some_and(|cached| cached.is_expired)
      || self
        .store
        .is_cache_changed(&self.gen_cache_store_key(module))
  }

  fn cache_outdated(&self, key: &ModuleId) -> bool {
    !self.cached_modules.contains_key(key)
  }
}

#[cfg(test)]
mod tests {
  use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
  };

  use super::*;
  use crate::{cache::store::CacheStore, config::Mode, module::Module};

  static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

  struct Fixture {
    cache: Option<MutableModulesMemoryStore>,
    directory: PathBuf,
  }

  impl Fixture {
    fn new() -> Self {
      let directory = std::env::temp_dir().join(format!(
        "farm-mutable-memory-hit-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .expect("test clock")
          .as_nanos(),
        NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
      ));
      std::fs::create_dir(&directory).expect("create owned test directory");
      let cache = MutableModulesMemoryStore {
        store: Box::new(CacheStore::new(
          directory
            .join("cache")
            .to_str()
            .expect("test directory is UTF-8"),
          "",
          Mode::Development,
        )),
        cached_modules: DashMap::new(),
      };
      Self {
        cache: Some(cache),
        directory,
      }
    }

    fn cache(&self) -> &MutableModulesMemoryStore {
      self.cache.as_ref().expect("fixture cache")
    }
  }

  impl Drop for Fixture {
    fn drop(&mut self) {
      drop(self.cache.take());
      if let (Ok(directory), Ok(temporary)) = (
        self.directory.canonicalize(),
        std::env::temp_dir().canonicalize(),
      ) {
        if directory.parent() == Some(temporary.as_path())
          && directory.file_name() == self.directory.file_name()
        {
          let _ = std::fs::remove_dir_all(directory);
        }
      }
    }
  }

  fn cached_module(id: &ModuleId) -> CachedModule {
    let mut module = Module::new(id.clone());
    module.content_hash = "original-content".to_string();
    CachedModule {
      module,
      dependencies: vec![],
      watch_dependencies: vec![],
      is_expired: false,
    }
  }

  #[test]
  fn memory_hit_preserves_written_lower_store_copy() {
    let fixture = Fixture::new();
    let cache = fixture.cache();
    let id = ModuleId::from("memory-hit.ts");
    cache.set_cache(id.clone(), cached_module(&id));
    cache.write_cache();
    let before = cache
      .store
      .read_cache(&id.to_string())
      .expect("written copy");

    for _ in 0..3 {
      assert_eq!(
        cache
          .get_cache_ref(&id)
          .expect("memory hit")
          .module
          .content_hash,
        "original-content"
      );
      assert!(
        cache.store.has_cache(&id.to_string()),
        "a memory hit must preserve its persisted copy"
      );
      assert!(cache.has_cache(&id));
      assert!(!cache.get_cache(&id).expect("owned memory hit").is_expired);
    }
    assert_eq!(cache.store.read_cache(&id.to_string()), Some(before));
  }

  #[test]
  fn expired_memory_marker_preserves_written_lower_store_copy() {
    let fixture = Fixture::new();
    let cache = fixture.cache();
    let id = ModuleId::from("expired-memory-hit.ts");
    cache.set_cache(id.clone(), cached_module(&id));
    cache.write_cache();
    let before = cache
      .store
      .read_cache(&id.to_string())
      .expect("written copy");
    cache.invalidate_cache(&id);

    assert!(cache.get_cache_ref(&id).expect("expired marker").is_expired);
    assert!(!cache.has_cache(&id));
    assert_eq!(cache.store.read_cache(&id.to_string()), Some(before));
  }

  #[test]
  fn expired_same_content_rebuild_replaces_expired_metadata() {
    let fixture = Fixture::new();
    let cache = fixture.cache();
    let id = ModuleId::from("forced-same-content.ts");
    cache.set_cache(id.clone(), cached_module(&id));
    cache.write_cache();
    cache.invalidate_cache(&id);
    let rebuilt = cached_module(&id);
    assert!(
      cache.is_cache_changed(&rebuilt.module),
      "an explicit invalidation requires fresh metadata even with the same content hash"
    );
    cache.set_cache(id.clone(), rebuilt);
    cache.write_cache();
    cache.cached_modules.remove(&id);
    assert!(
      cache.has_cache(&id),
      "the next build must restore the repaired module"
    );
    assert!(!cache.get_cache(&id).unwrap().is_expired);
  }

  #[test]
  fn cold_memory_miss_restores_written_module() {
    let fixture = Fixture::new();
    let cache = fixture.cache();
    let id = ModuleId::from("cold-memory-miss.ts");
    cache.set_cache(id.clone(), cached_module(&id));
    cache.write_cache();
    cache.cached_modules.remove(&id);

    assert_eq!(
      cache
        .get_cache_ref(&id)
        .expect("cold recovery")
        .module
        .content_hash,
      "original-content"
    );
    assert!(cache.has_cache(&id));
    assert!(
      !cache.is_cache_changed(&cache.get_cache(&id).expect("restored module").module),
      "cold recovery must preserve unchanged module cache identity"
    );
  }
}

#[cfg(test)]
mod controlled_write_overlap_probe {
  use super::*;
  use crate::{cache::store::controlled_write_backend_probe, module::Module};
  use dashmap::try_result::TryResult;
  use std::{sync::mpsc, thread, time::Duration};
  fn payload(id: ModuleId, hash: &str) -> CachedModule {
    let mut module = Module::new(id);
    module.content_hash = hash.into();
    module.content = Arc::new(hash.into());
    CachedModule {
      module,
      dependencies: vec![],
      watch_dependencies: vec![],
      is_expired: false,
    }
  }
  fn separate_keys(cache: &MutableModulesMemoryStore, count: usize) -> Vec<ModuleId> {
    let ids = (0..256)
      .map(|n| ModuleId::from(format!("probe-key-{n}")))
      .collect::<Vec<_>>();
    for id in &ids {
      cache.set_cache(id.clone(), payload(id.clone(), "initial"));
    }
    let mut selected = vec![ids[0].clone()];
    for candidate in ids.iter().skip(1) {
      if selected.len() == count {
        break;
      }
      let independent = {
        let guards = selected
          .iter()
          .map(|id| cache.cached_modules.get(id).expect("selected module"))
          .collect::<Vec<_>>();
        let available = match cache.cached_modules.try_get_mut(candidate) {
          TryResult::Present(guard) => {
            drop(guard);
            true
          }
          TryResult::Locked => false,
          TryResult::Absent => panic!("candidate missing"),
        };
        drop(guards);
        available
      };
      if independent {
        selected.push(candidate.clone());
      }
    }
    assert_eq!(
      selected.len(),
      count,
      "actual concurrent try_get_mut establishes disjoint shard locks; no raw-api feature"
    );
    cache.cached_modules.retain(|id, _| selected.contains(id));
    cache
      .cached_modules
      .iter()
      .map(|entry| entry.key().clone())
      .collect()
  }
  #[test]
  fn control_without_writer_advances() {
    let (store, _, _) = controlled_write_backend_probe::create();
    let cache = MutableModulesMemoryStore {
      store,
      cached_modules: DashMap::new(),
    };
    let id = separate_keys(&cache, 1).remove(0);
    cache.invalidate_cache(&id);
    cache.set_cache(id.clone(), payload(id.clone(), "updated"));
    assert_eq!(
      cache
        .get_cache(&id)
        .expect("updated cache")
        .module
        .content_hash,
      "updated"
    );
  }
  #[test]
  fn unrelated_update_advances_while_second_backend_check_is_paused() {
    let (store, state, paused) = controlled_write_backend_probe::create();
    let cache = Arc::new(MutableModulesMemoryStore {
      store,
      cached_modules: DashMap::new(),
    });
    separate_keys(&cache, 2);
    let writing = cache.clone();
    let writer = thread::spawn(move || writing.write_cache());
    let (first, _) = paused
      .recv_timeout(Duration::from_secs(10))
      .expect("second backend check");
    let first = ModuleId::from(first);
    let (start_tx, start_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let updating = cache.clone();
    let id = first.clone();
    let updater = thread::spawn(move || {
      start_tx.send(()).expect("start");
      updating.invalidate_cache(&id);
      updating.set_cache(id.clone(), payload(id, "updated"));
      done_tx.send(()).expect("done");
    });
    start_rx
      .recv_timeout(Duration::from_secs(10))
      .expect("updater started");
    let advanced = done_rx.recv_timeout(Duration::from_secs(2)).is_ok();
    state.release();
    writer.join().expect("writer");
    updater.join().expect("updater");
    assert_eq!(state.written_payload_count(), 2);
    assert_eq!(
      cache
        .get_cache(&first)
        .expect("updated entry")
        .module
        .content_hash,
      "updated"
    );
    println!(
      "disjoint_shards_observed=true; advanced_while_other_backend_paused={advanced}; threads_joined=true"
    );
    assert!(
      advanced,
      "write_cache retained unrelated module read guard across a distinct backend check"
    );
  }
  #[test]
  fn publication_does_not_reintroduce_same_source_old_metadata() {
    let (store, state, paused) = controlled_write_backend_probe::create_publish();
    let cache = Arc::new(MutableModulesMemoryStore {
      store,
      cached_modules: DashMap::new(),
    });
    let id = ModuleId::from("generated-same-source.css");
    let mut old = payload(id.clone(), "unchanged-source-hash");
    old.module.content = Arc::new("generated-old".into());
    cache.set_cache(id.clone(), old);
    let writing = cache.clone();
    let writer = thread::spawn(move || writing.write_cache());
    paused
      .recv_timeout(Duration::from_secs(10))
      .expect("publication paused");
    let (start_tx, start_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let updating = cache.clone();
    let update_id = id.clone();
    let updater = thread::spawn(move || {
      start_tx.send(()).expect("start");
      let mut new = payload(update_id.clone(), "unchanged-source-hash");
      new.module.content = Arc::new("generated-new".into());
      updating.set_cache(update_id, new);
      done_tx.send(()).expect("done");
    });
    start_rx
      .recv_timeout(Duration::from_secs(10))
      .expect("started");
    let progressed = done_rx.recv_timeout(Duration::from_millis(250)).is_ok();
    state.release();
    writer.join().expect("writer");
    updater.join().expect("updater");
    let persisted = state.persisted_content(&id.to_string());
    println!("updated_before_publication_release={progressed}; persisted={persisted:?}");
    assert_eq!(
      cache
        .get_cache(&id)
        .expect("latest memory entry")
        .module
        .content
        .as_str(),
      "generated-new"
    );
    assert!(
      persisted
        .as_deref()
        .is_none_or(|content| content == "generated-new"),
      "older serialized same-source metadata was published after a newer set_cache invalidated it"
    );
  }
  #[test]
  fn expired_cleanup_preserves_fresh_replacement() {
    let (store, state, paused) = controlled_write_backend_probe::create();
    let cache = Arc::new(MutableModulesMemoryStore {
      store,
      cached_modules: DashMap::new(),
    });
    let ordered = separate_keys(&cache, 3);
    let expired = ordered[0].clone();
    cache.invalidate_cache(&expired);
    let writing = cache.clone();
    let writer = thread::spawn(move || writing.write_cache());
    let (first, second) = paused
      .recv_timeout(Duration::from_secs(10))
      .expect("live backend entries");
    assert_ne!(expired.to_string(), first);
    assert_ne!(expired.to_string(), second);
    let (done_tx, done_rx) = mpsc::channel();
    let updating = cache.clone();
    let id = expired.clone();
    let updater = thread::spawn(move || {
      updating.set_cache(id.clone(), payload(id, "fresh-replacement"));
      done_tx.send(()).expect("done");
    });
    let replaced = done_rx.recv_timeout(Duration::from_secs(2)).is_ok();
    state.release();
    writer.join().expect("writer");
    updater.join().expect("updater");
    assert!(
      replaced,
      "expired entry replacement should not be pinned by live entries"
    );
    assert_eq!(
      cache
        .get_cache(&expired)
        .expect("expired cleanup erased a fresh replacement")
        .module
        .content_hash,
      "fresh-replacement"
    );
  }
}
