use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::Module,
  plugin::{PluginHookContext, PluginParseHookParam},
};

pub fn parse(
  parse_param: PluginParseHookParam,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> Result<Module> {
  match context
    .plugin_driver
    .parse(&parse_param, context, hook_context)
  {
    Ok(module) => match module {
      Some(module) => Ok(module),
      None => Err(CompilationError::ParseError {
        id: parse_param.id,
        source: None,
      }),
    },
    Err(e) => Err(CompilationError::ParseError {
      id: parse_param.id,
      source: Some(Box::new(e)),
    }),
  }
}
