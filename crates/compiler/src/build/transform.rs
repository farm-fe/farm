use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  plugin::{plugin_driver::PluginDriverTransformHookResult, PluginTransformHookParam},
};

pub fn transform(
  transform_param: PluginTransformHookParam,
  context: &Arc<CompilationContext>,
) -> Result<PluginDriverTransformHookResult> {
  #[cfg(feature = "profile")]
  farmfe_core::puffin::profile_function!();
  let module_id = transform_param.module_id.to_string();
  let transformed = context
    .plugin_driver
    .transform(transform_param, context)
    .map_err(|e| CompilationError::TransformError {
      resolved_path: module_id,
      msg: e.to_string(),
    })?;

  Ok(transformed)
}
