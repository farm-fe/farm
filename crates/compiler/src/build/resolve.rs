use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  plugin::{PluginHookContext, PluginResolveHookParam, PluginResolveHookResult},
};
use farmfe_toolkit::tracing;

#[tracing::instrument(skip_all)]
pub fn resolve(
  resolve_param: &PluginResolveHookParam,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> Result<PluginResolveHookResult> {
  tracing::debug!(
    "resolve: {}, importer: {:?}",
    resolve_param.source,
    resolve_param.importer
  );

  let importer = resolve_param
    .importer
    .clone()
    .map(|p| p.relative_path().to_string())
    .unwrap_or_else(|| context.config.root.clone());

  let resolved = match context
    .plugin_driver
    .resolve(resolve_param, context, hook_context)
  {
    Ok(resolved) => match resolved {
      Some(res) => res,
      None => {
        return Err(CompilationError::ResolveError {
          importer,
          src: resolve_param.source.clone(),
          source: None,
        });
      }
    },
    Err(e) => {
      return Err(CompilationError::ResolveError {
        importer,
        src: resolve_param.source.clone(),
        source: Some(Box::new(e)),
      });
    }
  };

  tracing::debug!(
    "resolved: {}, importer: {:?}, source {}",
    resolved.resolved_path,
    resolve_param.importer,
    resolve_param.source
  );
  Ok(resolved)
}
