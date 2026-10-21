use std::sync::{
  atomic::{AtomicU64, Ordering},
  Arc,
};

use dashmap::DashMap;
use farmfe_macro_cache_item::cache_item;
use farmfe_utils::hash::sha256;
use parking_lot::Mutex;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
  cache::{
    store::{constant::CacheStoreTrait, CacheStoreKey},
    utils::cache_panic,
    CacheContext, CacheType,
  },
  deserialize,
  module::ModuleId,
  HashMap, HashSet,
};

use super::{module_memory_store::ModuleMemoryStore, CachedModule};

const MANIFEST_KEY: &str = "immutable-modules.json";

#[cache_item]
pub struct CachedPackage {
  pub list: Vec<CachedModule>,
  name: String,
  version: String,
}

impl CachedPackage {
  pub fn gen_key(name: &str, version: &str) -> String {
    format!("{name}@{version}")
  }

  pub fn key(&self) -> String {
    Self::gen_key(&self.name, &self.version)
  }
}

/// In memory store for mutable modules
pub struct ImmutableModulesMemoryStore {
  cache_dir: String,
  /// low level cache store
  store: Box<dyn CacheStoreTrait>,
  /// ModuleId -> Cached Module
  cached_modules: DashMap<ModuleId, CachedModule>,
  /// moduleId -> PackageKey
  manifest: DashMap<ModuleId, String>,
  manifest_reversed: DashMap<String, HashSet<ModuleId>>,
  dirty_packages: DashMap<String, u64>,
  next_revision: AtomicU64,
  package_locks: Mutex<HashMap<String, Arc<Mutex<()>>>>,
}

impl ImmutableModulesMemoryStore {
  pub fn new(context: Arc<CacheContext>) -> Self {
    let store = context.store_factory.create_cache_store("immutable-module");
    let manifest: HashMap<String, String> =
      serde_json::from_slice(&store.read_cache(MANIFEST_KEY).unwrap_or_default())
        .unwrap_or_default();
    let manifest = manifest
      .into_iter()
      .map(|(key, value)| (ModuleId::from(key), value))
      .collect::<HashMap<ModuleId, String>>();

    let manifest_reversed = DashMap::new();

    for (key, value) in manifest.iter() {
      let mut set = manifest_reversed
        .entry(value.clone())
        .or_insert_with(HashSet::default);
      set.insert(key.clone());
    }

    let cache_dir_str = if let CacheType::Disk { cache_dir, .. } = &context.option {
      cache_dir.clone()
    } else {
      "VIRTUAL_CACHE_DIR".to_string()
    };

    Self {
      store,
      cached_modules: DashMap::new(),
      manifest: manifest.into_iter().collect(),
      manifest_reversed,
      dirty_packages: DashMap::new(),
      next_revision: AtomicU64::new(1),
      package_locks: Mutex::new(HashMap::default()),
      cache_dir: cache_dir_str.to_string(),
    }
  }

  fn read_cached_package(&self, package_key: &str) -> Option<CachedPackage> {
    let cache = self.store.read_cache(package_key)?;

    Some(deserialize!(&cache, CachedPackage))
  }

  fn package_lock(&self, key: &str) -> Arc<Mutex<()>> {
    self
      .package_locks
      .lock()
      .entry(key.to_owned())
      .or_insert_with(|| Arc::new(Mutex::new(())))
      .clone()
  }

  fn read_package(&self, module_id: &ModuleId) -> Option<()> {
    if let Some(package_key) = self
      .manifest
      .get(module_id)
      .map(|entry| entry.value().clone())
    {
      let lock = self.package_lock(&package_key);
      let _guard = lock.lock();
      let package = self.read_cached_package(&package_key)?;

      for module in package.list {
        if self.manifest.contains_key(&module.module.id) {
          self
            .cached_modules
            .entry(module.module.id.clone())
            .or_insert(module);
        }
      }

      return Some(());
    }

    None
  }

