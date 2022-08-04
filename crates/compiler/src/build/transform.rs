use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::CompilationError,
  error::Result,
  plugin::{plugin_driver::PluginDriverTransformHookResult, PluginTransformHookParam},
};

pub fn transform(
  transform_param: PluginTransformHookParam,
  context: &Arc<CompilationContext>,
) -> Result<PluginDriverTransformHookResult> {
  let id = transform_param.id.to_string();
  let transformed = match context.plugin_driver.transform(transform_param, &context) {
    Ok(transformed) => transformed,
    Err(e) => {
      return Err(CompilationError::TransformError {
        id,
        source: Some(Box::new(e)),
      });
    }
  };

  println!("transformed {:?}", transformed);
  Ok(transformed)
}
