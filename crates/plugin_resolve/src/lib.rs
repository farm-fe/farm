use std::{
  collections::HashMap,
  path::Path,
  sync::{Arc, RwLock},
};

use farmfe_core::{
  config::{external::ExternalConfig, Config},
  context::CompilationContext,
  error::{CompilationError, Result},
  farm_profile_function, farm_profile_scope,
  plugin::{
    Plugin, PluginHookContext, PluginResolveHookParam, PluginResolveHookResult, ResolveKind,
  },
  serde_json,
};

use farmfe_toolkit::resolve::DYNAMIC_EXTENSION_PRIORITY;
use farmfe_utils::parse_query;
use resolver::{ResolveOptions, Resolver};

pub mod resolver;

pub struct FarmPluginResolve {
  root: String,
  resolver: Resolver,
  external_config: RwLock<Option<ExternalConfig>>,
}

impl FarmPluginResolve {
  pub fn new(config: &Config) -> Self {
    Self {
      root: config.root.clone(),
      resolver: Resolver::new(),
      external_config: RwLock::new(None),
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
    hook_context: &PluginHookContext,
  ) -> Result<Option<PluginResolveHookResult>> {
    farm_profile_function!("plugin_resolve::resolve".to_string());

    let mut external_config = self
      .external_config
      .read()
      .map_err(|_| CompilationError::GenericError("failed get lock".to_string()))?;

    if external_config.is_none() {
      drop(external_config);
      let mut external_config_mut = self.external_config.write().unwrap();

      *external_config_mut = Some(ExternalConfig::from(&*context.config));

      drop(external_config_mut);

      external_config = self.external_config.read().unwrap();
    }

    let external_config = external_config.as_ref().unwrap();

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
    if !matches!(param.kind, ResolveKind::Entry(_)) {
      farm_profile_scope!("plugin_resolve::resolve::check_external".to_string());
      // check external first, if the source is set as external, return it immediately
      if external_config.is_external(source) {
        return Ok(Some(PluginResolveHookResult {
          resolved_path: param.source.clone(),
          external: true,
          side_effects: false,
          query,
          meta: HashMap::new(),
        }));
      }
    }

    let dynamic_extensions =
      if let Some(dynamic_extensions) = hook_context.meta.get(DYNAMIC_EXTENSION_PRIORITY) {
        let exts = serde_json::from_str::<Vec<String>>(dynamic_extensions).unwrap_or_default();

        if exts.len() > 0 {
          Some(exts)
        } else {
          None
        }
      } else {
        None
      };
    let resolve_options = ResolveOptions { dynamic_extensions };

    let resolver = &self.resolver;
    let result = resolver.resolve(
      source,
      basedir.clone(),
      &param.kind,
      &resolve_options,
      context,
    );

    // remove the .js if the result is not found to support using native esm with typescript
    let mut resolve_result = if result.is_none() && source.ends_with(".js") {
      farm_profile_scope!("plugin_resolve::resolve::remove_dot_js".to_string());
      let source = source.replace(".js", "");

      resolver
        .resolve(&source, basedir, &param.kind, &resolve_options, context)
        .map(|result| PluginResolveHookResult { query, ..result })
    } else {
      result.map(|result| PluginResolveHookResult { query, ..result })
    };

    if resolve_result.is_none() && context.config.resolve.auto_external_failed_resolve {
      resolve_result = Some(PluginResolveHookResult {
        resolved_path: param.source.clone(),
        external: true,
        side_effects: false,
        query: vec![],
        meta: HashMap::new(),
      });
    }

    Ok(resolve_result)
  }
}