  fn mark_package_dirty(&self, key: String) {
    let revision = self.next_revision.fetch_add(1, Ordering::Relaxed);
    self.dirty_packages.insert(key, revision);
  }

  fn acknowledge_written_packages(&self, revisions: &HashMap<String, u64>) {
    for (key, revision) in revisions {
      self
        .dirty_packages
        .remove_if(key, |_, current| current == revision);
    }
  }

  fn remove_expired_modules(&self, pending: &HashSet<ModuleId>) -> HashSet<ModuleId> {
    let mut removed = HashSet::default();
    for key in pending {
      let Some(package) = self.manifest.get(key).map(|entry| entry.value().clone()) else {
        continue;
      };
      let lock = self.package_lock(&package);
      let _guard = lock.lock();
      if self
        .cached_modules
        .remove_if(key, |_, module| module.is_expired)
        .is_none()
      {
        continue;
      }
      self.manifest.remove(key);
      self.manifest_reversed.iter_mut().for_each(|mut item| {
        item.value_mut().remove(key);
      });
      removed.insert(key.clone());
    }
    removed
  }

  pub fn set_updated_cache(&self, key: ModuleId, module: CachedModule) {
    let package =
      CachedPackage::gen_key(&module.module.package_name, &module.module.package_version);
    let lock = self.package_lock(&package);
    let _guard = lock.lock();
    self.manifest.insert(key.clone(), package.clone());
    self.set_cache(key, module);
    // Publish the revision after the replacement; a writer must not acknowledge
    // a new revision while still observing the previous in-memory module.
    self.mark_package_dirty(package);
  }
  fn write_cache_with_revisions(&self, dirty_revisions: HashMap<String, u64>) {
    let mut packages = HashMap::default();
    let mut pending_remove_modules = HashSet::default();
    let mut expired_packages = HashSet::default();

    for item in self.cached_modules.iter() {
      let module = item.value();
      let package_key =
        CachedPackage::gen_key(&module.module.package_name, &module.module.package_version);

      if module.is_expired {
        pending_remove_modules.insert(item.key().clone());
        expired_packages.insert(package_key);
        continue;
      }

      let package = packages.entry(package_key.clone()).or_insert_with(Vec::new);

      package.push(item.key().clone());
      self.manifest.insert(item.key().clone(), package_key);
    }

    let pending_remove_modules = self.remove_expired_modules(&pending_remove_modules);

    // A dirty package can contain only unloaded surviving members. It still
    // needs the merge path, even when every loaded member was removed.
    for package in dirty_revisions.keys() {
      packages.entry(package.clone()).or_insert_with(Vec::new);
    }

    // Invalidation can happen after the dirty snapshot. Every package seen
    // with an expired member must merge its unloaded survivors as well.
    for package in &expired_packages {
      packages.entry(package.clone()).or_insert_with(Vec::new);
    }

    let manifest = self
      .manifest
      .iter()
      .map(|item| (item.key().to_string(), item.value().to_string()))
      .collect::<HashMap<String, String>>();

    let manifest_bytes = serde_json::to_vec(&manifest)
      .unwrap_or_else(|e| cache_panic(&e.to_string(), &self.cache_dir));

    let mut cache_map = packages
      .into_par_iter()
      .filter_map(|(key, modules)| {
        if dirty_revisions.contains_key(&key) || expired_packages.contains(&key) {
          // A forced update may keep every module ID and source hash unchanged.
          // Merge the persisted package with memory, preserving unloaded members.
          let previous = self.read_cached_package(&key);
          let mut members = previous
            .as_ref()
            .map(|package| {
              package
                .list
                .iter()
                .filter(|module| !pending_remove_modules.contains(&module.module.id))
                .map(|module| (module.module.id.clone(), module.clone()))
                .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default();
          for id in modules {
            if let Some(module) = self.cached_modules.get(&id) {
              if module.is_expired {
                members.remove(&id);
              } else {
                members.insert(id, module.value().clone());
              }
            }
          }
          let mut list = members.into_values().collect::<Vec<_>>();
          list.sort_by_key(|module| module.module.id.to_string());
          let identity = list
            .first()
            .map(|module| {
              (
                module.module.package_name.clone(),
                module.module.package_version.clone(),
              )
            })
            .or_else(|| {
              previous
                .as_ref()
                .map(|package| (package.name.clone(), package.version.clone()))
            })
            .unwrap_or_default();
          let package = CachedPackage {
            name: identity.0,
            version: identity.1,
            list,
          };
          let bytes = crate::serialize!(&package);
          return Some((
            CacheStoreKey {
              name: key,
              key: sha256(&bytes, 32),
            },
            bytes,
          ));
        }
        let gen_cache_store_key = |mut modules: Vec<String>| {
          modules.sort();

          CacheStoreKey {
            name: key.clone(),
            key: sha256(modules.join(",").as_bytes(), 32),
          }
        };

        // the package is already cached, we only need to update it
        if self.manifest_reversed.contains_key(&key) {
          let modules_in_package = self.manifest_reversed.get(&key).unwrap();
          let mut added_modules = vec![];

          for module_id in modules {
            if modules_in_package.contains(&module_id) {
              continue;
            }
            added_modules.push(module_id);
          }

          // add the new modules to the package
          if !added_modules.is_empty() {
            let mut package = self.read_cached_package(&key).unwrap();
            package.list.extend(
              added_modules
                .into_par_iter()
                .map(|module_id| {
                  self
                    .cached_modules
                    .get(&module_id)
                    .unwrap_or_else(|| cache_panic(&key.to_string(), &self.cache_dir))
                    .clone()
                })
                .collect::<Vec<_>>(),
            );
            let modules = package
              .list
              .iter()
              .map(|cm| cm.module.id.to_string())
              .collect::<Vec<_>>();
            let package_bytes = crate::serialize!(&package);
            return Some((gen_cache_store_key(modules), package_bytes));
          }
          return None;
        }

        let module_strings = modules.iter().map(|m| m.to_string()).collect::<Vec<_>>();
        let package = CachedPackage {
          list: modules
            .into_par_iter()
            .map(|module_id| {
              self
                .cached_modules
                .get(&module_id)
                .unwrap_or_else(|| cache_panic(&key.to_string(), &self.cache_dir))
                .clone()
            })
            .collect(),
          name: key.split('@').next().unwrap().to_string(),
          version: key.split('@').next_back().unwrap().to_string(),
        };

        let package_bytes = crate::serialize!(&package);
        Some((gen_cache_store_key(module_strings), package_bytes))
      })
      .collect::<HashMap<CacheStoreKey, Vec<u8>>>();

    cache_map.insert(
      CacheStoreKey {
        name: MANIFEST_KEY.to_string(),
        key: sha256(manifest_bytes.as_slice(), 32),
      },
      manifest_bytes,
    );

    self.store.write_cache(cache_map);
    // A concurrent later replacement remains dirty. Revisions never restart
    // when a package is acknowledged, avoiding an ABA on the package name.
    self.acknowledge_written_packages(&dirty_revisions);
  }
}

impl ModuleMemoryStore for ImmutableModulesMemoryStore {
  fn has_cache(&self, key: &crate::module::ModuleId) -> bool {
    self.get_cache_ref(key).is_some_and(|m| !m.is_expired)
  }

  fn set_cache(&self, key: crate::module::ModuleId, module: super::CachedModule) {
    self.store.remove_cache_only(&key.to_string());
    self.cached_modules.insert(key, module);
  }

  fn get_cache(&self, key: &crate::module::ModuleId) -> Option<super::CachedModule> {
    self.get_cache_ref(key).map(|v| v.value().clone())
  }

  fn get_cache_ref(
    &self,
    key: &crate::module::ModuleId,
  ) -> Option<dashmap::mapref::one::Ref<'_, crate::module::ModuleId, super::CachedModule>> {
    if !self.manifest.contains_key(key) {
      return None;
    }

    if !self.cached_modules.contains_key(key) {
      self.read_package(key);
    }

    self.cached_modules.get(key)
  }

  fn get_cache_mut_ref(
    &self,
    key: &crate::module::ModuleId,
  ) -> Option<dashmap::mapref::one::RefMut<'_, crate::module::ModuleId, super::CachedModule>> {
    if self.cached_modules.contains_key(key) {
      return Some(self.cached_modules.get_mut(key).unwrap());
    }

    self.read_package(key);

    self.cached_modules.get_mut(key)
  }

