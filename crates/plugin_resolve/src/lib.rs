use std::{collections::HashMap, path::Path, sync::Arc};

use farmfe_core::{
  config::Config,
  context::{CompilationContext},
  error::Result,
  plugin::{Plugin, PluginHookContext, PluginResolveHookParam, PluginResolveHookResult},
};
use farmfe_toolkit::regex::Regex;
use farmfe_utils::parse_query;
use resolver::Resolver;

pub mod resolver;

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

  // Internal Resolve Plugin has the lower priority, so it will be executed at last
  fn priority(&self) -> i32 {
    return 99;
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
    if context.config.external.iter().any(|e| {
      let reg = Regex::new(e).unwrap();
      reg.is_match(source)
    }) {
      return Ok(Some(PluginResolveHookResult {
        resolved_path: param.source.clone(),
        external: true,
        side_effects: false,
        query,
        meta: HashMap::new(),
      }));
    }

    let resolver = Resolver::new(context.config.resolve.clone(), context.config.output.clone());
    let result = resolver.resolve(source, basedir.clone(), &param.kind);

    // remove the .js if the result is not found to support using native esm with typescript
    if result.is_none() && source.ends_with(".js") {
      let source = source.replace(".js", "");

      return Ok(
        resolver
          .resolve(&source, basedir.clone(), &param.kind)
          .map(|result| PluginResolveHookResult { query, ..result }),
      );
    }

    Ok(
      resolver
        .resolve(source, basedir.clone(), &param.kind)
        .map(|result| PluginResolveHookResult { query, ..result }),
    )
  }
}
