//! Copied from Swc-Project, see https://github.com/swc-project/swc/blob/main/crates/swc/src/plugin.rs
//! License: MIT

use std::{path::PathBuf, sync::Arc};

use farmfe_core::config::script::{ScriptConfigPlugin, ScriptConfigPluginFilters};
use farmfe_core::context::CompilationContext;
use farmfe_core::module::ModuleType;
use farmfe_core::plugin::PluginProcessModuleHookParam;
use farmfe_core::swc_common::errors::{DiagnosticId, HANDLER};
use farmfe_core::swc_common::plugin::metadata::TransformPluginMetadataContext;
use farmfe_core::swc_common::{self, FileName, Mark};
use farmfe_core::{parking_lot, swc_ecma_ast::*};
use farmfe_toolkit::anyhow::{self, Context, Result};
use farmfe_toolkit::swc_atoms::Atom;
use farmfe_toolkit::swc_ecma_visit::{noop_fold_type, Fold, FoldWith};
use once_cell::sync::Lazy;
use swc_ecma_loader::{
  resolve::Resolve,
  resolvers::{lru::CachingResolver, node::NodeModulesResolver},
};

use swc_plugin_runner::runtime::Runtime as PluginRuntime;

pub fn transform_by_swc_plugins(
  param: &mut PluginProcessModuleHookParam,
  plugin_runtime: Arc<dyn PluginRuntime>,
  context: &Arc<CompilationContext>,
) -> Result<()> {
  let mut plugins_should_execute = vec![];

  let plugins = &context.config.script.plugins;
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
  }

  if plugins_should_execute.is_empty() {
    return Ok(());
  }

  let transform_metadata_context = Arc::new(TransformPluginMetadataContext::new(
    Some(param.module_id.to_string()),
    context.config.mode.to_string(),
    None,
  ));
  let unresolved_mark = Mark::from_u32(param.meta.as_script().unresolved_mark);
  let cm = context.meta.get_module_source_map(&param.module_id);
  let comments = param.meta.as_script().comments.clone().into();
  let mut plugin_transforms = swc_plugins(
    Some(plugins_should_execute),
    None,
    transform_metadata_context,
    Some(comments),
    cm,
    unresolved_mark,
    plugin_runtime,
  );

  let mut program = Program::Module(param.meta.as_script_mut().take_ast());
  // Fold module
  program = program.fold_with(&mut plugin_transforms);

  param.meta.as_script_mut().set_ast(program.expect_module());
  Ok(())
}

/// A shared instance to plugin's module bytecode cache.
pub static PLUGIN_MODULE_CACHE: Lazy<swc_plugin_runner::cache::PluginModuleCache> =
  Lazy::new(Default::default);

/// Create a new cache instance if not initialized. This can be called multiple
/// time, but any subsequent call will be ignored.
///
/// This fn have a side effect to create path to cache if given path is not
/// resolvable if fs_cache_store is enabled. If root is not specified, it'll
/// generate default root for cache location.
///
/// If cache failed to initialize filesystem cache for given location
/// it'll be serve in-memory cache only.
pub fn init_plugin_module_cache_once(
  enable_fs_cache_store: bool,
  fs_cache_store_root: Option<&str>,
) {
  PLUGIN_MODULE_CACHE.inner.get_or_init(|| {
    parking_lot::Mutex::new(swc_plugin_runner::cache::PluginModuleCache::create_inner(
      enable_fs_cache_store,
      fs_cache_store_root,
    ))
  });
}

pub(crate) fn swc_plugins(
  configured_plugins: Option<Vec<ScriptConfigPlugin>>,
  plugin_env_vars: Option<Vec<Atom>>,
  metadata_context: std::sync::Arc<swc_common::plugin::metadata::TransformPluginMetadataContext>,
  comments: Option<swc_common::comments::SingleThreadedComments>,
  source_map: std::sync::Arc<swc_common::SourceMap>,
  unresolved_mark: swc_common::Mark,
  plugin_runtime: Arc<dyn PluginRuntime>,
) -> impl Fold {
  RustPlugins {
    plugins: configured_plugins,
    plugin_env_vars: plugin_env_vars.map(std::sync::Arc::new),
    metadata_context,
    comments,
    source_map,
    unresolved_mark,
    plugin_runtime,
  }
}

struct RustPlugins {
  plugins: Option<Vec<ScriptConfigPlugin>>,
  plugin_env_vars: Option<std::sync::Arc<Vec<Atom>>>,
  metadata_context: std::sync::Arc<swc_common::plugin::metadata::TransformPluginMetadataContext>,
  comments: Option<swc_common::comments::SingleThreadedComments>,
  source_map: std::sync::Arc<swc_common::SourceMap>,
  unresolved_mark: swc_common::Mark,
  plugin_runtime: Arc<dyn PluginRuntime>,
}

