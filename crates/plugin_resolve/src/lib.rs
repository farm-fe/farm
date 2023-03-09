use std::{collections::HashMap, path::Path, sync::Arc};

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::Result,
  plugin::{Plugin, PluginHookContext, PluginResolveHookParam, PluginResolveHookResult},
};
use farmfe_utils::parse_query;
use resolver::Resolver;

pub mod resolver;

/// ScriptPlugin is used to support compiling js/ts/jsx/tsx files to js chunks
pub struct FarmPluginResolve {
  root: String,
}

impl FarmPluginResolve {
  pub fn new(config: &Config) -> Self {
    Self {
      root: config.root.clone(),
    }
  }
}

impl Plugin for FarmPluginResolve {
  fn name(&self) -> &str {
    "FarmPluginResolve"
  }

  fn resolve(
    &self,
    param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<PluginResolveHookResult>> {
    let source = &param.source;
    let query = parse_query(source);
    // split query from source
    let splits: Vec<&str> = source.split('?').collect();
    let source = splits[0];
    
    let basedir = if let Some(importer) = &param.importer {
      if let Some(p) = Path::new(&importer.resolved_path(&context.config.root)).parent() {
        p.to_path_buf()
      } else {
        Path::new(&importer.resolved_path(&context.config.root)).to_path_buf()
      }
    } else {
      Path::new(&self.root).to_path_buf()
    };

    // check external first, if the source is set as external, return it immediately
    if context.config.external.iter().any(|e| &param.source == e) {
      return Ok(Some(PluginResolveHookResult {
        resolved_path: param.source.clone(),
        external: true,
        side_effects: false,
        query,
      }));
    }

    let resolver = Resolver::new(context.config.resolve.clone());
    Ok(resolver.resolve(source, basedir, &param.kind).map(|result| {
      PluginResolveHookResult {
        query,
        ..result
      }}))
  }
}
