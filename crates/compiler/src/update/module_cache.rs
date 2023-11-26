use std::sync::Arc;

use farmfe_core::{context::CompilationContext, module::ModuleId};

use crate::build::module_cache::set_module_graph_cache;

use super::diff_and_patch_module_graph::DiffResult;

pub fn get_affected_module_ids(
  updated_modules: &Vec<ModuleId>,
  diff_result: &DiffResult,
  context: &Arc<CompilationContext>,
) -> Vec<ModuleId> {
  let mut affected_module_ids = vec![];

  for module_id in updated_modules {
    affected_module_ids.push(module_id.clone());
  }

  for added_id in &diff_result.added_modules {
    affected_module_ids.push(added_id.clone());
  }

  for removed_id in &diff_result.removed_modules {
    context
      .cache_manager
      .module_cache
      .invalidate_cache(removed_id);
  }

  affected_module_ids
}

pub fn set_updated_modules_cache(
  updated_modules: &Vec<ModuleId>,
  diff_result: &DiffResult,
  context: &Arc<CompilationContext>,
) {
  let affected_module_ids = get_affected_module_ids(updated_modules, diff_result, context);

  set_module_graph_cache(affected_module_ids, context);
}
