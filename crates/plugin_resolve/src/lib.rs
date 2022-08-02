use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::Result,
  plugin::{Plugin, PluginResolveHookParam, PluginResolveHookResult},
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
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<PluginResolveHookResult>> {
    let source = &param.source;
    let basedir = if let Some(importer) = &param.importer {
      Path::new(importer).parent().unwrap().to_path_buf()
    } else {
      Path::new(&self.root).to_path_buf()
    };

    self
      .resolver
      .resolve(source, basedir, &param.kind)
      .map(|r| Some(r))
  }
}
