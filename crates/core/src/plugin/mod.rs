use std::{any::Any, sync::Arc};

use crate::{
  config::Config,
  context::CompilationContext,
  error::Result,
  module::{
    module_graph::ModuleGraph, module_group::ModuleGroupGraph, Module, ModuleId, ModuleMetaData,
  },
  resource::{meta_data::ResourcePotMetaData, resource_pot::ResourcePot},
  stats::Stats,
  HashMap,
};

pub mod constants;
pub mod hooks;
pub mod plugin_driver;

pub use hooks::{
  analyze_deps::{PluginAnalyzeDepsHookParam, PluginAnalyzeDepsHookResultEntry},
  finalize_module::PluginFinalizeModuleHookParam,
  finalize_resources::PluginFinalizeResourcesHookParam,
  freeze_module::PluginFreezeModuleHookParam,
  generate_resources::{GeneratedResource, PluginGenerateResourcesHookResult},
  handle_entry_resource::PluginHandleEntryResourceHookParam,
  load::{PluginLoadHookParam, PluginLoadHookResult},
  module_graph_updated::PluginModuleGraphUpdatedHookParam,
  parse::PluginParseHookParam,
  process_module::PluginProcessModuleHookParam,
  resolve::{PluginResolveHookParam, PluginResolveHookResult, ResolveKind},
  transform::{PluginTransformHookParam, PluginTransformHookResult},
  update_modules::{PluginUpdateModulesHookParam, UpdateResult, UpdateType},
};

pub const DEFAULT_PRIORITY: i32 = 100;

pub trait Plugin: Any + Send + Sync {
  fn name(&self) -> &str;

  fn priority(&self) -> i32 {
    DEFAULT_PRIORITY
  }

  fn config(&self, _config: &mut Config) -> Result<Option<()>> {
    Ok(None)
  }

