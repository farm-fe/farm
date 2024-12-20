use std::{any::Any, sync::Arc};

use farmfe_macro_cache_item::cache_item;
use rkyv::Deserialize;

use crate::{
  config::Config,
  context::CompilationContext,
  error::Result,
  module::{
    module_graph::ModuleGraph, module_group::ModuleGroupGraph, Module, ModuleId, ModuleMetaData,
    ModuleType,
  },
  resource::{meta_data::ResourcePotMetaData, resource_pot::ResourcePot, Resource, ResourceType},
  stats::Stats,
  HashMap,
};

pub mod constants;
pub mod hooks;
pub mod plugin_driver;

pub use hooks::resolve::{PluginResolveHookParam, PluginResolveHookResult, ResolveKind};

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

  /// Freeze the module after all deep dependencies are resolved
  /// You can modify the module here, but after this hook, the module should be immutable
  /// Example: Analyze the module, for example, analyze import/export statements, top level and unresolved identifiers
  fn freeze_module(
    &self,
    _module: &mut Module,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// The module graph should be constructed and finalized here
  /// You can modify the module graph here, for example, add or remove modules/edges
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

  /// Some optimization of the module graph should be performed here, for example, tree shaking, scope hoisting
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
    _resource: &mut PluginHandleEntryResourceHookParams,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// Do some finalization work on the generated resources, for example, add hash to the file name,
  /// or insert the generated resources into html
  fn finalize_resources(
    &self,
    _param: &mut PluginFinalizeResourcesHookParams,
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
    _params: &mut PluginUpdateModulesHookParams,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// Called when calling compiler.update(module_paths).
  /// Useful to do some operations like modifying the module graph
  fn module_graph_updated(
    &self,
    _param: &PluginModuleGraphUpdatedHookParams,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginLoadHookParam<'a> {
  /// the module id string
  pub module_id: String,
  /// the resolved path from resolve hook
  pub resolved_path: &'a str,
  /// the query map
  pub query: Vec<(String, String)>,
  /// the meta data passed between plugins and hooks
  pub meta: HashMap<String, String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginLoadHookResult {
  /// the source content of the module
  pub content: String,
  /// the type of the module, for example [ModuleType::Js] stands for a normal javascript file,
  /// usually end with `.js` extension
  pub module_type: ModuleType,
  /// source map of the module
  pub source_map: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginTransformHookParam<'a> {
  /// the module id string
  pub module_id: String,
  /// source content after load or transformed result of previous plugin
  pub content: String,
  /// module type after load
  pub module_type: ModuleType,
  /// resolved path from resolve hook
  pub resolved_path: &'a str,
  /// query from resolve hook
  pub query: Vec<(String, String)>,
  /// the meta data passed between plugins and hooks
  pub meta: HashMap<String, String>,
  /// source map chain of previous plugins
  pub source_map_chain: Vec<Arc<String>>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginTransformHookResult {
  /// transformed source content, will be passed to next plugin.
  pub content: String,
  /// you can change the module type after transform.
  pub module_type: Option<ModuleType>,
  /// transformed source map, all plugins' transformed source map will be stored as a source map chain.
  pub source_map: Option<String>,
  /// if true, the previous source map chain will be ignored, and the source map chain will be reset to [source_map] returned by this plugin.
  pub ignore_previous_source_map: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PluginParseHookParam {
  /// module id
  pub module_id: ModuleId,
  /// resolved path
  pub resolved_path: String,
  /// resolved query
  pub query: Vec<(String, String)>,
  pub module_type: ModuleType,
  /// source content(after transform)
  pub content: Arc<String>,
}

pub struct PluginProcessModuleHookParam<'a> {
  pub module_id: &'a ModuleId,
  pub module_type: &'a ModuleType,
  pub content: Arc<String>,
  pub meta: &'a mut ModuleMetaData,
}

pub type PluginAnalyzeModuleHookParam<'a> = PluginProcessModuleHookParam<'a>;

#[derive(Clone)]
pub struct PluginAnalyzeDepsHookParam<'a> {
  pub module: &'a Module,
  /// analyzed deps from previous plugins, you can push new entries to it for your plugin.
  pub deps: Vec<PluginAnalyzeDepsHookResultEntry>,
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
#[cache_item]
pub struct PluginAnalyzeDepsHookResultEntry {
  pub source: String,
  pub kind: ResolveKind,
}

pub struct PluginFinalizeModuleHookParam<'a> {
  pub module: &'a mut Module,
  pub deps: &'a Vec<PluginAnalyzeDepsHookResultEntry>,
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct WatchDiffResult {
  pub add: Vec<String>,
  pub remove: Vec<String>,
}

/// The output after the updating process
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateResult {
  pub added_module_ids: Vec<ModuleId>,
  pub updated_module_ids: Vec<ModuleId>,
  pub removed_module_ids: Vec<ModuleId>,
  /// Javascript module map string, the key is the module id, the value is the module function
  /// This code string should be returned to the client side as MIME type `application/javascript`
  pub immutable_resources: String,
  pub mutable_resources: String,
  pub boundaries: HashMap<String, Vec<Vec<String>>>,
  pub dynamic_resources_map: Option<HashMap<ModuleId, Vec<(String, ResourceType)>>>,
  pub extra_watch_result: WatchDiffResult,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum UpdateType {
  // added a new module
  Added,
  // updated a module
  Updated,
  // removed a module
  Removed,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginUpdateModulesHookParams {
  pub paths: Vec<(String, UpdateType)>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginModuleGraphUpdatedHookParams {
  pub added_modules_ids: Vec<ModuleId>,
  pub removed_modules_ids: Vec<ModuleId>,
  pub updated_modules_ids: Vec<ModuleId>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmptyPluginHookParam {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmptyPluginHookResult {}

#[cache_item]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginGenerateResourcesHookResult {
  pub resource: Resource,
  pub source_map: Option<Resource>,
}

pub struct PluginFinalizeResourcesHookParams<'a> {
  pub resources_map: &'a mut HashMap<String, Resource>,
  pub config: &'a Config,
}

pub struct PluginHandleEntryResourceHookParams<'a> {
  pub resource: &'a mut Resource,
  pub module_graph: &'a ModuleGraph,
  pub module_group_graph: &'a ModuleGroupGraph,
  pub entry_module_id: &'a ModuleId,
}
