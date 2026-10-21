use std::sync::Arc;

use dashmap::DashMap;
use farmfe_utils::hash::sha256;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
  cache::{
    store::{constant::CacheStoreTrait, CacheStoreKey},
    CacheContext,
  },
  deserialize,
  module::ModuleId,
  serialize, HashMap,
};

use super::{module_memory_store::ModuleMemoryStore, CachedModule};

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
    let mut cache_map = HashMap::default();
    let mut pending_removed_modules = vec![];

    for entry in self.cached_modules.iter() {
      let module = entry.value();
      if module.is_expired {
        pending_removed_modules.push(module.module.id.clone());
        continue;
      }

      let store_key = self.gen_cache_store_key(&module.module);

      if self.store.is_cache_changed(&store_key) {
        cache_map.insert(store_key, entry);
      }
    }

    let cache_map = cache_map
      .into_par_iter()
      .map(|(store_key, module)| (store_key, serialize!(module.value())))
      .collect::<HashMap<_, _>>();

    self.store.write_cache(cache_map);

    for module_id in pending_removed_modules {
      self.cached_modules.remove(&module_id);
      self.store.remove_cache_only(&module_id.to_string());
    }
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