  fn plugin_cache_loaded(
    &self,
    _cache: &Vec<u8>,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  fn build_start(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    Ok(None)
  }

  fn resolve(
    &self,
    _param: &PluginResolveHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<PluginResolveHookResult>> {
    Ok(None)
  }

  fn load(
    &self,
    _param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    Ok(None)
  }

  fn transform(
    &self,
    _param: &PluginTransformHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<PluginTransformHookResult>> {
    Ok(None)
  }

  fn parse(
    &self,
    _param: &PluginParseHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<ModuleMetaData>> {
    Ok(None)
  }

  /// Process the module, especially for updating the ast
  fn process_module(
    &self,
    _param: &mut PluginProcessModuleHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  fn analyze_deps(
    &self,
    _param: &mut PluginAnalyzeDepsHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  fn finalize_module(
    &self,
    _param: &mut PluginFinalizeModuleHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// Freeze the module after module graph is built. You can modify the module here, but after this hook, module level transformation should not be performed any more.
  /// Note that the module still can be updated when optimizing module graph, we use this hook as a end of module level transformation, not all level transformation.
  /// Example: Analyze the module, for example, analyze import/export statements, top level and unresolved identifiers
  fn freeze_module(
    &self,
    _param: &mut PluginFreezeModuleHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// The module graph should be constructed and finalized here
  /// You can modify the module graph here, for example, add or remove modules/edges
  /// If module's ast is updated in this hook, fields on module.meta should update manually too
  fn module_graph_build_end(
    &self,
    _module_graph: &mut ModuleGraph,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// All modules are resolved and the module graph is finalized
  fn build_end(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    Ok(None)
  }

  fn generate_start(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    Ok(None)
  }

  /// Some optimization of the module graph should be performed here, for example, tree shaking
  fn optimize_module_graph(
    &self,
    _module_graph: &mut ModuleGraph,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// Analyze module group based on module graph
  fn analyze_module_graph(
    &self,
    _module_graph: &mut ModuleGraph,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<ModuleGroupGraph>> {
    Ok(None)
  }

  /// partial bundling modules to [Vec<ResourcePot>]
  fn partial_bundling(
    &self,
    _modules: &Vec<ModuleId>,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<Vec<ResourcePot>>> {
    Ok(None)
  }

  /// process resource graph before render and generating each resource
  /// Note that this hook can not be cached, you should not do heavy work in this hook
  fn process_resource_pots(
    &self,
    _resource_pots: &mut Vec<&mut ResourcePot>,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  fn render_start(
    &self,
    _config: &Config,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// Transform rendered bundled code for the given resource_pot
  fn render_resource_pot(
    &self,
    _resource_pot: &ResourcePot,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<ResourcePotMetaData>> {
    Ok(None)
  }

  fn augment_resource_hash(
    &self,
    _render_pot_info: &ResourcePot,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<String>> {
    Ok(None)
  }

  /// Optimize the resource pot, for example, minimize
  fn optimize_resource_pot(
    &self,
    _resource: &mut ResourcePot,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// Generate resources based on the [ResourcePot], return [Resource] and [Option<SourceMap>]
  fn generate_resources(
    &self,
    _resource_pot: &mut ResourcePot,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<PluginGenerateResourcesHookResult>> {
    Ok(None)
  }

  /// Process generated resources after the file name of the resource is hashed
  fn process_generated_resources(
    &self,
    _resources: &mut PluginGenerateResourcesHookResult,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// handle entry resource after all resources are generated and processed.
  /// For example, insert the generated resources into html
  fn handle_entry_resource(
    &self,
    _resource: &mut PluginHandleEntryResourceHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// Do some finalization work on the generated resources, for example, add hash to the file name,
  /// or insert the generated resources into html
  fn finalize_resources(
    &self,
    _param: &mut PluginFinalizeResourcesHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  fn generate_end(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    Ok(None)
  }

  fn finish(&self, _stat: &Stats, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    Ok(None)
  }

  /// Called when calling compiler.update(module_paths).
  /// Useful to do some operations like clearing previous state or ignore some files when performing HMR
  fn update_modules(
    &self,
    _params: &mut PluginUpdateModulesHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// Called when calling compiler.update(module_paths).
  /// Useful to do some operations like modifying the module graph
  fn module_graph_updated(
    &self,
    _param: &PluginModuleGraphUpdatedHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// Called when calling compiler.update(module_paths).
  /// This hook is called after all compilation work is done, including the resources regeneration and finalization.
  fn update_finished(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    Ok(None)
  }

  // Called when hit persistent cache. return false to invalidate the cache
  fn handle_persistent_cached_module(
    &self,
    _module: &Module,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<bool>> {
    Ok(None)
  }

  fn write_plugin_cache(&self, _context: &Arc<CompilationContext>) -> Result<Option<Vec<u8>>> {
    Ok(None)
  }
}

/// Plugin hook call context, designed for `first type` hook, used to provide info when call plugins from another plugin
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct PluginHookContext {
  /// if this hook is called by the compiler, its value is [None]
  /// if this hook is called by other plugins, its value is set by the caller plugins.
  pub caller: Option<String>,
  /// meta data passed between plugins
  pub meta: HashMap<String, String>,
}

impl PluginHookContext {
  fn caller_format<T: AsRef<str>>(name: T) -> String {
    format!("[{}]", name.as_ref())
  }

  pub fn add_caller<T: AsRef<str>>(&self, name: T) -> Option<String> {
    match self.caller.as_ref() {
      Some(c) => Some(format!("{}{}", c, Self::caller_format(name))),
      None => Some(Self::caller_format(name)),
    }
  }
  pub fn contain_caller<T: AsRef<str>>(&self, name: T) -> bool {
    if let Some(ref s) = self.caller {
      s.contains(&Self::caller_format(name))
    } else {
      false
    }
  }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmptyPluginHookParam {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmptyPluginHookResult {}
