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

use farmfe_toolkit::resolve::load_package_json;

pub fn get_timestamp_of_module(module_id: &ModuleId, root: &str) -> u128 {
  let resolved_path = module_id.resolved_path(root);

  if !PathBuf::from(&resolved_path).exists() {
    return std::time::Instant::now().elapsed().as_nanos();
  }

  let file_meta = std::fs::metadata(resolved_path).unwrap_or_else(|_| {
    panic!(
      "Failed to get metadata of module {:?}",
      module_id.resolved_path(root)
    )
  });
  file_meta
    .modified()
    .unwrap()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_nanos()
}

pub fn get_content_hash_of_module(content: &str) -> String {
  let module_content_hash = farmfe_toolkit::hash::sha256(content.as_bytes(), 32);
  module_content_hash
}

pub fn try_get_module_cache_by_timestamp(
  module_id: &ModuleId,
  timestamp: u128,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<Option<CachedModule>> {
  if context.config.persistent_cache.timestamp_enabled()
    && context.cache_manager.module_cache.has_cache(module_id)
  {
    let cached_module = context.cache_manager.module_cache.get_cache_ref(module_id);

    if cached_module.value().module.last_update_timestamp == timestamp {
      let mut cached_module = context.cache_manager.module_cache.get_cache(module_id);
      handle_cached_modules(&mut cached_module, context)?;

      if cached_module.module.immutable
        || !is_watch_dependencies_timestamp_changed(&cached_module, context)
      {
        return Ok(Some(cached_module));
      }
    } else if !context.config.persistent_cache.hash_enabled() {
      drop(cached_module);
      context
        .cache_manager
        .module_cache
        .invalidate_cache(module_id);
    }
  }

  // println!(
  //   "module not found: {:?} timestamp enabled {}, has_cache {}",
  //   module_id,
  //   context.config.persistent_cache.timestamp_enabled(),
  //   context.cache_manager.module_cache.has_cache(module_id)
  // );

  Ok(None)
}

pub fn try_get_module_cache_by_hash(
  module_id: &ModuleId,
  content_hash: &str,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<Option<CachedModule>> {
  if context.config.persistent_cache.hash_enabled()
    && context.cache_manager.module_cache.has_cache(module_id)
  {
    let cached_module = context.cache_manager.module_cache.get_cache_ref(module_id);

    if cached_module.value().module.content_hash == content_hash {
      let mut cached_module = context.cache_manager.module_cache.get_cache(module_id);

      handle_cached_modules(&mut cached_module, context)?;

      if cached_module.module.immutable
        || !is_watch_dependencies_content_hash_changed(&cached_module, context)
      {
        return Ok(Some(cached_module));
      }
    } else {
      drop(cached_module);

      context
        .cache_manager
        .module_cache
        .invalidate_cache(module_id);
    }
  }

  Ok(None)
}

pub fn set_module_graph_cache(
  module_ids: Vec<ModuleId>,
  check_initial_cache: bool,
  context: &Arc<CompilationContext>,
) {
  farm_profile_function!("set_module_graph_cache".to_string());
  let module_graph = context.module_graph.read();
  let mut cacheable_modules = HashSet::new();

  let modules = module_ids
    .iter()
    .map(|id| module_graph.module(id).unwrap())
    .filter(|m| !m.external)
    .collect::<Vec<_>>();

  for module in &modules {
    // if the module has already in the cache, skip it.
    if check_initial_cache && context.cache_manager.module_cache.has_cache(&module.id) {
      let cached_ref = context.cache_manager.module_cache.get_cache_ref(&module.id);

      if cached_ref.value().module.content_hash == module.content_hash {
        continue;
      }
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
      let dependencies = cached_dependency_map.remove(&module.id).unwrap().1;

      let resolved_path = module.id.resolved_path(&context.config.root);
      let package_info =
        load_package_json(PathBuf::from(resolved_path), Default::default()).unwrap_or_default();
      let cached_module = CachedModule {
        module: cloned_module,
        dependencies,
        package_name: package_info.name.unwrap_or("default".to_string()),
        package_version: package_info.version.unwrap_or("0.0.0".to_string()),
        entry_name: module_graph.entries.get(&module.id).cloned(),
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
              // treat the virtual module as always changed for now
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

// fn load_module_cache_into_context(
//   cached_module_id: &ModuleId,
//   visited: &mut HashMap<ModuleId, bool>,
//   context: &Arc<CompilationContext>,
// ) -> bool {
//   if visited.contains_key(cached_module_id) {
//     return visited[cached_module_id];
//   }

//   if !context
//     .cache_manager
//     .module_cache
//     .has_cache(cached_module_id)
//   {
//     visited.insert(cached_module_id.clone(), false);
//     return false;
//   }
//   // if cycle detected, return true to skip this module
//   visited.insert(cached_module_id.clone(), true);

//   let cached_module = context
//     .cache_manager
//     .module_cache
//     .get_cache_ref(cached_module_id);

//   if !cached_module.module.immutable {
//     visited.insert(cached_module_id.clone(), false);
//     return false;
//   }
//   for dep in &cached_module.dependencies {
//     if !load_module_cache_into_context(&dep.dependency, visited, context) {
//       visited.insert(cached_module_id.clone(), false);
//       return false;
//     }
//   }

//   visited.insert(cached_module_id.clone(), true);

//   true
// }

// /// Load module graph cache to context.
// /// All immutable modules and all of its immutable dependencies will be loaded into context.module_graph
// pub fn load_module_graph_cache_into_context(
//   context: &Arc<CompilationContext>,
// ) -> farmfe_core::error::Result<()> {
//   farm_profile_function!("load_module_graph_cache_into_context".to_string());
//   // TODO load immutable dependencies into context in need
//   let immutable_modules = context.cache_manager.module_cache.get_immutable_modules();

//   immutable_modules
//     .par_iter()
//     .try_for_each(|cached_module_id| {
//       let mut cached_module = context
//         .cache_manager
//         .module_cache
//         .get_cache_mut_ref(cached_module_id);

//       handle_cached_modules(cached_module.value_mut(), context)
//     })?;

//   let mut visited = HashMap::new();

//   immutable_modules.iter().for_each(|cached_module_id| {
//     load_module_cache_into_context(cached_module_id, &mut visited, context);
//   });

//   let mut module_graph = context.module_graph.write();
//   let mut edges = vec![];

//   visited
//     .into_iter()
//     .try_for_each(|(cached_module_id, is_cached)| {
//       if is_cached {
//         let cached_module = context
//           .cache_manager
//           .module_cache
//           .get_cache(&cached_module_id);

//         module_graph.add_module(cached_module.module);
//         edges.push((cached_module_id.clone(), cached_module.dependencies));

//         if let Some(entry_name) = cached_module.entry_name {
//           module_graph
//             .entries
//             .insert(cached_module_id.clone(), entry_name);
//         }
//       }

//       Ok(())
//     })?;

//   for (from, edges) in edges {
//     {
//       for edge in edges {
//         module_graph
//           .add_edge(&from, &edge.dependency, edge.edge_info)
//           .unwrap();
//       }
//     }
//   }

//   Ok(())
// }

/// recreate syntax context for the cached module
pub fn handle_cached_modules(
  cached_module: &mut CachedModule,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  // using swc resolver
  match &mut cached_module.module.meta {
    farmfe_core::module::ModuleMetaData::Script(script) => {
      // reset the mark to prevent the mark from being reused, it will be re-resolved later
      script.top_level_mark = 0;
      script.unresolved_mark = 0;
    }
    farmfe_core::module::ModuleMetaData::Css(_) | farmfe_core::module::ModuleMetaData::Html(_) => { /* do nothing */
    }
    farmfe_core::module::ModuleMetaData::Custom(_) => { /* TODO: add a hook for custom module */ }
  };

  handle_relation_roots(
    &cached_module.module.id,
    &cached_module.watch_dependencies,
    context,
  )?;

  Ok(())
}

// pub fn clear_unused_cached_modules(context: &Arc<CompilationContext>) {
//   farm_profile_function!("clear_unused_cached_modules".to_string());
//   let mut module_graph = context.module_graph.write();
//   clear_unused_cached_modules_from_module_graph(&mut module_graph);
// }

// fn clear_unused_cached_modules_from_module_graph(module_graph: &mut ModuleGraph) {
//   let mut removed_modules = HashSet::new();

//   // module that does not belong to any ModuleGroup will be removed
//   for module in module_graph.modules() {
//     if !module_graph.entries.contains_key(&module.id)
//       && module_graph.dependents_ids(&module.id).is_empty()
//     {
//       removed_modules.insert(module.id.clone());
//     }
//   }

//   let mut removed_modules_vec = removed_modules.iter().cloned().collect::<VecDeque<_>>();

//   while !removed_modules_vec.is_empty() {
//     let removed_module = removed_modules_vec.pop_front().unwrap();
//     let dependencies = module_graph.dependencies_ids(&removed_module);

//     for dep in dependencies {
//       let dependents = module_graph.dependents_ids(&dep);

//       if dependents.iter().all(|dept| removed_modules.contains(dept)) {
//         removed_modules_vec.push_back(dep.clone());
//         removed_modules.insert(dep);
//       }
//     }
//   }

//   for removed_module in removed_modules {
//     module_graph.remove_module(&removed_module);
//   }
// }

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
  let watch_graph = context.watch_graph.read();
  let relation_dependencies = watch_graph.relation_dependencies(&cached_module.module.id);

  if relation_dependencies.is_empty() {
    return false;
  }

  // println!(
  //   "{:?} relation_dependencies: {:?}",
  //   cached_module.module.id, relation_dependencies
  // );

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

// #[cfg(test)]
// mod tests {
//   use farmfe_testing_helpers::construct_test_module_graph_complex;

//   use super::clear_unused_cached_modules_from_module_graph;

//   #[test]
//   fn test_clear_unused_cached_modules_from_module_graph() {
//     let mut module_graph = construct_test_module_graph_complex();
//     module_graph.remove_edge(&"B".into(), &"E".into()).unwrap();

//     clear_unused_cached_modules_from_module_graph(&mut module_graph);
//     assert!(!module_graph.has_module(&"E".into()));
//   }
// }
