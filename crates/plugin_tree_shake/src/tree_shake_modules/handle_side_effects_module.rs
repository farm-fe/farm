use std::collections::HashMap;

use farmfe_core::module::{module_graph::ModuleGraph, ModuleId};

use crate::module::TreeShakeModule;

use super::utils::{add_used_exports_by_export_info, add_used_exports_by_import_info};

pub fn handle_side_effects_module(
  tree_shake_module_id: &ModuleId,
  tree_shake_modules_map: &mut HashMap<ModuleId, TreeShakeModule>,
  module_graph: &ModuleGraph,
) {
  let (imports, exports) = {
    let tree_shake_module = tree_shake_modules_map
      .get(tree_shake_module_id)
      .unwrap_or_else(|| {
        panic!("tree shake module not found: {:?}", tree_shake_module_id);
      });
    (
      tree_shake_module
        .imports()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>(),
      tree_shake_module
        .exports()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>(),
    )
  };

  for import_info in imports {
    add_used_exports_by_import_info(
      tree_shake_modules_map,
      module_graph,
      tree_shake_module_id,
      import_info,
    );
  }

  for export_info in exports {
    add_used_exports_by_export_info(
      tree_shake_modules_map,
      module_graph,
      tree_shake_module_id,
      export_info,
    );
  }
}
