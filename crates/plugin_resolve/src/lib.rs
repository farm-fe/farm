use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::Arc,
};

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::Result,
  plugin::{Plugin, PluginHookContext, PluginResolveHookParam, PluginResolveHookResult},
  serde_json::Value,
};
use resolver::Resolver;

pub mod resolver;

/// ScriptPlugin is used to support compiling js/ts/jsx/tsx files to js chunks
pub struct FarmPluginResolve {
  resolver: Resolver,
  root: String,
}

impl FarmPluginResolve {
  pub fn new(config: &Config) -> Self {
    Self {
      resolver: Resolver::new(config.resolve.clone()),
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
    let basedir = if let Some(importer) = &param.importer {
      if let Some(p) = Path::new(importer).parent() {
        p.to_path_buf()
      } else {
        Path::new(importer).to_path_buf()
      }
    } else {
      Path::new(&self.root).to_path_buf()
    };

    // check external first, if the source is set as external, return it immediately
    if context.config.external.iter().any(|e| &param.source == e) {
      return Ok(Some(PluginResolveHookResult {
        id: param.source.clone(),
        external: true,
        side_effects: false,
        package_json_info: None,
        query: HashMap::new(),
      }));
    }

    self
      .resolver
      .resolve(source, basedir, &param.kind)
      .map(|r| Some(r))
  }
}
