use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext, error::CompilationError, plugin::PluginHookContext,
};

pub fn merge_modules(
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<()> {
  let mut module_graph = context.module_graph.write();

  println!("analyze module graph start");

  let mut module_group_map = context
    .plugin_driver
    .analyze_module_graph(&mut *module_graph, context, hook_context)?
    .ok_or(CompilationError::PluginHookResultCheckError {
      hook_name: "analyze_module_graph".to_string(),
    })?;

  drop(module_graph);

  println!("module group map len: {}", module_group_map.len());

  println!("merge modules start");

  let resource_pot_graph = context
    .plugin_driver
    .merge_modules(&mut module_group_map, context, hook_context)?
    .ok_or(CompilationError::PluginHookResultCheckError {
      hook_name: "merge_modules".to_string(),
    })?;

  context.module_group_map.write().replace(module_group_map);

  let mut g = context.resource_pot_graph.write();
  g.replace(resource_pot_graph);

  Ok(())
}
