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
  #[cfg(feature = "profile")]
  farmfe_core::puffin::profile_function!();

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

  if analyze_deps_param
    .deps
    .iter()
    .any(|dep| dep.source.is_empty())
  {
    println!(
      "Warning: module {} has empty source in analyze_deps",
      module.id.to_string()
    );
  }

  Ok(analyze_deps_param.deps)
}
