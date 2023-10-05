use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::ModuleMetaData,
  plugin::{PluginHookContext, PluginParseHookParam},
};

pub fn parse(
  parse_param: &PluginParseHookParam,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> Result<ModuleMetaData> {
  #[cfg(feature = "profile")]
  farmfe_core::puffin::profile_function!();

  if (parse_param
    .resolved_path
    .contains("src/components/Card.vue"))
  {
    println!(
      "parse_param.resolved_path: {} content: {}",
      parse_param.resolved_path, parse_param.content
    );
  }

  match context
    .plugin_driver
    .parse(parse_param, context, hook_context)
  {
    Ok(meta) => match meta {
      Some(meta) => Ok(meta),
      None => Err(CompilationError::ParseError {
        resolved_path: parse_param.resolved_path.clone(),
        msg: "No plugins handle this kind of module".to_string(),
      }),
    },
    Err(e) => Err(e),
  }
}
