use std::{collections::HashMap, path::Path, sync::Arc};

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::Result,
  farm_profile_function, farm_profile_scope,
  plugin::{
    Plugin, PluginHookContext, PluginResolveHookParam, PluginResolveHookResult, ResolveKind,
  },
};

use farmfe_utils::parse_query;
use resolver::Resolver;

pub mod resolver;

pub struct FarmPluginResolve {
  root: String,
  resolver: Resolver,
}

impl FarmPluginResolve {
  pub fn new(config: &Config) -> Self {
    Self {
      root: config.root.clone(),
      resolver: Resolver::new(),
    }
  }
}

impl Plugin for FarmPluginResolve {
  fn name(&self) -> &str {
    "FarmPluginResolve"
  }

  // Internal Resolve Plugin has the lower priority, so it will be executed at last
  fn priority(&self) -> i32 {
    99
  }

  fn resolve(
    &self,
    param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<PluginResolveHookResult>> {
    farm_profile_function!("plugin_resolve::resolve".to_string());

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

    // Entry module and internal modules should not be external
    if !matches!(param.kind, ResolveKind::Entry(_))
      && [
        "@swc/helpers/_/_interop_require_default",
        "@swc/helpers/_/_interop_require_wildcard",
        "@swc/helpers/_/_export_star",
      ]
      .into_iter()
      .all(|s| source != s)
    {
      farm_profile_scope!("plugin_resolve::resolve::check_external".to_string());
      // check external first, if the source is set as external, return it immediately
      if context.config.external.iter().any(|e| e.is_match(source)) {
        return Ok(Some(PluginResolveHookResult {
          resolved_path: param.source.clone(),
          external: true,
          side_effects: false,
          query,
          meta: HashMap::new(),
        }));
      }
    }

    let resolver = &self.resolver;
    let result = resolver.resolve(source, basedir.clone(), &param.kind, context);

    // remove the .js if the result is not found to support using native esm with typescript
    if result.is_none() && source.ends_with(".js") {
      farm_profile_scope!("plugin_resolve::resolve::remove_dot_js".to_string());
      let source = source.replace(".js", "");

      return Ok(
        resolver
          .resolve(&source, basedir, &param.kind, context)
          .map(|result| PluginResolveHookResult { query, ..result }),
      );
    }

    Ok(result.map(|result| PluginResolveHookResult { query, ..result }))
  }
}
