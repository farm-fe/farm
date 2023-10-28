use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  plugin::{PluginHookContext, PluginLoadHookParam, PluginLoadHookResult},
};

pub fn load(
  load_param: &PluginLoadHookParam,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> Result<PluginLoadHookResult> {
  #[cfg(feature = "profile")]
  farmfe_core::puffin::profile_function!();

  let loaded = match context
    .plugin_driver
    .load(load_param, context, hook_context)
  {
    Ok(loaded) => match loaded {
      Some(loaded) => loaded,
      None => {
        return Err(CompilationError::LoadError {
          resolved_path: load_param.module_id.to_string(),
          source: None,
        });
      }
    },
    Err(e) => {
      return Err(CompilationError::LoadError {
        resolved_path: load_param.module_id.to_string(),
        source: Some(Box::new(e)),
      });
    }
  };

  Ok(loaded)
}
