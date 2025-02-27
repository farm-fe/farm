use farmfe_core::module::{module_graph::ModuleGraph, ModuleId, ModuleSystem};
use farmfe_core::HashMap;

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

  let module_ids = tree_shake_modules_map.keys().cloned().collect::<Vec<_>>();

  for module_id in module_ids {
    if let Some(shake_module) = tree_shake_modules_map.get_mut(&module_id) {
      // if the module is not esm, set default exports all so it won't be tree shaken
      if shake_module.module_system != ModuleSystem::EsModule {
        shake_module.pending_used_exports.set_export_all();
      }
    }
  }

  entry_module_ids
}
