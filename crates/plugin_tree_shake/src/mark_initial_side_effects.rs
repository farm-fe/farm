use std::collections::HashMap;

use farmfe_core::module::{module_graph::ModuleGraph, ModuleId};

use crate::module::{TreeShakeModule, UsedExports};

pub fn mark_initial_side_effects(
  module_graph: &ModuleGraph,
  tree_shake_modules_map: &mut HashMap<ModuleId, TreeShakeModule>,
) -> Vec<ModuleId> {
  let mut entry_module_ids = vec![];

  for (entry_module_id, _) in module_graph.entries.clone() {
    // mark entry modules as UsedExports::All
    if let Some(tree_shake_module) = tree_shake_modules_map.get_mut(&entry_module_id) {
      tree_shake_module.pending_used_exports = UsedExports::All;
    }

    entry_module_ids.push(entry_module_id);
  }

  let module_ids = tree_shake_modules_map
    .keys()
    .map(|m| m.clone())
    .collect::<Vec<_>>();

  for module_id in module_ids {
    let no_dependents_tree_shakeable = module_graph
      .dependents_ids(&module_id)
      .iter()
      .all(|dept_id| !tree_shake_modules_map.contains_key(dept_id));

    if let Some(shake_module) = tree_shake_modules_map.get_mut(&module_id) {
      // if the module do not have tree shakeable parent, mark it as side effects
      if no_dependents_tree_shakeable {
        shake_module.side_effects = true;
      }

      if shake_module.side_effects {
        shake_module.pending_used_exports.set_export_all();
      }
    }
  }

  // // update contains_self_executed_stmt for the tree_shake_modules
  // for tree_shake_module_id in module_graph.toposort().0 {
  //   if let Some(tree_shake_module) = tree_shake_modules_map.get(&tree_shake_module_id) {
  //     let contains_self_executed_stmt = tree_shake_module.contains_self_executed_stmt;
  //     module_graph
  //       .dependents_ids(&tree_shake_module_id)
  //       .into_iter()
  //       .for_each(|dept_id| {
  //         if let Some(dept_tree_shake_module) = tree_shake_modules_map.get_mut(&dept_id) {
  //           dept_tree_shake_module.contains_self_executed_stmt =
  //             dept_tree_shake_module.contains_self_executed_stmt || contains_self_executed_stmt
  //         }
  //       });
  //   }
  // }

  entry_module_ids
}
