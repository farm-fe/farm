use anyhow::Context;
use farmfe_core::{
  error::{CompilationError, Result},
  serde_json, swc_common,
  swc_ecma_ast::{Program, Module as SwcModule},
};
use farmfe_toolkit::swc_ecma_visit::{noop_fold_type, Fold};
use swc_ecma_loader::{
  resolve::Resolve,
  resolvers::{lru::CachingResolver, node::NodeModulesResolver},
  TargetEnv,
};

// This file is modified from https://github.com/swc-project/swc/tree/main/crates/swc/src/plugin.rs

#[derive(Debug, Clone)]
pub struct PluginConfig(pub String, pub serde_json::Value);

pub fn plugins(
  configured_plugins: Option<Vec<PluginConfig>>,
  metadata_context: std::sync::Arc<swc_common::plugin::metadata::TransformPluginMetadataContext>,
  comments: Option<swc_common::comments::SingleThreadedComments>,
  source_map: std::sync::Arc<swc_common::SourceMap>,
  unresolved_mark: swc_common::Mark,
) -> impl Fold {
  {
    RustPlugins {
      plugins: configured_plugins,
      metadata_context,
      comments,
      source_map,
      unresolved_mark,
      resolver: CachingResolver::new(
        40,
        NodeModulesResolver::new(TargetEnv::Node, Default::default(), true),
      ),
    }
  }
}

struct RustPlugins {
  plugins: Option<Vec<PluginConfig>>,
  metadata_context: std::sync::Arc<swc_common::plugin::metadata::TransformPluginMetadataContext>,
  comments: Option<swc_common::comments::SingleThreadedComments>,
  source_map: std::sync::Arc<swc_common::SourceMap>,
  unresolved_mark: swc_common::Mark,
  resolver: CachingResolver<NodeModulesResolver>,
}

impl RustPlugins {
  fn apply(&mut self, n: Program) -> Result<Program> {
    if self.plugins.is_none() || self.plugins.as_ref().unwrap().is_empty() {
      return Ok(n);
    }

    self.apply_inner(n)
  }

  fn apply_inner(&mut self, n: Program) -> Result<Program> {
    use std::{path::PathBuf, sync::Arc};

    use swc_common::{plugin::serialized::PluginSerializedBytes, FileName};

    // swc_plugin_macro will not inject proxy to the comments if comments is empty
    let should_enable_comments_proxy = self.comments.is_some();

    // Set comments once per whole plugin transform execution.
    swc_plugin_proxy::COMMENTS
      .set(
        &swc_plugin_proxy::HostCommentsStorage {
          inner: self.comments.clone(),
        },
        || {
          let program = swc_common::plugin::serialized::VersionedSerializable::new(n);
          let mut serialized = PluginSerializedBytes::try_serialize(&program)?;

          if let Some(plugins) = &mut self.plugins {
            for p in plugins.drain(..) {
              let resolved_path = self
                .resolver
                .resolve(&FileName::Real(PathBuf::from(&p.0)), &p.0)?;

              let path = if let FileName::Real(value) = resolved_path {
                Arc::new(value)
              } else {
                anyhow::bail!("Failed to resolve plugin path: {:?}", resolved_path);
              };

              let mut transform_plugin_executor =
                swc_plugin_runner::create_plugin_transform_executor(
                  &path,
                  &swc_plugin_runner::cache::PLUGIN_MODULE_CACHE,
                  &self.source_map,
                  &self.metadata_context,
                  Some(p.1),
                )?;

              if !transform_plugin_executor.is_transform_schema_compatible()? {
                anyhow::bail!("Cannot execute incompatible plugin {}", &p.0);
              }

              serialized = transform_plugin_executor
                .transform(
                  &serialized,
                  self.unresolved_mark,
                  should_enable_comments_proxy,
                )
                .with_context(|| {
                  format!(
                    "failed to invoke `{}` as js transform plugin at {}",
                    &p.0,
                    path.display()
                  )
                })?;
            }
          }

          // Plugin transformation is done. Deserialize transformed bytes back
          // into Program
          serialized.deserialize().map(|v| v.into_inner())
        },
      )
      .map_err(|e| CompilationError::GenericError(e.to_string()))
  }
}

impl Fold for RustPlugins {
  noop_fold_type!();

  fn fold_module(&mut self, n: SwcModule) -> SwcModule {
    self.apply(Program::Module(n)).expect("failed to invoke plugin").expect_module()
  }
}