  fn write_cache(&self) {
    let dirty_revisions = self
      .dirty_packages
      .iter()
      .map(|entry| (entry.key().clone(), *entry.value()))
      .collect::<HashMap<_, _>>();
    self.write_cache_with_revisions(dirty_revisions);
  }

  fn invalidate_cache(&self, key: &ModuleId) {
    let package = if let Some(mut m) = self.get_cache_mut_ref(key) {
      m.is_expired = true;
      Some(CachedPackage::gen_key(
        &m.module.package_name,
        &m.module.package_version,
      ))
    } else {
      None
    };
    if let Some(package) = package {
      self.mark_package_dirty(package);
    }
  }

  fn is_cache_changed(&self, module: &crate::module::Module) -> bool {
    // we do not need to check the hash of immutable modules, just check the cache
    !self.has_cache(&module.id)
  }

  fn cache_outdated(&self, key: &ModuleId) -> bool {
    !self.cached_modules.contains_key(key)
  }
}

#[cfg(test)]
mod forced_package_tests {
  use super::*;
  use crate::module::{Module, ModuleMetaData, ModuleType};

  const PACKAGE: &str = "fixture@1.0.0";

  fn module(id: &str, generated_async: bool) -> CachedModule {
    let mut module = Module::new(id.into());
    module.immutable = true;
    module.module_type = ModuleType::Js;
    module.package_name = "fixture".into();
    module.package_version = "1.0.0".into();
    module.content_hash = "same-source".into();
    module.meta = Box::new(ModuleMetaData::Script(Default::default()));
    module.meta.as_script_mut().is_async = generated_async;
    CachedModule {
      module,
      dependencies: vec![],
      watch_dependencies: vec![],
      is_expired: false,
    }
  }

