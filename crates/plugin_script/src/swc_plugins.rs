use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use farmfe_core::{
  config::{ScriptConfigPlugin, ScriptConfigPluginFilters},
  context::CompilationContext,
  error::Result,
  module::ModuleType,
  parking_lot,
  plugin::PluginProcessModuleHookParam,
  swc_common::{self, plugin::metadata::TransformPluginMetadataContext, FileName, Mark},
  swc_ecma_ast::{Module as SwcModule, Program, Script},
};
use farmfe_toolkit::{
  regex::Regex,
  swc_ecma_visit::{noop_fold_type, Fold, FoldWith},
};
use once_cell::sync::Lazy;
use swc_ecma_loader::{
  resolve::Resolve,
  resolvers::{lru::CachingResolver, node::NodeModulesResolver},
};

// This file is modified from https://github.com/swc-project/swc/tree/main/crates/swc/src/plugin.rs

/// A shared instance to plugin's module bytecode cache.
pub static PLUGIN_MODULE_CACHE: Lazy<swc_plugin_runner::cache::PluginModuleCache> =
  Lazy::new(Default::default);
pub static CACHING_RESOLVER: Lazy<CachingResolver<NodeModulesResolver>> =
  Lazy::new(|| CachingResolver::new(40, NodeModulesResolver::default()));

pub fn init_plugin_module_cache_once(
  enable_fs_cache_store: bool,
  fs_cache_store_root: &Option<String>,
) {
  PLUGIN_MODULE_CACHE.inner.get_or_init(|| {
    parking_lot::Mutex::new(swc_plugin_runner::cache::PluginModuleCache::create_inner(
      enable_fs_cache_store,
      fs_cache_store_root,
    ))
  });
}

pub fn transform_by_swc_plugins(
  param: &mut PluginProcessModuleHookParam,
  context: &Arc<CompilationContext>,
) -> Result<()> {
  let mut plugins_should_execute = vec![];

  let plugins = &context.config.script.plugins;
  let mut inner_cache = PLUGIN_MODULE_CACHE
    .inner
    .get()
    .expect("Cache should be available")
    .lock();
  // Populate cache to the plugin modules if not loaded
  for plugin_config in plugins.iter() {
    if !should_execute_swc_plugin(
      param.module_id.resolved_path(&context.config.root),
      param.module_type.clone(),
      &plugin_config.filters,
    ) {
      continue;
    }

    plugins_should_execute.push(plugin_config.clone());

    let plugin_name = &plugin_config.name;

    if !inner_cache.contains(&plugin_name) {
      let plugin_resolver = &CACHING_RESOLVER;
      let resolved_path = plugin_resolver
        .resolve(&FileName::Real(PathBuf::from(&plugin_name)), &plugin_name)
        .unwrap();

      let path = if let FileName::Real(value) = resolved_path {
        value
      } else {
        panic!("Failed to resolve plugin path: {:?}", resolved_path);
      };

      inner_cache
        .store_bytes_from_path(&path, &plugin_name)
        .unwrap();
    }
  }
  drop(inner_cache);

  if plugins_should_execute.is_empty() {
    return Ok(());
  }

  let transform_metadata_context = Arc::new(TransformPluginMetadataContext::new(
    Some(param.module_id.to_string()),
    context.config.mode.to_string(),
    None,
  ));
  let unresolved_mark = Mark::from_u32(param.meta.as_script().unresolved_mark);
  let mut plugin_transforms = swc_plugins(
    Some(plugins_should_execute),
    transform_metadata_context,
    None,
    context.meta.script.cm.clone(),
    unresolved_mark,
  );

  let mut program = Program::Module(param.meta.as_script_mut().take_ast());
  // Fold module
  program = program.fold_with(&mut plugin_transforms);

  param.meta.as_script_mut().set_ast(program.expect_module());
  Ok(())
}

