use std::{
  path::{Path, PathBuf},
  sync::Arc,
  time::SystemTime,
};

use farmfe_core::{
  HashMap, HashSet,
  cache::module_cache::{
    CachedModule, CachedModuleDependency, CachedWatchDependency, MetadataOption,
  },
  context::{CachedEmittedResources, CompilationContext, create_swc_source_map},
  dashmap::DashMap,
  farm_profile_function,
  module::ModuleId,
  rayon::prelude::*,
  resource::{Resource, ResourceOrigin},
  swc_common::Globals,
};
use farmfe_toolkit::script::{
  analyze_statement::analyze_statements, swc_try_with::resolve_module_mark,
};

pub fn get_timestamp_of_module(module_id: &ModuleId, root: &str) -> u128 {
  farm_profile_function!(format!("get_timestamp_of_module: {:?}", module_id));
  let resolved_path = module_id.resolved_path(root);

  if !PathBuf::from(&resolved_path).exists() {
    // return unix epoch if the module is not found
    return SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
  }

  let file_meta = std::fs::metadata(resolved_path).unwrap_or_else(|_| {
    panic!(
      "Failed to get metadata of module {:?}",
      module_id.resolved_path(root)
    )
  });
  let system_time = file_meta.modified();

  if let Ok(system_time) = system_time
    && let Ok(dur) = system_time.duration_since(SystemTime::UNIX_EPOCH)
  {
    return dur.as_nanos();
  }

  SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_nanos()
}

#[cfg(test)]
mod watched_cache_tests {
  use super::*;
  use farmfe_core::{
    config::{
      Config,
      persistent_cache::{PersistentCacheConfig, PersistentCacheConfigObj},
    },
    module::Module,
  };

