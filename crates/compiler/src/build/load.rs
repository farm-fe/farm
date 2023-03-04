use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  plugin::{PluginHookContext, PluginLoadHookParam, PluginLoadHookResult},
};
use farmfe_toolkit::tracing;

#[tracing::instrument(skip_all)]
pub fn load(
  load_param: &PluginLoadHookParam,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> Result<PluginLoadHookResult> {
  tracing::trace!("load: {}", load_param.resolved_path);

  let loaded = match context
    .plugin_driver
    .load(load_param, context, hook_context)
  {
    Ok(loaded) => match loaded {
      Some(loaded) => loaded,
      None => {
        return Err(CompilationError::LoadError {
          resolved_path: load_param.resolved_path.to_string(),
          source: None,
        });
      }
    },
    Err(e) => {
      return Err(CompilationError::LoadError {
        resolved_path: load_param.resolved_path.to_string(),
        source: Some(Box::new(e)),
      });
    }
  };
  tracing::trace!("loaded: {}", load_param.resolved_path);

  Ok(loaded)
}
