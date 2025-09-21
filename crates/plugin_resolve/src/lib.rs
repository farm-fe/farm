use std::{path::Path, sync::Arc};

use farmfe_core::{
  config::{external::ExternalConfig, Config},
  context::CompilationContext,
  error::Result,
  farm_profile_function, farm_profile_scope,
  plugin::{
    Plugin, PluginHookContext, PluginResolveHookParam, PluginResolveHookResult, ResolveKind,
  },
  serde_json, HashMap, HashSet,
};

use farmfe_toolkit::resolve::DYNAMIC_EXTENSION_PRIORITY;
use farmfe_utils::parse_query;
use once_cell::sync::OnceCell;
use resolver::{parse_package_source, ResolveOptions, Resolver};

pub mod resolver;

pub struct FarmPluginResolve {
  root: String,
  resolver: Resolver,
  external_config: OnceCell<ExternalConfig>,
  dedupe: HashSet<String>,
}

impl FarmPluginResolve {
  pub fn new(config: &Config) -> Self {
    Self {
      dedupe: config.resolve.dedupe.clone().into_iter().collect(),
      root: config.root.clone(),
      resolver: Resolver::new(),
      external_config: OnceCell::new(),
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

    let external_config = self
      .external_config
      .get_or_init(|| ExternalConfig::from(&*context.config));

    let source = &param.source;

    let query = parse_query(source);
    // split query from source
    let splits: Vec<&str> = source.split('?').collect();
    let source = splits[0];

    let basedir =
      if parse_package_source(source).is_some_and(|r| self.dedupe.contains(&r.package_name)) {
        Path::new(&self.root).to_path_buf()
      } else if let Some(importer) = &param.importer {
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
          resolved_path: String::from(source),
          external: true,
          side_effects: false,
          query,
          meta: HashMap::default(),
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
        meta: HashMap::default(),
      });
    }

    Ok(resolve_result)
  }
}
