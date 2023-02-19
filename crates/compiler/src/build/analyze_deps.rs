use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::Module,
  plugin::{PluginAnalyzeDepsHookParam, PluginAnalyzeDepsHookResultEntry},
};
use farmfe_toolkit::tracing;

#[tracing::instrument(skip_all)]
pub fn analyze_deps(
  module: &Module,
  context: &Arc<CompilationContext>,
) -> Result<Vec<PluginAnalyzeDepsHookResultEntry>> {
  tracing::debug!("analyze_deps: {:?}", module.id);

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

  tracing::debug!("analyzed_deps: {:?}", module.id);
  Ok(analyze_deps_param.deps)
}
