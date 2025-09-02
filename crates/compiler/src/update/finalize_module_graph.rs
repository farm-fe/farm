use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext, farm_profile_function, module::ModuleId,
  plugin::hooks::freeze_module::PluginFreezeModuleHookParam,
};

use crate::build::finalize_module_graph::{
  call_freeze_module_with_params, get_resolved_deps_of_modules, update_modules_resolved_deps,
};

use super::diff_and_patch_module_graph::DiffResult;

/// Finalize module graph when updated module graph is built
pub fn finalize_updated_module_graph(
  updated_modules: &Vec<ModuleId>,
  removed_module_ids: Vec<ModuleId>,
  diff_result: &DiffResult,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  // call module graph updated hook
  context.plugin_driver.module_graph_updated(
    &farmfe_core::plugin::PluginModuleGraphUpdatedHookParam {
      added_modules_ids: diff_result.added_modules.clone().into_iter().collect(),
      removed_modules_ids: removed_module_ids,
      updated_modules_ids: updated_modules.clone(),
      deps_changes: diff_result.deps_changes.clone(),
    },
    context,
  )?;

  Ok(())
}

pub(super) fn freeze_module_of_affected_module_graph(
  updated_modules: &Vec<ModuleId>,
  diff_result: &DiffResult,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  farm_profile_function!("call freeze_module hook".to_string());

  let mut module_graph = context.module_graph.write();

  let mut modules_ids = diff_result.added_modules.clone();
  modules_ids.extend(updated_modules.clone());
  let modules_ids_vec = modules_ids.clone().into_iter().collect::<Vec<_>>();

  let mut resolved_deps_map = get_resolved_deps_of_modules(modules_ids_vec, &module_graph);

  let hook_params = module_graph
    .modules_mut()
    .into_iter()
    .filter(|module| modules_ids.contains(&module.id))
    .map(|module| {
      let resolved_deps = resolved_deps_map.remove(&module.id).unwrap();
      PluginFreezeModuleHookParam {
        module,
        resolved_deps,
      }
    })
    .collect::<Vec<_>>();

  let module_resolved_deps = call_freeze_module_with_params(hook_params, context)?;

  update_modules_resolved_deps(module_resolved_deps, &mut module_graph);

  Ok(())
}
