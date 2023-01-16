//! diff the module_graph and update_module_graph, analyze the changes and then patch the module_graph

use farmfe_core::{
  hashbrown::HashMap,
  module::{module_graph::ModuleGraph, ModuleId},
};

/// the diff result of a module's dependencies
pub struct ModuleDepsDiffResult {
  /// added dependencies
  pub added: Vec<ModuleId>,
  /// removed dependencies
  pub removed: Vec<ModuleId>,
}

pub type DiffResult = HashMap<ModuleId, ModuleDepsDiffResult>;

#[derive(Debug, Default)]
pub struct PatchModuleGraphResult {
  /// added modules
  pub added: Vec<ModuleId>,
  /// updated modules
  pub updated: Vec<ModuleId>,
  /// removed modules
  pub removed: Vec<ModuleId>,
}

/// diff the module_graph and update_module_graph, return the diff result
pub fn diff_module_graph(
  start_points: Vec<ModuleId>,
  module_graph: &ModuleGraph,
  update_module_graph: &ModuleGraph,
) -> DiffResult {
  let mut res = HashMap::new();

  for start_point in start_points {
    let diff_result = diff_module_deps(&start_point, module_graph, update_module_graph);
    res.insert(start_point, diff_result);
  }

  res
}

/// patch the module_graph according to the diff result, return [PatchModuleGraphResult] after patching, as patching may discover new changes
/// TODO: describe the patching process
pub fn patch_module_graph(
  diff_result: &DiffResult,
  module_graph: &mut ModuleGraph,
  _update_module_graph: &ModuleGraph,
) -> PatchModuleGraphResult {
  let mut res = PatchModuleGraphResult::default();

  // let mut children_to_remove = vec![];

  for (module_id, diff_result) in diff_result {
    res.updated.push(module_id.clone());

    for removed_dep in &diff_result.removed {
      module_graph.remove_edge(module_id, removed_dep).unwrap();
    }
  }

  res
}

fn diff_module_deps(
  module_id: &ModuleId,
  module_graph: &ModuleGraph,
  update_module_graph: &ModuleGraph,
) -> ModuleDepsDiffResult {
  let mut added = Vec::new();
  let mut removed = Vec::new();

  let deps = module_graph.dependencies_ids(module_id);
  let update_deps = update_module_graph.dependencies_ids(module_id);

  for dep in &deps {
    if !update_deps.contains(dep) {
      removed.push(dep.clone());
    }
  }

  for dep in update_deps {
    if !deps.contains(&dep) {
      added.push(dep);
    }
  }

  ModuleDepsDiffResult { added, removed }
}
