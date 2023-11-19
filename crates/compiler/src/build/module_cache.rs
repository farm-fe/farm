use std::{
  collections::{HashMap, HashSet, VecDeque},
  path::PathBuf,
  sync::Arc,
  time::SystemTime,
};

use farmfe_core::{
  cache::module_cache::{CachedModule, CachedModuleDependency},
  context::CompilationContext,
  dashmap::DashMap,
  farm_profile_function,
  module::{module_graph::ModuleGraph, ModuleId},
  rayon::prelude::*,
  swc_common::{Mark, Span, SyntaxContext, GLOBALS},
};

use farmfe_toolkit::{
  resolve::load_package_json,
  swc_ecma_transforms_base::resolver,
  swc_ecma_visit::{VisitMut, VisitMutWith},
};

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

      return Ok(Some(cached_module));
    } else if !context.config.persistent_cache.hash_enabled() {
      drop(cached_module);
      context
        .cache_manager
        .module_cache
        .invalidate_cache(module_id);
    }
  }

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

      return Ok(Some(cached_module));
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

pub struct ResetSpanVisitMut;

impl VisitMut for ResetSpanVisitMut {
  fn visit_mut_span(&mut self, span: &mut Span) {
    span.ctxt = SyntaxContext::empty();
  }
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
      };

      context
        .cache_manager
        .module_cache
        .set_cache(module.id.clone(), cached_module);
    });
}

fn load_module_cache_into_context(
  cached_module_id: &ModuleId,
  visited: &mut HashMap<ModuleId, bool>,
  context: &Arc<CompilationContext>,
) -> bool {
  if visited.contains_key(cached_module_id) {
    return visited[cached_module_id];
  }

  if !context
    .cache_manager
    .module_cache
    .has_cache(cached_module_id)
  {
    visited.insert(cached_module_id.clone(), false);
    return false;
  }
  // if cycle detected, return true to skip this module
  visited.insert(cached_module_id.clone(), true);

  let cached_module = context
    .cache_manager
    .module_cache
    .get_cache_ref(cached_module_id);

  if !cached_module.module.immutable {
    visited.insert(cached_module_id.clone(), false);
    return false;
  }
  for dep in &cached_module.dependencies {
    if !load_module_cache_into_context(&dep.dependency, visited, context) {
      visited.insert(cached_module_id.clone(), false);
      return false;
    }
  }

  visited.insert(cached_module_id.clone(), true);

  true
}

/// Load module graph cache to context.
/// All immutable modules and all of its immutable dependencies will be loaded into context.module_graph
pub fn load_module_graph_cache_into_context(
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  farm_profile_function!("load_module_graph_cache_into_context".to_string());

  let immutable_modules = context.cache_manager.module_cache.get_immutable_modules();

  immutable_modules
    .par_iter()
    .try_for_each(|cached_module_id| {
      let mut cached_module = context
        .cache_manager
        .module_cache
        .get_cache_mut_ref(cached_module_id);

      handle_cached_modules(cached_module.value_mut(), context)
    })?;

  let mut visited = HashMap::new();

  immutable_modules.iter().for_each(|cached_module_id| {
    load_module_cache_into_context(cached_module_id, &mut visited, context);
  });

  let mut module_graph = context.module_graph.write();
  let mut edges = vec![];

  visited
    .into_iter()
    .for_each(|(cached_module_id, is_cached)| {
      if is_cached {
        let cached_module = context
          .cache_manager
          .module_cache
          .get_cache(&cached_module_id);

        module_graph.add_module(cached_module.module);
        edges.push((cached_module_id, cached_module.dependencies));
      }
    });

  for (from, edges) in edges {
    {
      for edge in edges {
        module_graph
          .add_edge(&from, &edge.dependency, edge.edge_info)
          .unwrap();
      }
    }
  }

  Ok(())
}

/// recreate syntax context for the cached module
fn handle_cached_modules(
  cached_module: &mut CachedModule,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  // using swc resolver
  match &mut cached_module.module.meta {
    farmfe_core::module::ModuleMetaData::Script(script) => {
      GLOBALS.set(&context.meta.script.globals, || {
        let ast = &mut script.ast;
        // clear ctxt
        ast.visit_mut_with(&mut ResetSpanVisitMut);

        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        ast.visit_mut_with(&mut resolver(
          unresolved_mark,
          top_level_mark,
          cached_module.module.module_type.is_typescript(),
        ));

        script.top_level_mark = top_level_mark.as_u32();
        script.unresolved_mark = unresolved_mark.as_u32();
      });
    }
    farmfe_core::module::ModuleMetaData::Css(_) | farmfe_core::module::ModuleMetaData::Html(_) => { /* do nothing */
    }
    farmfe_core::module::ModuleMetaData::Custom(_) => { /* TODO: add a hook for custom module */ }
  };

  Ok(())
}

pub fn clear_unused_cached_modules(context: &Arc<CompilationContext>) {
  farm_profile_function!("clear_unused_cached_modules".to_string());
  let mut module_graph = context.module_graph.write();
  clear_unused_cached_modules_from_module_graph(&mut module_graph);
}

fn clear_unused_cached_modules_from_module_graph(module_graph: &mut ModuleGraph) {
  let mut removed_modules = HashSet::new();

  // module that does not belong to any ModuleGroup will be removed
  for module in module_graph.modules() {
    if !module_graph.entries.contains_key(&module.id)
      && module_graph.dependents_ids(&module.id).is_empty()
    {
      removed_modules.insert(module.id.clone());
    }
  }

  let mut removed_modules_vec = removed_modules.iter().cloned().collect::<VecDeque<_>>();

  while !removed_modules_vec.is_empty() {
    let removed_module = removed_modules_vec.pop_front().unwrap();
    let dependencies = module_graph.dependencies_ids(&removed_module);

    for dep in dependencies {
      let dependents = module_graph.dependents_ids(&dep);

      if dependents.iter().all(|dept| removed_modules.contains(dept)) {
        removed_modules_vec.push_back(dep.clone());
        removed_modules.insert(dep);
      }
    }
  }

  for removed_module in removed_modules {
    module_graph.remove_module(&removed_module);
  }
}

#[cfg(test)]
mod tests {
  use farmfe_testing_helpers::construct_test_module_graph_complex;

  use super::clear_unused_cached_modules_from_module_graph;

  #[test]
  fn test_clear_unused_cached_modules_from_module_graph() {
    let mut module_graph = construct_test_module_graph_complex();
    module_graph.remove_edge(&"B".into(), &"E".into()).unwrap();

    clear_unused_cached_modules_from_module_graph(&mut module_graph);
    assert!(!module_graph.has_module(&"E".into()));
  }
}
