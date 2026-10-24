#[cfg(test)]
mod tests {
  #[test]
  fn interrupted_disk_cache_is_a_miss_and_can_be_rebuilt() {
    use farmfe_core::cache::store::{CacheStore, CacheStoreKey, constant::CacheStoreTrait};
    use farmfe_core::config::Mode;
    let root = std::env::temp_dir().join(format!(
      "farm-interrupted-cache-{}-{}",
      std::process::id(),
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    let cache = root.join("cache");
    let cache_path = cache.to_str().unwrap();
    let store = CacheStore::new(cache_path, "", Mode::Development);
    let key = CacheStoreKey::from(("interrupted-module", "same-input-hash"));
    store.write_single_cache(key.clone(), vec![17, 25]).unwrap();
    store.shutdown();
    let version = farmfe_core::cache::store::constant::FARM_CACHE_VERSION;
    let directory = root.join(format!("{version}-cache/development"));
    let shard = std::fs::read_dir(&directory)
      .unwrap()
      .map(|entry| entry.unwrap().path())
      .find(|file| {
        file
          .file_name()
          .unwrap()
          .to_string_lossy()
          .starts_with("cache-")
      })
      .unwrap();
    let complete = std::fs::read(&shard).unwrap();
    assert!(complete.len() > 1);
    for damaged in [
      vec![],
      complete[..complete.len() / 2].to_vec(),
      vec![255; complete.len()],
    ] {
      std::fs::write(&shard, damaged).unwrap();
      let restored = CacheStore::new(cache_path, "", Mode::Development);
      assert_eq!(
        restored.read_cache(&key.name),
        None,
        "an interrupted cache must be treated as a miss"
      );
      restored.write_single_cache(key.clone(), vec![42]).unwrap();
      restored.shutdown();
      let rebuilt = CacheStore::new(cache_path, "", Mode::Development);
      assert_eq!(
        rebuilt.read_cache(&key.name),
        Some(vec![42]),
        "same-key rebuild must replace invalid cache state"
      );
    }
  }

  #[test]
  fn invalid_cache_manifest_can_be_replaced_by_a_rebuild() {
    use farmfe_core::cache::store::{CacheStore, CacheStoreKey, constant::CacheStoreTrait};
    use farmfe_core::config::Mode;
    let root = std::env::temp_dir().join(format!(
      "farm-invalid-manifest-{}-{}",
      std::process::id(),
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    let cache = root.join("cache");
    let cache_path = cache.to_str().unwrap();
    let version = farmfe_core::cache::store::constant::FARM_CACHE_VERSION;
    let directory = root.join(format!("{version}-cache/development"));
    std::fs::create_dir_all(&directory).unwrap();
    let manifest = directory.join(farmfe_core::cache::store::constant::FARM_CACHE_MANIFEST_FILE);
    for invalid in [b"".as_slice(), b"{\"module\":", b"[]", &[255]] {
      std::fs::write(&manifest, invalid).unwrap();
      let store = CacheStore::new(cache_path, "", Mode::Development);
      assert_eq!(store.read_cache("module"), None);
      store
        .write_single_cache(CacheStoreKey::from(("module", "input")), vec![42])
        .unwrap();
      store.shutdown();
      let rebuilt = CacheStore::new(cache_path, "", Mode::Development);
      assert_eq!(rebuilt.read_cache("module"), Some(vec![42]));
    }
  }

  #[test]
  fn old_manifest_with_a_new_shard_is_a_rebuildable_cache_miss() {
    use farmfe_core::cache::store::{CacheStore, CacheStoreKey, constant::CacheStoreTrait};
    use farmfe_core::config::Mode;
    let root = std::env::temp_dir().join(format!(
      "farm-interrupted-manifest-{}-{}",
      std::process::id(),
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    let cache = root.join("cache");
    let cache_path = cache.to_str().unwrap();
    let version = farmfe_core::cache::store::constant::FARM_CACHE_VERSION;
    let manifest = root
      .join(format!("{version}-cache/development"))
      .join(farmfe_core::cache::store::constant::FARM_CACHE_MANIFEST_FILE);
    let old_key = CacheStoreKey::from(("module", "old-input"));
    let store = CacheStore::new(cache_path, "", Mode::Development);
    store.write_single_cache(old_key.clone(), vec![17]).unwrap();
    store.shutdown();
    let old_manifest = std::fs::read(&manifest).unwrap();
    let store = CacheStore::new(cache_path, "", Mode::Development);
    assert_eq!(store.read_cache("module"), Some(vec![17]));
    store
      .write_single_cache(CacheStoreKey::from(("module", "new-input")), vec![42])
      .unwrap();
    store.shutdown();
    // Simulate stopping after the new shard was published but before its manifest.
    std::fs::write(&manifest, old_manifest).unwrap();
    let store = CacheStore::new(cache_path, "", Mode::Development);
    assert_eq!(store.read_cache("module"), None);
    store.write_single_cache(old_key, vec![25]).unwrap();
    store.shutdown();
    let rebuilt = CacheStore::new(cache_path, "", Mode::Development);
    assert_eq!(rebuilt.read_cache("module"), Some(vec![25]));
  }

  #[test]
  fn shared_mutex_remains_usable_after_a_panicking_writer() {
    let lock = farmfe_core::sync::Mutex::new(1);
    let result = std::panic::catch_unwind(|| {
      let mut guard = lock.lock();
      *guard = 2;
      panic!("simulate an interrupted plugin hook");
    });
    assert!(result.is_err());
    assert_eq!(*lock.lock(), 2);
    assert_eq!(*lock.try_lock().expect("poisoned lock remains usable"), 2);
    let guard = lock.lock();
    assert!(lock.try_lock().is_none());
    drop(guard);
  }

  #[test]
  fn shared_rwlock_remains_usable_after_a_panicking_writer() {
    let lock = farmfe_core::sync::RwLock::new(1);
    let result = std::panic::catch_unwind(|| {
      let mut guard = lock.write();
      *guard = 2;
      panic!("simulate an interrupted plugin hook");
    });
    assert!(result.is_err());
    assert_eq!(*lock.read(), 2);
    assert_eq!(*lock.try_read().expect("poisoned lock remains readable"), 2);
    *lock.try_write().expect("poisoned lock remains writable") = 3;
    let guard = lock.write();
    assert!(lock.try_read().is_none());
    assert!(lock.try_write().is_none());
    drop(guard);
    assert_eq!(*lock.read(), 3);
  }

  use farmfe_core::{
    config::{Config, persistent_cache::PersistentCacheConfig},
    context::CompilationContext,
  };
  use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
  };
  use std::time::{Duration, Instant};

  #[test]
  fn host_reader_wakes_after_dynamic_plugin_releases_watch_graph() {
    assert_wake(b"hold_watch_graph", |context| {
      context
        .watch_graph
        .read()
        .has_module(&"cross-library-module".into())
    });
  }

  #[test]
  fn host_reader_wakes_after_dynamic_plugin_releases_resources() {
    assert_wake(b"hold_resources", |context| {
      context.resources_map.lock().is_empty()
    });
  }

  fn assert_wake(symbol: &'static [u8], read: fn(&CompilationContext) -> bool) {
    let filename =
      std::env::var("FARM_SHARED_LOCK_LIBRARY").expect("Build the DLL with npm run build first");
    // SAFETY: this loads our explicitly built, matching test-only fixture.
    let library = Arc::new(unsafe { libloading::Library::new(filename) }.unwrap());
    let config = Config {
      persistent_cache: Box::new(PersistentCacheConfig::Bool(false)),
      ..Default::default()
    };
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    let ready = Arc::new(AtomicBool::new(false));
    let started = Arc::new(AtomicBool::new(false));
    let writer = {
      let (library, context, ready, started) = (
        library.clone(),
        context.clone(),
        ready.clone(),
        started.clone(),
      );
      std::thread::spawn(move || {
        type Hold =
          unsafe extern "C" fn(*const CompilationContext, *const AtomicBool, *const AtomicBool);
        // SAFETY: the fixture signature matches; Arc owners and the Library
        // stay alive until this synchronous invocation returns.
        unsafe {
          let hold: libloading::Symbol<Hold> = library.get(symbol).unwrap();
          hold(
            Arc::as_ptr(&context),
            Arc::as_ptr(&ready),
            Arc::as_ptr(&started),
          );
        }
      })
    };
    let deadline = Instant::now() + Duration::from_secs(5);
    while !ready.load(Ordering::Acquire) {
      assert!(
        Instant::now() < deadline,
        "Fixture failed to acquire its write guard"
      );
      std::thread::yield_now();
    }
    let (send, receive) = mpsc::channel();
    let reader = std::thread::spawn(move || {
      started.store(true, Ordering::Release);
      send.send(read(&context)).unwrap();
    });
    writer.join().unwrap();
    assert_eq!(
      receive.recv_timeout(Duration::from_secs(3)),
      Ok(true),
      "The DLL released its guard, but the host reader was never woken"
    );
    reader.join().unwrap();
  }
}