  fn context_with_changed_watch_dependency() -> (Arc<CompilationContext>, Module, PathBuf) {
    let root = std::env::temp_dir().join(format!(
      "farm-watched-cache-{}-{}",
      std::process::id(),
      SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("App.vue"), "new-class").unwrap();
    let context = Arc::new(
      CompilationContext::new(
        Config {
          root: root.to_string_lossy().replace('\\', "/"),
          persistent_cache: Box::new(PersistentCacheConfig::Obj(PersistentCacheConfigObj {
            memory: true,
            ..Default::default()
          })),
          ..Default::default()
        },
        vec![],
      )
      .unwrap(),
    );
    let mut module = Module::new("style.css".into());
    module.content_hash = "unchanged-style-input".into();
    module.last_update_timestamp = 7;
    context.cache_manager.module_cache.set_cache(
      module.id.clone(),
      CachedModule {
        module: module.clone(),
        dependencies: vec![],
        watch_dependencies: vec![CachedWatchDependency {
          dependency: "App.vue".into(),
          timestamp: 0,
          hash: get_content_hash_of_module("old-class"),
        }],
        is_expired: false,
      },
    );
    (context, module, root)
  }

  #[test]
  fn timestamp_watch_miss_expires_unchanged_css_source_cache() {
    let (context, module, root) = context_with_changed_watch_dependency();
    assert!(
      try_get_module_cache_by_timestamp(&module.id, module.last_update_timestamp, &context)
        .unwrap()
        .is_none()
    );
    assert!(!context.cache_manager.module_cache.has_cache(&module.id));
    drop(context);
    std::fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn hash_watch_miss_expires_unchanged_css_source_cache() {
    let (context, module, root) = context_with_changed_watch_dependency();
    assert!(
      try_get_module_cache_by_hash(&module.id, &module.content_hash, &context)
        .unwrap()
        .is_none()
    );
    assert!(!context.cache_manager.module_cache.has_cache(&module.id));
    drop(context);
    std::fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn unchanged_watch_content_can_still_use_hash_fallback_after_timestamp_change() {
    let (context, module, root) = context_with_changed_watch_dependency();
    {
      let mut cached = context
        .cache_manager
        .module_cache
        .get_cache_mut_option_ref(&module.id)
        .unwrap();
      cached.watch_dependencies[0].hash = get_content_hash_of_module("new-class");
    }
    assert!(
      try_get_module_cache_by_timestamp(&module.id, module.last_update_timestamp, &context)
        .unwrap()
        .is_none()
    );
    assert!(
      try_get_module_cache_by_hash(&module.id, &module.content_hash, &context)
        .unwrap()
        .is_some()
    );
    drop(context);
    std::fs::remove_dir_all(root).unwrap();
  }
}

pub fn get_content_hash_of_module(content: &str) -> String {
  farm_profile_function!("get_content_hash_of_module".to_string());

  let content = if content.is_empty() {
    "empty".to_string()
  } else {
    content.to_string()
  };

  farmfe_toolkit::hash::sha256(content.as_bytes(), 32)
}

/// Unsupported watch inputs deliberately have no legacy file-cache fingerprint.
/// Directories cannot be validated by their mtime: child content may change alone.
fn watch_dependency_fingerprint(path: &Path) -> Option<(u128, String)> {
  let before = std::fs::metadata(path).ok()?;
  if !before.is_file() {
    return None;
  }
  let timestamp = before
    .modified()
    .ok()?
    .duration_since(SystemTime::UNIX_EPOCH)
    .ok()?
    .as_nanos();
  let content = std::fs::read_to_string(path).ok()?;
  let after = std::fs::metadata(path).ok()?;
  if !after.is_file()
    || after
      .modified()
      .ok()?
      .duration_since(SystemTime::UNIX_EPOCH)
      .ok()?
      .as_nanos()
      != timestamp
  {
    return None;
  }
  Some((timestamp, get_content_hash_of_module(&content)))
}

fn cache_watch_dependencies(
  module: &ModuleId,
  context: &Arc<CompilationContext>,
) -> Option<Vec<CachedWatchDependency>> {
  context
    .watch_graph
    .read()
    .relation_dependencies(module)
    .into_iter()
    .map(|id| {
      let path = PathBuf::from(id.resolved_path(&context.config.root));
      let (timestamp, hash) = watch_dependency_fingerprint(&path)?;
      Some(CachedWatchDependency {
        dependency: if cfg!(windows) {
          id.to_string().into()
        } else {
          id.clone()
        },
        timestamp,
        hash,
      })
    })
    .collect()
}

// Diagnostic-only evidence of a fully accepted module cache hit, after watch
// dependency and plugin validation. No paths or source content are logged.
fn trace_cache_hit(strategy: &str) {
  if std::env::var("FARM_CACHE_DIAGNOSTICS").as_deref() == Ok("1") {
    eprintln!("[FARM_CACHE_DIAGNOSTICS] module-hit:{strategy}");
  }
}

pub fn try_get_module_cache_by_timestamp(
  module_id: &ModuleId,
  timestamp: u128,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<Option<CachedModule>> {
  farm_profile_function!(format!(
    "try_get_module_cache_by_timestamp: {:?}",
    module_id
  ));
  let mut should_invalidate_cache = false;

  if context.config.persistent_cache.timestamp_enabled()
    && context.cache_manager.module_cache.has_cache(module_id)
  {
    let cached_module = context.cache_manager.module_cache.get_cache_ref(module_id);

    if cached_module.value().module.last_update_timestamp == timestamp {
      drop(cached_module);
      let mut cached_module = context.cache_manager.module_cache.get_cache(module_id);
      handle_cached_modules(&mut cached_module, context)?;

      if !is_watch_dependencies_timestamp_changed(&cached_module, context) {
        let should_invalidate_cached_module = context
          .plugin_driver
          .handle_persistent_cached_module(&cached_module.module, context)?
          .unwrap_or(false);

        if !should_invalidate_cached_module {
          trace_cache_hit("timestamp");
          return Ok(Some(cached_module));
        } else {
          should_invalidate_cache = true;
        }
      } else {
        // A timestamp-only change may still use the content-hash cache path.
        should_invalidate_cache = !context.config.persistent_cache.hash_enabled()
          || is_watch_dependencies_content_hash_changed(&cached_module, context);
      }
    } else if !context.config.persistent_cache.hash_enabled() {
      should_invalidate_cache = true;
    }
  }

  if should_invalidate_cache {
    context
      .cache_manager
      .module_cache
      .invalidate_cache(module_id);
  }

  Ok(None)
}

pub fn try_get_module_cache_by_hash(
  module_id: &ModuleId,
  content_hash: &str,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<Option<CachedModule>> {
  farm_profile_function!(format!("try_get_module_cache_by_hash: {:?}", module_id));

  let mut should_invalidate_cache = false;

  if context.config.persistent_cache.hash_enabled()
    && context.cache_manager.module_cache.has_cache(module_id)
  {
    let cached_module = context.cache_manager.module_cache.get_cache_ref(module_id);

    if cached_module.value().module.content_hash == content_hash {
      drop(cached_module);
      let mut cached_module = context.cache_manager.module_cache.get_cache(module_id);

      handle_cached_modules(&mut cached_module, context)?;

      if !is_watch_dependencies_content_hash_changed(&cached_module, context) {
        let should_invalidate_cached_module = context
          .plugin_driver
          .handle_persistent_cached_module(&cached_module.module, context)?
          .unwrap_or(false);

        if !should_invalidate_cached_module {
          trace_cache_hit("hash");
          return Ok(Some(cached_module));
        } else {
          should_invalidate_cache = true;
        }
      } else {
        should_invalidate_cache = true;
      }
    } else {
      should_invalidate_cache = true;
    }
  }

  if should_invalidate_cache {
    context
      .cache_manager
      .module_cache
      .invalidate_cache(module_id);
  }

  Ok(None)
}

pub fn set_module_graph_cache(module_ids: Vec<ModuleId>, context: &Arc<CompilationContext>) {
  set_module_graph_cache_with_forced_updates(module_ids, &HashSet::default(), context);
}

pub(crate) fn set_module_graph_cache_with_forced_updates(
  module_ids: Vec<ModuleId>,
  forced_updates: &HashSet<ModuleId>,
  context: &Arc<CompilationContext>,
) {
  farm_profile_function!("set_module_graph_cache".to_string());
  let module_graph = context.module_graph.read();
  let mut cacheable_modules = HashSet::default();

  let modules = module_ids
    .iter()
    .map(|id| module_graph.module(id).unwrap())
    .filter(|m| !m.external)
    .collect::<Vec<_>>();

  for module in &modules {
    // skip cache unchanged modules if following conditions are met:
    // 1. There is no query string in the module id when timestamp enabled
    // 2. if the module has already in the cache and the cache hash not changed
    // Note: For condition 1, here is the case: `index.vue` imports virtual modules like `index.vue?vue&setup&lang.tsx`,
    // when content of `index.vue` is changed, the content hash of `index.vue?vue&setup&lang.tsx` may not be changed,
    // but the cached timestamp of `index.vue?vue&setup&lang.tsx` should be updated to make sure the cache is always up-to-date.
    // otherwise, cache of `index.vue` is used but cache of `index.vue?vue&setup&lang.tsx` is not used, which would cause a bug like transform error.
    if !forced_updates.contains(&module.id)
      && (!context.config.persistent_cache.timestamp_enabled()
        || module.id.query_string().is_empty())
      && context
        .watch_graph
        .read()
        .relation_dependencies(&module.id)
        .is_empty()
      && !context.cache_manager.module_cache.is_cache_changed(module)
    {
      continue;
    }

    cacheable_modules.insert(module.id.clone());
  }

  let cached_dependency_map = DashMap::new();

  for module_id in &cacheable_modules {
    let dependencies = module_graph
      .dependencies(module_id)
      .into_iter()
      .map(|(id, e)| CachedModuleDependency {
        dependency: id,
        edge_info: e.clone(),
      })
      .collect();
    cached_dependency_map.insert(module_id.clone(), dependencies);
  }

  modules
    .into_par_iter()
    .filter(|module| cacheable_modules.contains(&module.id))
    .for_each(|module| {
      let Some(watch_dependencies) = cache_watch_dependencies(&module.id, context) else {
        // Missing/unsupported proof must also invalidate an older cached entry.
        context
          .cache_manager
          .module_cache
          .invalidate_cache(&module.id);
        return;
      };
      // Replace the module with a placeholder module to prevent the module from being cloned
      let cloned_module = module.clone();
      let dependencies = cached_dependency_map
        .remove(&module.id)
        .unwrap_or_else(|| panic!("module {:?} not found in cached_dependency_map", module.id))
        .1;

      let cached_module = CachedModule {
        module: cloned_module,
        dependencies,
        watch_dependencies,
        is_expired: false,
      };

      if forced_updates.contains(&module.id) {
        context
          .cache_manager
          .module_cache
          .set_updated_cache(module.id.clone(), cached_module);
      } else {
        context
          .cache_manager
          .module_cache
          .set_cache(module.id.clone(), cached_module);
      }
    });
}

/// recreate syntax context for the cached module
pub fn handle_cached_modules(
  cached_module: &mut CachedModule,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  // create a new sourcemap for the cached module cause the sourcemap of swc is not cacheable
  let (source_map, _) = create_swc_source_map(
    &cached_module.module.id,
    cached_module.module.content.clone(),
  );
  context
    .meta
    .set_module_source_map(&cached_module.module.id, source_map);
  context
    .meta
    .set_globals(&cached_module.module.id, Globals::new());

  // using swc resolver
  match &mut cached_module.module.meta {
    box farmfe_core::module::ModuleMetaData::Script(script) => {
      // reset the mark to prevent the mark from being reused
      let (unresolved_mark, top_level_mark) = resolve_module_mark(
        &mut script.ast,
        cached_module.module.module_type.is_typescript(),
        context.meta.get_globals(&cached_module.module.id).value(),
      );

      script.top_level_mark = top_level_mark.as_u32();
      script.unresolved_mark = unresolved_mark.as_u32();
      script.statements = analyze_statements(&script.ast);
    }
    box farmfe_core::module::ModuleMetaData::Css(_)
    | box farmfe_core::module::ModuleMetaData::Html(_) => { /* do nothing */ }
    box farmfe_core::module::ModuleMetaData::Custom(_) => { /* TODO: add a hook for custom module */
    }
  };

  handle_relation_roots(
    &cached_module.module.id,
    &cached_module.watch_dependencies,
    context,
  )?;

  // clear module groups and resource pot as it will be re-resolved later
  cached_module.module.module_groups.clear();
  cached_module.module.resource_pots = Default::default();
  cached_module.module.used_exports.clear();

  let options = || Some(MetadataOption::default().refer(vec![cached_module.module.id.clone()]));
  let resources = if let Some(cached) =
    context.read_metadata::<CachedEmittedResources>("builtin:emit_files", options())
  {
    cached.resources
  } else {
    // Read caches written by versions that supported only one emitted file.
    context
      .read_metadata::<Resource>("builtin:emit_file", options())
      .map(|resource| vec![*resource])
      .unwrap_or_default()
  };
  let mut resource_map = context.resources_map.lock();
  for resource in resources {
    if matches!(resource.origin, ResourceOrigin::Module(_)) {
      resource_map.insert(resource.name.clone(), resource);
    }
  }

  // TODO: return of resolve hook should be treated as part of the cache key
  Ok(())
}

/// recreate the watch graph for the cached module
fn handle_relation_roots(
  cached_module_id: &ModuleId,
  watch_dependencies: &Vec<CachedWatchDependency>,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  if !watch_dependencies.is_empty() {
    let mut watch_graph = context.watch_graph.write();
    watch_graph.add_node(cached_module_id.clone());

    for cached_dep in watch_dependencies {
      let dep = &cached_dep.dependency;
      watch_graph.add_node(dep.clone());
      watch_graph.add_edge(cached_module_id, dep)?;
    }
  }

  Ok(())
}

fn is_watch_dependencies_timestamp_changed(
  cached_module: &CachedModule,
  context: &Arc<CompilationContext>,
) -> bool {
  farm_profile_function!(format!(
    "is_watch_dependencies_timestamp_changed: {:?}",
    cached_module.module.id
  ));
  let watch_graph = context.watch_graph.read();
  let relation_dependencies = watch_graph.relation_dependencies(&cached_module.module.id);

  if relation_dependencies.is_empty() {
    return false;
  }

  let cached = cached_module
    .watch_dependencies
    .iter()
    .map(|dep| (&dep.dependency, dep))
    .collect::<HashMap<_, _>>();
  for dep in relation_dependencies {
    let Some(previous) = cached.get(dep) else {
      return true;
    };
    let path = PathBuf::from(dep.resolved_path(&context.config.root));
    let Some((timestamp, hash)) = watch_dependency_fingerprint(&path) else {
      return true;
    };
    if timestamp != previous.timestamp || hash != previous.hash {
      return true;
    }
  }

  false
}

fn is_watch_dependencies_content_hash_changed(
  cached_module: &CachedModule,
  context: &Arc<CompilationContext>,
) -> bool {
  farm_profile_function!(format!(
    "is_watch_dependencies_content_hash_changed: {:?}",
    cached_module.module.id
  ));
  let watch_graph = context.watch_graph.read();
  let relation_dependencies = watch_graph.relation_dependencies(&cached_module.module.id);

  if relation_dependencies.is_empty() {
    return false;
  }

  let cached = cached_module
    .watch_dependencies
    .iter()
    .map(|dep| (&dep.dependency, dep))
    .collect::<HashMap<_, _>>();
  for dep in relation_dependencies {
    let Some(previous) = cached.get(dep) else {
      return true;
    };
    let path = PathBuf::from(dep.resolved_path(&context.config.root));
    let Some((_, hash)) = watch_dependency_fingerprint(&path) else {
      return true;
    };
    if hash != previous.hash {
      return true;
    }
  }

  false
}

#[cfg(test)]
mod watch_cache_tests {
  use super::*;
  use farmfe_core::{
    config::{Config, persistent_cache::PersistentCacheConfig},
    module::Module,
  };

  struct Fixture(PathBuf);
  impl Fixture {
    fn new() -> Self {
      let root = std::env::temp_dir().join(format!(
        "farm-watch-cache-{}-{}",
        std::process::id(),
        SystemTime::now()
          .duration_since(SystemTime::UNIX_EPOCH)
          .unwrap()
          .as_nanos()
      ));
      std::fs::create_dir(&root).unwrap();
      Self(root)
    }
  }
  impl Drop for Fixture {
    fn drop(&mut self) {
      let _ = std::fs::remove_dir_all(&self.0);
    }
  }
  fn context(root: &Path) -> Arc<CompilationContext> {
    let mut config = Config {
      root: root.to_string_lossy().into_owned(),
      ..Default::default()
    };
    let mut cache = PersistentCacheConfig::get_default_config(&config.root)
      .as_raw_object()
      .clone();
    cache.memory = true;
    config.persistent_cache = Box::new(PersistentCacheConfig::Obj(cache));
    Arc::new(CompilationContext::new(config, vec![]).unwrap())
  }
  fn add_module(context: &Arc<CompilationContext>, watch: &Path) -> ModuleId {
    let id = ModuleId::new("entry.js", "", &context.config.root);
    let mut module = Module::new(id.clone());
    module.content_hash = "owner-hash".into();
    module.last_update_timestamp = 123;
    // Even immutable owners cannot bypass a mutable or unsupported watch.
    module.immutable = true;
    context.module_graph.write().add_module(module);
    let dep = ModuleId::new(watch.to_str().unwrap(), "", &context.config.root);
    context.add_watch_files(id.clone(), vec![dep]).unwrap();
    id
  }
  fn assert_restore_misses(
    context: &Arc<CompilationContext>,
    id: &ModuleId,
    original: &CachedModule,
  ) {
    context
      .cache_manager
      .module_cache
      .set_updated_cache(id.clone(), original.clone());
    assert!(
      try_get_module_cache_by_timestamp(id, 123, context)
        .unwrap()
        .is_none()
    );
    assert!(!context.cache_manager.module_cache.has_cache(id));
    context
      .cache_manager
      .module_cache
      .set_updated_cache(id.clone(), original.clone());
    assert!(
      try_get_module_cache_by_hash(id, "owner-hash", context)
        .unwrap()
        .is_none()
    );
    assert!(!context.cache_manager.module_cache.has_cache(id));
  }

  #[test]
  fn unsupported_watches_are_not_written_or_restored_as_unchanged() {
    for kind in ["directory", "missing", "binary"] {
      let fixture = Fixture::new();
      let watch = fixture.0.join("watched");
      match kind {
        "directory" => std::fs::create_dir(&watch).unwrap(),
        "binary" => std::fs::write(&watch, [0xff, 0xfe]).unwrap(),
        _ => (),
      }
      assert!(watch_dependency_fingerprint(&watch).is_none());
      let context = context(&fixture.0);
      let id = add_module(&context, &watch);
      set_module_graph_cache_with_forced_updates(
        vec![id.clone()],
        &[id.clone()].into_iter().collect::<HashSet<_>>(),
        &context,
      );
      assert!(!context.cache_manager.module_cache.has_cache(&id), "{kind}");
      // An older writer's empty/mtime entry must not be accepted by either strategy.
      let old = CachedModule {
        module: context.module_graph.read().module(&id).unwrap().clone(),
        dependencies: vec![],
        watch_dependencies: vec![CachedWatchDependency {
          dependency: ModuleId::new(watch.to_str().unwrap(), "", &context.config.root),
          timestamp: get_timestamp_of_module(
            &ModuleId::new(watch.to_str().unwrap(), "", &context.config.root),
            &context.config.root,
          ),
          hash: get_content_hash_of_module("empty"),
        }],
        is_expired: false,
      };
      assert_restore_misses(&context, &id, &old);
    }
  }

  #[test]
  fn regular_watch_hashes_are_compatible_and_content_changes_invalidate_both_paths() {
    let fixture = Fixture::new();
    let watch = fixture.0.join("watched.txt");
    std::fs::write(&watch, "original").unwrap();
    let original_modified = std::fs::metadata(&watch).unwrap().modified().unwrap();
    let context = context(&fixture.0);
    let id = add_module(&context, &watch);
    set_module_graph_cache_with_forced_updates(
      vec![id.clone()],
      &[id.clone()].into_iter().collect::<HashSet<_>>(),
      &context,
    );
    assert!(context.cache_manager.module_cache.has_cache(&id));
    let old = context
      .cache_manager
      .module_cache
      .get_cache_ref(&id)
      .value()
      .clone();
    assert_eq!(
      old.watch_dependencies[0].hash,
      get_content_hash_of_module("original")
    );
    assert!(
      try_get_module_cache_by_timestamp(&id, 123, &context)
        .unwrap()
        .is_some()
    );
    assert!(
      try_get_module_cache_by_hash(&id, "owner-hash", &context)
        .unwrap()
        .is_some()
    );
    std::fs::write(&watch, "modified").unwrap();
    // Directory/main-module timestamps do not prove watched content unchanged.
    let file = std::fs::OpenOptions::new()
      .write(true)
      .open(&watch)
      .unwrap();
    file
      .set_times(std::fs::FileTimes::new().set_modified(original_modified))
      .unwrap();
    drop(file);
    assert_restore_misses(&context, &id, &old);
    std::fs::remove_file(&watch).unwrap();
    std::fs::create_dir(&watch).unwrap();
    assert_restore_misses(&context, &id, &old);
  }

  #[cfg(windows)]
  #[test]
  fn unreadable_locked_watch_misses_without_panicking() {
    use std::os::windows::fs::OpenOptionsExt;
    let fixture = Fixture::new();
    let watch = fixture.0.join("locked.txt");
    std::fs::write(&watch, "original").unwrap();
    let context = context(&fixture.0);
    let id = add_module(&context, &watch);
    set_module_graph_cache_with_forced_updates(
      vec![id.clone()],
      &[id.clone()].into_iter().collect::<HashSet<_>>(),
      &context,
    );
    let old = context
      .cache_manager
      .module_cache
      .get_cache_ref(&id)
      .value()
      .clone();
    let locked = std::fs::OpenOptions::new()
      .read(true)
      .write(true)
      .share_mode(0)
      .open(&watch)
      .unwrap();
    assert!(watch_dependency_fingerprint(&watch).is_none());
    assert_restore_misses(&context, &id, &old);
    set_module_graph_cache_with_forced_updates(
      vec![id.clone()],
      &[id.clone()].into_iter().collect::<HashSet<_>>(),
      &context,
    );
    assert!(!context.cache_manager.module_cache.has_cache(&id));
    drop(locked);
  }
}