impl RustPlugins {
  fn apply(&mut self, n: Program) -> Result<Program, anyhow::Error> {
    use anyhow::Context;
    if self.plugins.is_none() || self.plugins.as_ref().unwrap().is_empty() {
      return Ok(n);
    }

    let filename = self.metadata_context.filename.clone();

    let fut = async move { self.apply_inner(n) };
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
      handle.block_on(fut)
    } else {
      tokio::runtime::Runtime::new().unwrap().block_on(fut)
    }
    .with_context(|| format!("failed to invoke plugin on '{filename:?}'"))
  }

  fn apply_inner(&mut self, n: Program) -> Result<Program, anyhow::Error> {
    use anyhow::Context;
    use swc_common::plugin::serialized::PluginSerializedBytes;

    // swc_plugin_macro will not inject proxy to the comments if comments is empty
    let should_enable_comments_proxy = self.comments.is_some();

    // Set comments once per whole plugin transform execution.
    swc_plugin_proxy::COMMENTS.set(
      &swc_plugin_proxy::HostCommentsStorage {
        inner: self.comments.clone(),
      },
      || {
        let program = swc_common::plugin::serialized::VersionedSerializable::new(n);
        let mut serialized = PluginSerializedBytes::try_serialize(&program)?;

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
              .get(&*self.plugin_runtime, &p.name)
              .expect("plugin module should be loaded");

            let plugin_name = plugin_module_bytes.get_module_name().to_string();

            let mut transform_plugin_executor = swc_plugin_runner::create_plugin_transform_executor(
              &self.source_map,
              &self.unresolved_mark,
              &self.metadata_context,
              self.plugin_env_vars.clone(),
              plugin_module_bytes,
              Some(p.options),
              self.plugin_runtime.clone(),
            );

            serialized = transform_plugin_executor
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
      },
    )
  }
}

impl Fold for RustPlugins {
  noop_fold_type!();

  fn fold_module(&mut self, n: Module) -> Module {
    match self.apply(Program::Module(n)) {
      Ok(program) => program.expect_module(),
      Err(err) => {
        HANDLER.with(|handler| {
          handler.err_with_code(&err.to_string(), DiagnosticId::Error("plugin".into()));
        });
        Module::default()
      }
    }
  }

  fn fold_script(&mut self, n: Script) -> Script {
    match self.apply(Program::Script(n)) {
      Ok(program) => program.expect_script(),
      Err(err) => {
        HANDLER.with(|handler| {
          handler.err_with_code(&err.to_string(), DiagnosticId::Error("plugin".into()));
        });
        Script::default()
      }
    }
  }
}

pub(crate) fn compile_wasm_plugins(
  cache_root: Option<&str>,
  plugins: &[ScriptConfigPlugin],
  plugin_runtime: &dyn PluginRuntime,
) -> Result<()> {
  let plugin_resolver = CachingResolver::new(
    40,
    NodeModulesResolver::new(swc_ecma_loader::TargetEnv::Node, Default::default(), true),
  );

  // Currently swc enables filesystem cache by default on Embedded runtime plugin
  // target.
  init_plugin_module_cache_once(cache_root.is_some(), cache_root);

  let mut inner_cache = PLUGIN_MODULE_CACHE
    .inner
    .get()
    .expect("Cache should be available")
    .lock();

  // Populate cache to the plugin modules if not loaded
  for plugin_config in plugins.iter() {
    let plugin_name = &plugin_config.name;

    if !inner_cache.contains(plugin_runtime, plugin_name) {
      let resolved_path = plugin_resolver
        .resolve(&FileName::Real(PathBuf::from(plugin_name)), plugin_name)
        .with_context(|| format!("failed to resolve plugin path: {plugin_name}"))?;

      let path = if let FileName::Real(value) = resolved_path.filename {
        value
      } else {
        anyhow::bail!("Failed to resolve plugin path: {:?}", resolved_path);
      };

      inner_cache.store_bytes_from_path(plugin_runtime, &path, plugin_name)?;
    }
  }

  Ok(())
}

fn should_execute_swc_plugin(
  resolved_path: String,
  module_type: ModuleType,
  filters: &ScriptConfigPluginFilters,
) -> bool {
  // transform it to Regex first and test against it
  let resolve_paths_regex = &filters.resolved_paths;

  resolve_paths_regex
    .iter()
    .any(|r| r.is_match(&resolved_path))
    || filters.module_types.contains(&module_type)
}