  fn seeded() -> Arc<CacheContext> {
    let context = Arc::new(CacheContext {
      cache_enable: true,
      ..Default::default()
    });
    let store = ImmutableModulesMemoryStore::new(context.clone());
    store.set_cache("a.js".into(), module("a.js", false));
    store.set_cache("b.js".into(), module("b.js", false));
    store.write_cache();
    context
  }

  #[test]
  fn forced_package_update_preserves_unloaded_members() {
    let context = seeded();
    let store = ImmutableModulesMemoryStore::new(context.clone());
    store.set_updated_cache("a.js".into(), module("a.js", true));
    assert!(!store.cached_modules.contains_key(&ModuleId::from("b.js")));
    store.write_cache();
    let restored = ImmutableModulesMemoryStore::new(context);
    assert!(
      restored
        .get_cache(&"a.js".into())
        .unwrap()
        .module
        .meta
        .as_script()
        .is_async
    );
    assert!(
      !restored
        .get_cache(&"b.js".into())
        .unwrap()
        .module
        .meta
        .as_script()
        .is_async
    );
    assert_eq!(
      restored.read_cached_package(PACKAGE).unwrap().key(),
      PACKAGE
    );
  }

  #[test]
  fn reading_sibling_keeps_new_in_memory_metadata() {
    let context = seeded();
    let store = ImmutableModulesMemoryStore::new(context);
    store.set_updated_cache("a.js".into(), module("a.js", true));
    assert!(store.get_cache(&"b.js".into()).is_some());
    assert!(
      store
        .get_cache(&"a.js".into())
        .unwrap()
        .module
        .meta
        .as_script()
        .is_async
    );
  }

