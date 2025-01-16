use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  farm_profile_scope,
  module::ModuleId,
  rayon::iter::{IntoParallelIterator, ParallelIterator},
};

use super::diff_and_patch_module_graph::DiffResult;

/// Finalize module graph when updated module graph is built
pub fn finalize_updated_module_graph(
  updated_modules: &Vec<ModuleId>,
  removed_module_ids: Vec<ModuleId>,
  diff_result: &DiffResult,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  // Topo sort the module graph
  let mut module_graph = context.module_graph.write();

  {
    farm_profile_scope!("call freeze_module hook".to_string());
    let mut modules_ids = diff_result.added_modules.clone();
    modules_ids.extend(updated_modules.clone());

    module_graph
      .modules_mut()
      .into_par_iter()
      .filter(|module| modules_ids.contains(&module.id))
      .try_for_each(|module| context.plugin_driver.freeze_module(module, context))?;
  }

  // call module graph updated hook
  context.plugin_driver.module_graph_updated(
    &farmfe_core::plugin::PluginModuleGraphUpdatedHookParams {
      added_modules_ids: diff_result.added_modules.clone().into_iter().collect(),
      removed_modules_ids: removed_module_ids,
      updated_modules_ids: updated_modules.clone(),
    },
    context,
  )?;

  Ok(())
}
