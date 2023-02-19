use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::CompilationError,
  error::Result,
  plugin::{plugin_driver::PluginDriverTransformHookResult, PluginTransformHookParam},
};
use farmfe_toolkit::tracing;

#[tracing::instrument(skip_all)]
pub fn transform(
  transform_param: PluginTransformHookParam,
  context: &Arc<CompilationContext>,
) -> Result<PluginDriverTransformHookResult> {
  let resolved_path = transform_param.resolved_path.to_string();
  tracing::debug!("transform: {}", resolved_path);

  let resolved_path = transform_param.resolved_path.to_string();
  let transformed = match context.plugin_driver.transform(transform_param, context) {
    Ok(transformed) => transformed,
    Err(e) => {
      return Err(CompilationError::TransformError {
        resolved_path,
        source: Some(Box::new(e)),
      });
    }
  };

  tracing::debug!("transformed: {}", resolved_path);
  Ok(transformed)
}
