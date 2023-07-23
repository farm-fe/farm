use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::Result,
  plugin::{plugin_driver::PluginDriverTransformHookResult, PluginTransformHookParam},
};

pub fn transform(
  transform_param: PluginTransformHookParam,
  context: &Arc<CompilationContext>,
) -> Result<PluginDriverTransformHookResult> {
  #[cfg(feature = "profile")]
  farmfe_core::puffin::profile_function!();

  let transformed = context.plugin_driver.transform(transform_param, context)?;

  Ok(transformed)
}
