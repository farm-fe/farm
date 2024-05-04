use std::collections::HashMap;

use farmfe_core::module::{module_graph::ModuleGraph, ModuleId};

use crate::module::TreeShakeModule;

pub fn mark_initial_side_effects(
  module_graph: &mut ModuleGraph,
  topo_sorted_modules: Vec<ModuleId>,
  tree_shake_modules_map: &mut HashMap<ModuleId, TreeShakeModule>,
) -> Vec<ModuleId> {
  // mark entry modules as side_effects
  for (entry_module_id, _) in module_graph.entries.clone() {
    let module = module_graph.module_mut(&entry_module_id).unwrap();
    module.side_effects = true;
  }

  let mut tree_shake_modules_ids = vec![];

  for module_id in topo_sorted_modules {
    let module = module_graph.module(&module_id).unwrap();

    // skip non script modules and external modules
    if !module.module_type.is_script() || module.external {
      if !module.module_type.is_script() && !module.external {
        // mark all non script modules' script dependencies as side_effects
        for dep_id in module_graph.dependencies_ids(&module_id) {
          let dep_module = module_graph.module_mut(&dep_id).unwrap();

          if !dep_module.module_type.is_script() {
            continue;
          }

          dep_module.side_effects = true;
        }
      }

      continue;
    }

    if let Some(shake_module) = tree_shake_modules_map.get_mut(&module.id) {
      shake_module.side_effects = module.side_effects;
      if shake_module.side_effects {
        shake_module.pending_used_exports.set_export_all();
      }
    }

    // add all dynamic imported dependencies as [UsedExports::All]
    for (dep, edge) in module_graph.dependencies(&module_id) {
      if edge.is_dynamic() && tree_shake_modules_map.contains_key(&dep) {
        let tree_shake_module = tree_shake_modules_map.get_mut(&dep).unwrap_or_else(|| {
          panic!("dynamic imported module not found: {:?}", dep);
        });
        tree_shake_module.side_effects = true;
        tree_shake_module.pending_used_exports.set_export_all();
      }
    }

    if let Some(tree_shake_module) = tree_shake_modules_map.get_mut(&module.id) {
      if tree_shake_module.side_effects {
        for stmt_id in tree_shake_module
          .stmt_graph
          .stmts()
          .iter()
          .map(|i| i.id)
          .collect::<Vec<_>>()
        {
          tree_shake_module.stmt_graph.mark_used_statements(stmt_id);
        }
      }
    }

    tree_shake_modules_ids.push(module_id);
  }

  tree_shake_modules_ids
}
