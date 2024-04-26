use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
  sync::Arc,
  time::SystemTime,
};

use farmfe_core::{
  cache::module_cache::{CachedModule, CachedModuleDependency, CachedWatchDependency},
  context::CompilationContext,
  dashmap::DashMap,
  farm_profile_function,
  module::ModuleId,
  rayon::prelude::*,
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

  if let Ok(system_time) = system_time {
    if let Ok(dur) = system_time.duration_since(SystemTime::UNIX_EPOCH) {
      return dur.as_nanos();
    }
  }

  SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_nanos()
}

pub fn get_content_hash_of_module(content: &str) -> String {
  farm_profile_function!("get_content_hash_of_module".to_string());

  let content = if content.is_empty() {
    "empty".to_string()
  } else {
    content.to_string()
  };

  let module_content_hash = farmfe_toolkit::hash::sha256(content.as_bytes(), 32);
  module_content_hash
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

      if cached_module.module.immutable
        || !is_watch_dependencies_timestamp_changed(&cached_module, context)
      {
        let should_invalidate_cached_module = context
          .plugin_driver
          .handle_persistent_cached_module(&cached_module.module, context)?
          .unwrap_or(false);

        if !should_invalidate_cached_module {
          return Ok(Some(cached_module));
        } else {
          should_invalidate_cache = true;
        }
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

      if cached_module.module.immutable
        || !is_watch_dependencies_content_hash_changed(&cached_module, context)
      {
        let should_invalidate_cached_module = context
          .plugin_driver
          .handle_persistent_cached_module(&cached_module.module, context)?
          .unwrap_or(false);

        if !should_invalidate_cached_module {
          return Ok(Some(cached_module));
        } else {
          should_invalidate_cache = true;
        }
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
  farm_profile_function!("set_module_graph_cache".to_string());
  let module_graph = context.module_graph.read();
  let mut cacheable_modules = HashSet::new();

  let modules = module_ids
    .iter()
    .map(|id| module_graph.module(id).unwrap())
    .filter(|m| !m.external)
    .collect::<Vec<_>>();

  for module in &modules {
    // if the module has already in the cache and not changed, skip it.
    if !context.cache_manager.module_cache.is_cache_changed(module) {
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
      // Replace the module with a placeholder module to prevent the module from being cloned
      let cloned_module = module.clone();
      let dependencies = cached_dependency_map
        .remove(&module.id)
        .unwrap_or_else(|| panic!("module {:?} not found in cached_dependency_map", module.id))
        .1;

      let cached_module = CachedModule {
        module: cloned_module,
        dependencies,
        watch_dependencies: context
          .watch_graph
          .read()
          .relation_dependencies(&module.id)
          .into_iter()
          .map(|id| {
            let resolved_path = PathBuf::from(id.resolved_path(&context.config.root));
            let content = if resolved_path.exists() {
              std::fs::read_to_string(resolved_path).unwrap()
            } else {
              // treat the virtual module as always unchanged for now
              "empty".to_string()
            };
            CachedWatchDependency {
              dependency: id.clone(),
              timestamp: get_timestamp_of_module(id, &context.config.root),
              hash: get_content_hash_of_module(&content),
            }
          })
          .collect(),
      };

      context
        .cache_manager
        .module_cache
        .set_cache(module.id.clone(), cached_module);
    });
}

/// recreate syntax context for the cached module
pub fn handle_cached_modules(
  cached_module: &mut CachedModule,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  // using swc resolver
  match &mut cached_module.module.meta {
    box farmfe_core::module::ModuleMetaData::Script(script) => {
      // reset the mark to prevent the mark from being reused, it will be re-resolved later
      script.top_level_mark = 0;
      script.unresolved_mark = 0;
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

  Ok(())
}

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

  let cached_dep_timestamp_map = cached_module
    .watch_dependencies
    .iter()
    .map(|dep| (dep.dependency.clone(), dep.timestamp))
    .collect::<HashMap<_, _>>();

  for dep in &relation_dependencies {
    let resolved_path = PathBuf::from(dep.resolved_path(&context.config.root));
    let cached_timestamp = cached_dep_timestamp_map.get(dep);

    if !resolved_path.exists()
      || cached_timestamp.is_none()
      || get_timestamp_of_module(dep, &context.config.root) != *cached_timestamp.unwrap()
    {
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

  let cached_dep_hash_map = cached_module
    .watch_dependencies
    .iter()
    .map(|dep| (dep.dependency.clone(), dep.hash.clone()))
    .collect::<HashMap<_, _>>();

  for dep in relation_dependencies {
    let resolved_path = PathBuf::from(dep.resolved_path(&context.config.root));
    let cached_hash = cached_dep_hash_map.get(dep);

    if !resolved_path.exists() || cached_hash.is_none() {
      return true;
    }

    let content = std::fs::read_to_string(resolved_path).unwrap();
    let hash = get_content_hash_of_module(&content);

    if hash != *cached_hash.unwrap() {
      return true;
    }
  }

  false
}
