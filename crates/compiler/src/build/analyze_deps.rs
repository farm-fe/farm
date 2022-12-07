use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::Module,
  plugin::{PluginAnalyzeDepsHookParam, PluginAnalyzeDepsHookResultEntry},
};

pub fn analyze_deps(
  module: &Module,
  context: &Arc<CompilationContext>,
) -> Result<Vec<PluginAnalyzeDepsHookResultEntry>> {
  let mut analyze_deps_param = PluginAnalyzeDepsHookParam {
    module,
    deps: vec![],
  };
  if let Err(e) = context
    .plugin_driver
    .analyze_deps(&mut analyze_deps_param, context)
  {
    return Err(CompilationError::AnalyzeDepsError {
      resolved_path: module.id.resolved_path(&context.config.root),
      source: Some(Box::new(e)),
    });
  };

  Ok(analyze_deps_param.deps)
}
