use std::sync::Arc;

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::{CompilationError, Result},
  plugin::{
    Plugin, PluginLoadHookParam, PluginLoadHookResult, PluginResolveHookParam,
    PluginResolveHookResult, PluginTransformHookParam, PluginTransformHookResult,
  },
};

use libloading::Library;

use self::plugin_loader::load_rust_plugin;

pub mod plugin_loader;

pub struct RustPluginAdapter {
  /// plugin instance
  plugin: Arc<dyn Plugin>,
  /// dynamic lib of this plugin, this lib should created and destroyed with the plugin instance as the same time
  _lib: Library,
}

impl RustPluginAdapter {
  pub fn new(plugin_path: &String, config: &Config) -> Result<Self> {
    let (plugin, _lib) = unsafe {
      load_rust_plugin(plugin_path, config).map_err(|e| {
        CompilationError::GenericError(format!("Load rust plugin {} failed. {:?}", plugin_path, e))
      })?
    };

    Ok(Self { plugin, _lib })
  }
}

/// Proxy to self.plugin.<hook>, remember to sync hooks here when add new hooks
impl Plugin for RustPluginAdapter {
  fn name(&self) -> &str {
    self.plugin.name()
  }

  fn priority(&self) -> i32 {
    self.plugin.priority()
  }

  fn build_start(&self, context: &Arc<CompilationContext>) -> Result<Option<()>> {
    self.plugin.build_start(context)
  }

  fn resolve(
    &self,
    param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<PluginResolveHookResult>> {
    self.plugin.resolve(param, context)
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<PluginLoadHookResult>> {
    self.plugin.load(param, context)
  }

  fn transform(
    &self,
    param: &PluginTransformHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<PluginTransformHookResult>> {
    self.plugin.transform(param, context)
  }

  fn parse(
    &self,
    param: &farmfe_core::plugin::PluginParseHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<farmfe_core::module::Module>> {
    self.plugin.parse(param, context)
  }

  fn module_parsed(
    &self,
    module: &mut farmfe_core::module::Module,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.module_parsed(module, context)
  }

  fn analyze_deps(
    &self,
    param: &mut farmfe_core::plugin::PluginAnalyzeDepsHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.analyze_deps(param, context)
  }

  fn build_end(&self, context: &Arc<CompilationContext>) -> Result<Option<()>> {
    self.plugin.build_end(context)
  }
}
