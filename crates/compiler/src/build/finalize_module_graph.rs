use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  farm_profile_scope,
  rayon::iter::{IntoParallelIterator, ParallelIterator},
};

/// Finalize module graph when module graph is built:
/// 1. call freeze module hook
/// 2. call module_graph_build_end hook
/// 3. update execution order or module graph
pub fn finalize_module_graph(context: &Arc<CompilationContext>) -> farmfe_core::error::Result<()> {
  // Topo sort the module graph
  let mut module_graph = context.module_graph.write();

  {
    farm_profile_scope!("call freeze_module hook".to_string());
    module_graph
      .modules_mut()
      .into_par_iter()
      .try_for_each(|module| context.plugin_driver.freeze_module(module, context))?;
  }

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