pub fn swc_plugins(
  configured_plugins: Option<Vec<ScriptConfigPlugin>>,
  metadata_context: std::sync::Arc<swc_common::plugin::metadata::TransformPluginMetadataContext>,
  comments: Option<swc_common::comments::SingleThreadedComments>,
  source_map: std::sync::Arc<swc_common::SourceMap>,
  unresolved_mark: swc_common::Mark,
) -> impl Fold {
  RustPlugins {
    plugins: configured_plugins,
    metadata_context,
    comments,
    source_map,
    unresolved_mark,
  }
}

struct RustPlugins {
  plugins: Option<Vec<ScriptConfigPlugin>>,
  metadata_context: std::sync::Arc<swc_common::plugin::metadata::TransformPluginMetadataContext>,
  comments: Option<swc_common::comments::SingleThreadedComments>,
  source_map: std::sync::Arc<swc_common::SourceMap>,
  unresolved_mark: swc_common::Mark,
}

impl RustPlugins {
  fn apply(&mut self, n: Program) -> std::result::Result<Program, anyhow::Error> {
    if self.plugins.is_none() || self.plugins.as_ref().unwrap().is_empty() {
      return Ok(n);
    }

    self.apply_inner(n).with_context(|| {
      format!(
        "failed to invoke plugin on '{:?}'",
        self.metadata_context.filename
      )
    })
  }

  fn apply_inner(&mut self, n: Program) -> std::result::Result<Program, anyhow::Error> {
    use swc_common::plugin::serialized::PluginSerializedBytes;

    // swc_plugin_macro will not inject proxy to the comments if comments is empty
    let should_enable_comments_proxy = self.comments.is_some();

    // TODO support comments
    // Set comments once per whole plugin transform execution.
    // swc_plugin_proxy::COMMENTS.set(
    //   &swc_plugin_proxy::HostCommentsStorage {
    //     inner: self.comments.clone(),
    //   },
    //   || {
    let mut serialized = PluginSerializedBytes::try_serialize(
      &swc_common::plugin::serialized::VersionedSerializable::new(n.clone()),
    )?;

    // Run plugin transformation against current program.
    // We do not serialize / deserialize between each plugin execution but
    // copies raw transformed bytes directly into plugin's memory space.
    // Note: This doesn't mean plugin won't perform any se/deserialization: it
    // still have to construct from raw bytes internally to perform actual
    // transform.
    if let Some(plugins) = &mut self.plugins {
      for p in plugins.drain(..) {
        let plugin_module_bytes = PLUGIN_MODULE_CACHE
          .inner
          .get()
          .unwrap()
          .lock()
          .get(&p.name)
          .expect("plugin module should be loaded");

        let plugin_name = plugin_module_bytes.get_module_name().to_string();

        let mut plugin_transform_executor = swc_plugin_runner::create_plugin_transform_executor(
          &self.source_map,
          &self.unresolved_mark,
          &self.metadata_context,
          plugin_module_bytes,
          Some(p.options.clone()),
        );

        serialized = plugin_transform_executor
          .transform(&serialized, Some(should_enable_comments_proxy))
          .with_context(|| {
            format!(
              "failed to invoke `{}` as js transform plugin at {}",
              &p.name, plugin_name
            )
          })?;
      }
    }

    // Plugin transformation is done. Deserialize transformed bytes back
    // into Program
    serialized.deserialize().map(|v| v.into_inner())
    //   },
    // )
  }
}

impl Fold for RustPlugins {
  noop_fold_type!();

  fn fold_module(&mut self, n: SwcModule) -> SwcModule {
    self
      .apply(Program::Module(n))
      .expect("failed to invoke plugin")
      .expect_module()
  }

  fn fold_script(&mut self, n: Script) -> Script {
    self
      .apply(Program::Script(n))
      .expect("failed to invoke plugin")
      .expect_script()
  }
}

fn should_execute_swc_plugin(
  resolved_path: String,
  module_type: ModuleType,
  filters: &ScriptConfigPluginFilters,
) -> bool {
  // transform it to Regex first and test against it
  let resolve_paths_regex = filters
    .resolved_paths
    .iter()
    .map(|p| Regex::new(p).unwrap())
    .collect::<Vec<Regex>>();

  resolve_paths_regex
    .iter()
    .any(|r| r.is_match(&resolved_path))
    || filters.module_types.contains(&module_type)
}
