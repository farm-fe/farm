use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  plugin::{PluginHookContext, PluginResolveHookParam, PluginResolveHookResult},
};

pub fn resolve(
  resolve_param: &PluginResolveHookParam,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> Result<PluginResolveHookResult> {
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

  println!(
    "resolved {:?} from {:?}, result: {:?}",
    resolve_param.source, importer, resolved
  );
  Ok(resolved)
}
