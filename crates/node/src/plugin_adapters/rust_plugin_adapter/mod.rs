use std::sync::Arc;

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::{CompilationError, Result},
  module::module_graph::ModuleGraph,
  parking_lot::RwLock,
  plugin::{
    Plugin, PluginHookContext, PluginLoadHookParam, PluginLoadHookResult, PluginResolveHookParam,
    PluginResolveHookResult, PluginTransformHookParam, PluginTransformHookResult,
  },
  resource::resource_pot_graph::ResourcePotGraph,
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
    hook_context: &PluginHookContext,
  ) -> Result<Option<PluginResolveHookResult>> {
    self.plugin.resolve(param, context, hook_context)
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    self.plugin.load(param, context, hook_context)
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
    hook_context: &PluginHookContext,
  ) -> Result<Option<farmfe_core::module::Module>> {
    self.plugin.parse(param, context, hook_context)
  }

  fn process_module(
    &self,
    module: &mut farmfe_core::module::Module,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.process_module(module, context)
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

  fn generate_start(&self, context: &Arc<CompilationContext>) -> Result<Option<()>> {
    self.plugin.generate_start(context)
  }

  fn optimize_module_graph(
    &self,
    module_graph: &mut ModuleGraph,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.optimize_module_graph(module_graph, context)
  }

  fn analyze_module_graph(
    &self,
    module_graph: &mut ModuleGraph,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> Result<Option<farmfe_core::module::module_group::ModuleGroupMap>> {
    self
      .plugin
      .analyze_module_graph(module_graph, context, hook_context)
  }

  fn merge_modules(
    &self,
    module_group: &farmfe_core::module::module_group::ModuleGroupMap,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> Result<Option<farmfe_core::resource::resource_pot_graph::ResourcePotGraph>> {
    self
      .plugin
      .merge_modules(module_group, context, hook_context)
  }

  fn process_resource_pot_graph(
    &self,
    resource_graph: &mut ResourcePotGraph,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self
      .plugin
      .process_resource_pot_graph(resource_graph, context)
  }

  fn render_resource_pot(
    &self,
    resource: &mut farmfe_core::resource::resource_pot::ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.render_resource_pot(resource, context)
  }

  fn optimize_resource_pot(
    &self,
    resource: &mut farmfe_core::resource::resource_pot::ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.optimize_resource_pot(resource, context)
  }

  fn generate_resources(
    &self,
    resource_pot: &mut farmfe_core::resource::resource_pot::ResourcePot,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> Result<Option<Vec<farmfe_core::resource::Resource>>> {
    self
      .plugin
      .generate_resources(resource_pot, context, hook_context)
  }

  fn write_resources(
    &self,
    resources: &mut Vec<farmfe_core::resource::Resource>,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.write_resources(resources, context)
  }

  fn generate_end(&self, context: &Arc<CompilationContext>) -> Result<Option<()>> {
    self.plugin.generate_end(context)
  }

  fn finish(
    &self,
    stat: &farmfe_core::stats::Stats,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.finish(stat, context)
  }
}
