use std::sync::Arc;

use farmfe_core::{
  cache::ResolveResultCacheKey,
  context::CompilationContext,
  error::{CompilationError, Result},
  plugin::{PluginHookContext, PluginResolveHookParam, PluginResolveHookResult},
};

pub fn resolve(
  resolve_param: &PluginResolveHookParam,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> Result<PluginResolveHookResult> {
  #[cfg(feature = "profile")]
  farmfe_core::puffin::profile_function!();

  let resolve_cache_key = ResolveResultCacheKey {
    source: resolve_param.source.clone(),
    importer: resolve_param.importer.clone(),
  };

  if let Some(resolved) = context
    .cache_manager
    .get_resolve_result_cache_by_key(&resolve_cache_key)
  {
    return Ok(resolved);
  }

  let importer = resolve_param
    .importer
    .clone()
    .map(|p| p.relative_path().to_string())
    .unwrap_or_else(|| context.config.root.clone());

  let resolved = {
    #[cfg(feature = "profile")]
    let id = farmfe_utils::transform_string_to_static_str(format!(
      "resolve {} from {:?}",
      resolve_param.source, importer
    ));
    #[cfg(feature = "profile")]
    farmfe_core::puffin::profile_scope!(id);

    match context
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
    }
  };

  context
    .cache_manager
    .set_resolve_result_cache_by_key(resolve_cache_key, resolved.clone());

  Ok(resolved)
}
