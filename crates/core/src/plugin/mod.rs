use std::{any::Any, collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
  config::Config,
  context::CompilationContext,
  error::Result,
  module::{
    module_graph::ModuleGraph, module_group::ModuleGroupGraph, Module, ModuleId, ModuleMetaData,
    ModuleType,
  },
  resource::{
    resource_pot::{
      RenderedModule, ResourcePot, ResourcePotId, ResourcePotMetaData, ResourcePotType,
    },
    Resource, ResourceType,
  },
  stats::Stats,
};

pub mod constants;
pub mod plugin_driver;

pub const DEFAULT_PRIORITY: i32 = 100;

pub trait Plugin: Any + Send + Sync {
  fn name(&self) -> &str;

  fn priority(&self) -> i32 {
    DEFAULT_PRIORITY
  }

  fn config(&self, _config: &mut Config) -> Result<Option<()>> {
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

  /// The module graph should be constructed and finalized here
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
  fn process_resource_pots(
    &self,
    _resource_pots: &mut Vec<&mut ResourcePot>,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  fn render_resource_pot_modules(
    &self,
    _resource_pot: &ResourcePot,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<ResourcePotMetaData>> {
    Ok(None)
  }

  /// Render the [ResourcePot] in [ResourcePotMap].
  /// May merge the module's ast in the same resource to a single ast and transform the output format to custom module system and ESM
  fn render_resource_pot(
    &self,
    _resource_pot: &PluginRenderResourcePotHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<PluginRenderResourcePotHookResult>> {
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

  /// Generate resources based on the [ResourcePot], return [Vec<Resource>] represents the final generated files.
  /// For example, a .js file and its corresponding source map file
  fn generate_resources(
    &self,
    _resource_pot: &mut ResourcePot,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<PluginGenerateResourcesHookResult>> {
    Ok(None)
  }

  /// Do some finalization work on the generated resources, for example, add hash to the file name,
  /// or insert the generated resources into html
  fn finalize_resources(
    &self,
    _resources: &mut hashbrown::HashMap<String, Resource>,
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
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ResolveKind {
  /// entry input in the config
  Entry(String),
  /// static import, e.g. `import a from './a'`
  #[default]
  Import,
  /// static export, e.g. `export * from './a'`
  ExportFrom,
  /// dynamic import, e.g. `import('./a').then(module => console.log(module))`
  DynamicImport,
  /// cjs require, e.g. `require('./a')`
  Require,
  /// @import of css, e.g. @import './a.css'
  CssAtImport,
  /// url() of css, e.g. url('./a.png')
  CssUrl,
  /// `<script src="./index.html" />` of html
  ScriptSrc,
  /// `<link href="index.css" />` of html
  LinkHref,
  /// Hmr update
  HmrUpdate,
  /// Custom ResolveKind, e.g. `const worker = new Worker(new Url("worker.js"))` of a web worker
  Custom(String),
}

impl ResolveKind {
  /// dynamic if self is [ResolveKind::DynamicImport] or [ResolveKind::Custom("dynamic:xxx")] (dynamic means the module is loaded dynamically, for example, fetch from network)
  /// used when analyzing module groups
  pub fn is_dynamic(&self) -> bool {
    matches!(self, ResolveKind::DynamicImport)
      || matches!(self, ResolveKind::Custom(c) if c.starts_with("dynamic:"))
  }
}

impl From<&str> for ResolveKind {
  fn from(value: &str) -> Self {
    serde_json::from_str(value).unwrap()
  }
}

impl From<ResolveKind> for String {
  fn from(value: ResolveKind) -> Self {
    serde_json::to_string(&value).unwrap()
  }
}

/// Plugin hook call context, designed for `first type` hook, used to provide info when call plugins from another plugin
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginHookContext {
  /// if this hook is called by the compiler, its value is [None]
  /// if this hook is called by other plugins, its value is set by the caller plugins.
  pub caller: Option<String>,
  /// meta data passed between plugins
  pub meta: HashMap<String, String>,
}

/// Parameter of the resolve hook
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginResolveHookParam {
  /// the source would like to resolve, for example, './index'
  pub source: String,
  /// the start location to resolve `specifier`, being [None] if resolving a entry or resolving a hmr update.
  pub importer: Option<ModuleId>,
  /// for example, [ResolveKind::Import] for static import (`import a from './a'`)
  pub kind: ResolveKind,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginResolveHookResult {
  /// resolved path, normally a absolute file path.
  pub resolved_path: String,
  /// whether this module should be external, if true, the module won't present in the final result
  pub external: bool,
  /// whether this module has side effects, affects tree shaking
  pub side_effects: bool,
  /// the query parsed from specifier, for example, query should be `{ inline: true }` if specifier is `./a.png?inline`
  /// if you custom plugins, your plugin should be responsible for parsing query
  /// if you just want a normal query parsing like the example above, [farmfe_toolkit::resolve::parse_query] should be helpful
  pub query: Vec<(String, String)>,
  /// the meta data passed between plugins and hooks
  pub meta: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginLoadHookResult {
  /// the source content of the module
  pub content: String,
  /// the type of the module, for example [ModuleType::Js] stands for a normal javascript file,
  /// usually end with `.js` extension
  pub module_type: ModuleType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginTransformHookResult {
  /// transformed source content, will be passed to next plugin.
  pub content: String,
  /// you can change the module type after transform.
  pub module_type: Option<ModuleType>,
  /// transformed source map, all plugins' transformed source map will be stored as a source map chain.
  pub source_map: Option<String>,
}

#[derive(Debug)]
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

pub struct PluginAnalyzeDepsHookParam<'a> {
  pub module: &'a Module,
  /// analyzed deps from previous plugins, if you want to analyzer more deps, you must push new entries to it.
  pub deps: Vec<PluginAnalyzeDepsHookResultEntry>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateType {
  // added a new module
  Added,
  // updated a module
  Updated,
  // removed a module
  Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginUpdateModulesHookParams {
  pub paths: Vec<(String, UpdateType)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmptyPluginHookParam {}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmptyPluginHookResult {}

pub struct PluginGenerateResourcesHookResult {
  pub resource: Resource,
  pub source_map: Option<Resource>,
}

/// Compatible with Rollup's ChunkInfo
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourcePotInfoOfPluginRenderResourcePotHookParam {
  pub id: ResourcePotId,
  pub resource_pot_type: ResourcePotType,
  pub content: Arc<String>,
  pub dynamic_imports: Vec<String>,
  pub exports: Vec<String>,
  pub facade_module_id: Option<String>,
  pub file_name: String,
  pub implicitly_loaded_before: Vec<String>,
  pub imports: Vec<String>,
  pub imported_bindings: HashMap<String, Vec<String>>,
  pub is_dynamic_entry: bool,
  pub is_entry: bool,
  pub is_implicit_entry: bool,
  pub map: Option<Arc<String>>,
  pub modules: HashMap<ModuleId, RenderedModule>,
  pub module_ids: Vec<ModuleId>,
  pub name: String,
  pub preliminary_file_name: String,
  pub referenced_files: Vec<String>,
  pub ty: String,
}

impl ResourcePotInfoOfPluginRenderResourcePotHookParam {
  pub fn new(resource_pot: &ResourcePot, context: &Arc<CompilationContext>) -> Self {
    let is_dynamic_entry = resource_pot
      .modules()
      .into_iter()
      .any(|m| context.module_group_graph.read().has(m));
    Self {
      id: resource_pot.id.clone(),
      resource_pot_type: resource_pot.resource_pot_type.clone(),
      content: resource_pot.meta.rendered_content.clone(),
      dynamic_imports: vec![], // TODO
      exports: vec![],         // TODO
      facade_module_id: None,  // TODO
      file_name: if resource_pot.entry_module.is_some() {
        context.config.output.entry_filename.clone()
      } else {
        context.config.output.filename.clone()
      },
      implicitly_loaded_before: vec![],  // TODO
      imports: vec![],                   // TODO
      imported_bindings: HashMap::new(), // TODO
      is_dynamic_entry,
      is_entry: resource_pot.entry_module.is_some(),
      is_implicit_entry: false, // TODO
      map: None,
      modules: resource_pot.meta.rendered_modules.clone(),
      module_ids: resource_pot.modules().into_iter().cloned().collect(),
      name: resource_pot.name.clone(),
      preliminary_file_name: "".to_string(), // TODO
      referenced_files: vec![],              // TODO
      ty: "chunk".to_string(),               // TODO
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginRenderResourcePotHookParam {
  pub content: Arc<String>,
  pub resource_pot_info: ResourcePotInfoOfPluginRenderResourcePotHookParam,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginRenderResourcePotHookResult {
  pub content: String,
  pub source_map: Option<String>,
}

pub struct PluginDriverRenderResourcePotHookResult {
  pub content: Arc<String>,
  pub source_map_chain: Vec<Arc<String>>,
}
