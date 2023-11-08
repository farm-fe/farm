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
  let print = transform_param.resolved_path.ends_with("index.module.less")
    || transform_param.resolved_path.ends_with("vue");
  let transformed = context.plugin_driver.transform(transform_param, context)?;
  if print {
    println!("transformed: {:?}", transformed);
  }
  Ok(transformed)
}
