use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  farm_profile_function, farm_profile_scope,
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdge},
    ModuleId,
  },
  plugin::hooks::freeze_module::PluginFreezeModuleHookParam,
  rayon::iter::{IntoParallelIterator, ParallelIterator},
  HashMap,
};

use super::module_cache::set_module_graph_cache;

/// Finalize module graph when module graph is built:
/// 1. call module_graph_build_end hook
/// 2. update execution order of module graph
pub fn finalize_module_graph(context: &Arc<CompilationContext>) -> farmfe_core::error::Result<()> {
  let mut module_graph = context.module_graph.write();

  {
    farm_profile_scope!("call module_graph_build_end hook".to_string());
    context
      .plugin_driver
      .module_graph_build_end(&mut module_graph, context)?;
  }

  // update execution order when the module graph is freezed in build stage
  module_graph.update_execution_order_for_modules();

  Ok(())
}

pub(super) fn cache_module_graph(context: &Arc<CompilationContext>) {
  let module_ids = {
    let module_graph = context.module_graph.read();
    module_graph
      .modules()
      .iter()
      .map(|m| m.id.clone())
      .collect::<Vec<_>>()
  };

  // set new module cache
  set_module_graph_cache(module_ids, context);
}

pub(super) fn freeze_module_of_module_graph(
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  let mut module_graph = context.module_graph.write();
  farm_profile_function!("call freeze_module hook".to_string());
  let module_ids = module_graph.modules().into_iter().map(|m| m.id.clone());

  let mut resolved_deps_map = get_resolved_deps_of_modules(module_ids.collect(), &module_graph);

  let hook_params = module_graph
    .modules_mut()
    .into_iter()
    .map(|module| {
      let resolved_deps = resolved_deps_map.remove(&module.id).unwrap();
      PluginFreezeModuleHookParam {
        module,
        resolved_deps,
      }
    })
    .collect::<Vec<_>>();

  hook_params
    .into_par_iter()
    .try_for_each(|mut param| call_freeze_module_hook(&mut param, context))
}

pub fn get_resolved_deps_of_modules(
  module_ids: Vec<ModuleId>,
  module_graph: &ModuleGraph,
) -> HashMap<ModuleId, Vec<(ModuleId, ModuleGraphEdge)>> {
  let mut resolved_deps = HashMap::default();

  for module_id in module_ids {
    let deps = module_graph.dependencies(&module_id);
    resolved_deps.insert(
      module_id,
      deps
        .into_iter()
        .map(|(id, edge)| (id, edge.clone()))
        .collect::<Vec<_>>(),
    );
  }

  resolved_deps
}

/// Call freeze module hook with persistent cache
pub fn call_freeze_module_hook(
  param: &mut PluginFreezeModuleHookParam,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  // the result of freeze_module will be used from module cache if the module does not change
  if context
    .cache_manager
    .module_cache
    .is_cache_changed(param.module)
  {
    return context.plugin_driver.freeze_module(param, context);
  }

  Ok(())
}
