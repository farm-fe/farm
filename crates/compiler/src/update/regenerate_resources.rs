use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  hashbrown::{HashMap, HashSet},
  module::{module_graph::ModuleGraph, module_group::ModuleGroupMap, Module, ModuleId},
  plugin::PluginHookContext,
};

use farmfe_plugin_runtime::render_resource_pot::resource_pot_to_runtime_object_lit;

use super::diff_and_patch_module_graph::{DiffResult, ModuleDepsDiffResultMap};

pub fn regenerate_resources(
  updated_module_ids: Vec<ModuleId>,
  removed_modules: HashMap<ModuleId, Module>,
  diff_result: &DiffResult,
  module_graph: &mut ModuleGraph,
  module_group_map: &mut ModuleGroupMap,
) -> farmfe_core::error::Result<String> {
  // let mut affected_resource_pots = HashSet::new();

  // let module_graph = context.module_graph.read();
  // let module_group_map = context.module_group_map.read();

  // Second, re-generating the resources
  Ok("".to_string())
}

fn re_calculate_module_group(
  updated_module_ids: Vec<ModuleId>,
  changed_deps: ModuleDepsDiffResultMap,
  module_graph: &ModuleGraph,
  module_group_map: &mut ModuleGroupMap,
) -> farmfe_core::error::Result<()> {
  let mut affected_module_groups = HashSet::new();

  // First, re-calculating the module group
  for module_id in &updated_module_ids {
    let module = module_graph.module(module_id).unwrap();
    let module_group_ids = module.module_groups.clone();
    affected_module_groups.extend(module_group_ids);
  }

  Ok(())
}
