use std::sync::Arc;

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{module_graph::ModuleGraph, ModuleId, ModuleMetaData},
  plugin::{
    Plugin, PluginFinalizeResourcesHookParam, PluginGenerateResourcesHookResult, PluginHookContext,
    PluginLoadHookParam, PluginLoadHookResult, PluginProcessModuleHookParam,
    PluginResolveHookParam, PluginResolveHookResult, PluginTransformHookParam,
    PluginTransformHookResult,
  },
  resource::{meta_data::ResourcePotMetaData, resource_pot::ResourcePot},
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
  pub fn new(plugin_path: &String, config: &Config, options: String) -> Result<Self> {
    let (plugin, _lib) = unsafe {
      load_rust_plugin(plugin_path, config, options).map_err(|e| {
        CompilationError::GenericError(format!("Load rust plugin {plugin_path} failed. {e:?}"))
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

  fn plugin_cache_loaded(
    &self,
    cache: &Vec<u8>,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.plugin_cache_loaded(cache, context)
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
  ) -> Result<Option<ModuleMetaData>> {
    self.plugin.parse(param, context, hook_context)
  }

  fn process_module(
    &self,
    param: &mut PluginProcessModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.process_module(param, context)
  }

  fn analyze_deps(
    &self,
    param: &mut farmfe_core::plugin::PluginAnalyzeDepsHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.analyze_deps(param, context)
  }

  fn freeze_module(
    &self,
    param: &mut farmfe_core::plugin::hooks::freeze_module::PluginFreezeModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.freeze_module(param, context)
  }

  fn module_graph_build_end(
    &self,
    module_graph: &mut ModuleGraph,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.module_graph_build_end(module_graph, context)
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

  fn freeze_module_graph_meta(
    &self,
    module_graph: &mut ModuleGraph,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.freeze_module_graph_meta(module_graph, context)
  }

  fn analyze_module_graph(
    &self,
    module_graph: &mut ModuleGraph,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> Result<Option<farmfe_core::module::module_group::ModuleGroupGraph>> {
    self
      .plugin
      .analyze_module_graph(module_graph, context, hook_context)
  }

  fn partial_bundling(
    &self,
    modules: &Vec<ModuleId>,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> Result<Option<Vec<ResourcePot>>> {
    self.plugin.partial_bundling(modules, context, hook_context)
  }

  fn process_resource_pots(
    &self,
    resource_pots: &mut Vec<&mut ResourcePot>,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.process_resource_pots(resource_pots, context)
  }

  fn render_start(&self, config: &Config, context: &Arc<CompilationContext>) -> Result<Option<()>> {
    self.plugin.render_start(config, context)
  }

  fn render_resource_pot(
    &self,
    resource_pot: &ResourcePot,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> Result<Option<ResourcePotMetaData>> {
    self
      .plugin
      .render_resource_pot(resource_pot, context, hook_context)
  }

  fn process_rendered_resource_pot(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self
      .plugin
      .process_rendered_resource_pot(resource_pot, context)
  }

  fn augment_resource_pot_hash(
    &self,
    render_pot: &ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<String>> {
    self.plugin.augment_resource_pot_hash(render_pot, context)
  }

  fn optimize_resource_pot(
    &self,
    resource_pot: &mut farmfe_core::resource::resource_pot::ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.optimize_resource_pot(resource_pot, context)
  }

  fn generate_resources(
    &self,
    resource_pot: &mut farmfe_core::resource::resource_pot::ResourcePot,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> Result<Option<PluginGenerateResourcesHookResult>> {
    self
      .plugin
      .generate_resources(resource_pot, context, hook_context)
  }

  fn process_generated_resources(
    &self,
    resources: &mut PluginGenerateResourcesHookResult,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.process_generated_resources(resources, context)
  }

  fn handle_entry_resource(
    &self,
    resource: &mut farmfe_core::plugin::PluginHandleEntryResourceHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.handle_entry_resource(resource, context)
  }

  fn finalize_resources(
    &self,
    param: &mut PluginFinalizeResourcesHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.finalize_resources(param, context)
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

  fn write_plugin_cache(&self, context: &Arc<CompilationContext>) -> Result<Option<Vec<u8>>> {
    self.plugin.write_plugin_cache(context)
  }

  fn config(&self, config: &mut Config) -> Result<Option<()>> {
    self.plugin.config(config)
  }

  fn finalize_module(
    &self,
    param: &mut farmfe_core::plugin::PluginFinalizeModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.finalize_module(param, context)
  }

  fn update_modules(
    &self,
    params: &mut farmfe_core::plugin::PluginUpdateModulesHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.update_modules(params, context)
  }

  fn module_graph_updated(
    &self,
    param: &farmfe_core::plugin::PluginModuleGraphUpdatedHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.plugin.module_graph_updated(param, context)
  }

  fn update_finished(&self, context: &Arc<CompilationContext>) -> Result<Option<()>> {
    self.plugin.update_finished(context)
  }

  fn handle_persistent_cached_module(
    &self,
    module: &farmfe_core::module::Module,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<bool>> {
    self.plugin.handle_persistent_cached_module(module, context)
  }
}