  #[test]
  fn package_reload_and_rewrite_do_not_revive_expired_member() {
    let context = seeded();
    let store = ImmutableModulesMemoryStore::new(context.clone());
    store.invalidate_cache(&"a.js".into());
    store.cached_modules.remove(&ModuleId::from("b.js"));
    assert!(store.get_cache(&"b.js".into()).is_some());
    assert!(!store.has_cache(&"a.js".into()));
    store.write_cache();
    let restored = ImmutableModulesMemoryStore::new(context);
    assert!(!restored.has_cache(&"a.js".into()));
    assert!(restored.get_cache(&"b.js".into()).is_some());
  }

  #[test]
  fn earlier_write_acknowledgement_preserves_later_dirty_revision() {
    let store = ImmutableModulesMemoryStore::new(seeded());
    store.set_updated_cache("a.js".into(), module("a.js", true));
    let first = *store.dirty_packages.get(PACKAGE).unwrap();
    store.set_updated_cache("a.js".into(), module("a.js", false));
    store.acknowledge_written_packages(&[(PACKAGE.into(), first)].into_iter().collect());
    assert!(*store.dirty_packages.get(PACKAGE).unwrap() > first);
  }

  #[test]
  fn acknowledged_package_revision_never_restarts() {
    let store = ImmutableModulesMemoryStore::new(seeded());
    store.set_updated_cache("a.js".into(), module("a.js", true));
    let first = *store.dirty_packages.get(PACKAGE).unwrap();
    let previous = [(PACKAGE.into(), first)].into_iter().collect();
    store.acknowledge_written_packages(&previous);
    assert!(!store.dirty_packages.contains_key(PACKAGE));
    store.set_updated_cache("a.js".into(), module("a.js", false));
    store.acknowledge_written_packages(&previous);
    assert!(*store.dirty_packages.get(PACKAGE).unwrap() > first);
  }

  #[test]
  fn expiring_only_loaded_member_preserves_unloaded_sibling() {
    let context = seeded();
    let store = ImmutableModulesMemoryStore::new(context.clone());
    store.set_updated_cache("a.js".into(), module("a.js", true));
    assert!(!store.cached_modules.contains_key(&ModuleId::from("b.js")));
    store.invalidate_cache(&"a.js".into());
    store.write_cache();
    let restored = ImmutableModulesMemoryStore::new(context);
    assert!(
      restored.has_cache(&"b.js".into()),
      "expired loaded member must preserve unloaded sibling"
    );
    assert!(!restored.has_cache(&"a.js".into()));
  }

  #[test]
  fn expired_scan_preserves_replacement_published_after_scan() {
    let store = ImmutableModulesMemoryStore::new(seeded());
    store.invalidate_cache(&"a.js".into());
    let scanned = [ModuleId::from("a.js")].into_iter().collect();
    store.set_updated_cache("a.js".into(), module("a.js", true));
    let removed = store.remove_expired_modules(&scanned);
    assert!(
      removed.is_empty(),
      "expired scan must preserve a replacement"
    );
    assert!(
      store
        .get_cache(&"a.js".into())
        .unwrap()
        .module
        .meta
        .as_script()
        .is_async
    );
  }

  #[test]
  fn invalidation_after_dirty_snapshot_preserves_unloaded_sibling() {
    let context = seeded();
    let store = ImmutableModulesMemoryStore::new(context.clone());
    store.set_updated_cache("a.js".into(), module("a.js", true));
    store.write_cache();
    assert!(store.dirty_packages.is_empty());
    assert!(!store.cached_modules.contains_key(&ModuleId::from("b.js")));
    // Deterministic interleaving: writer captures no dirty revisions, then
    // another operation invalidates A before that writer scans memory.
    let snapshot = HashMap::default();
    store.invalidate_cache(&"a.js".into());
    store.write_cache_with_revisions(snapshot);
    let restored = ImmutableModulesMemoryStore::new(context);
    assert!(
      restored.has_cache(&"b.js".into()),
      "invalidation after dirty snapshot must preserve unloaded sibling"
    );
    assert!(!restored.has_cache(&"a.js".into()));
  }
}
