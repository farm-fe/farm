use std::{any::Any, collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  context::CompilationContext,
  error::Result,
  module::{
    module_graph::ModuleGraph,
    module_group::{ModuleGroup, ModuleGroupMap},
    Module, ModuleType,
  },
  resource::{
    resource_graph::{self, ResourceGraph},
    Resource,
  },
  stats::Stats,
};

pub mod plugin_driver;

pub const DEFAULT_PRIORITY: i32 = 100;

pub trait Plugin: Any + Send + Sync {
  fn name(&self) -> &str;

  fn priority(&self) -> i32 {
    DEFAULT_PRIORITY
  }

  fn build_start(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    Ok(None)
  }

  fn resolve(
    &self,
    _param: &PluginResolveHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<PluginResolveHookResult>> {
    Ok(None)
  }

  fn load(
    &self,
    _param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
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
  ) -> Result<Option<Module>> {
    Ok(None)
  }

  fn module_parsed(
    &self,
    _module: &mut Module,
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
    _module_graph: &RwLock<ModuleGraph>,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// Analyze module group based on module graph
  fn analyze_module_graph(
    &self,
    _module_graph: &RwLock<ModuleGraph>,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<ModuleGroupMap>> {
    Ok(None)
  }

  /// Merging modules of the module group map to [crate::resource::resource_graph::ResourceGraph]
  fn merge_modules(
    &self,
    _module_group: &ModuleGroupMap,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<ResourceGraph>> {
    Ok(None)
  }

  /// process resource graph before render and generating each resource
  fn process_resource_graph(
    &self,
    _resource_graph: &RwLock<ResourceGraph>,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// Render the [Resource] in [ResourceGraph].
  /// May merge the module's ast in the same resource to a single ast and transform the output format to custom module system and ESM
  fn render_resource(
    &self,
    _resource: &mut Resource,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// Optimize the final resource, for example, minimize every resource in the resource graph
  fn optimize_resource(
    &self,
    _resource: &mut Resource,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// Generate resources based on the [ResourceGraph]
  /// This hook is executed in serial and should update the content inside ResourceGraph
  fn generate_resource(
    &self,
    _resource_graph: &mut Resource,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    Ok(None)
  }

  /// Write the final output [Resource] to disk or not
  fn write_resource(
    &self,
    _resource: &mut Resource,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub enum ResolveKind {
  /// entry input in the config
  Entry,
  /// static import, e.g. `import a from './a'`
  Import,
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
  /// Custom ResolveKind, e.g. `const worker = new Worker(new Url("worker.js"))` of a web worker
  Custom(String),
}

/// Parameter of the resolve hook
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub struct PluginResolveHookParam {
  /// the specifier would like to resolve, for example, './index'
  pub specifier: String,
  /// the start location to resolve `specifier`, being [None] if resolving a entry or resolving a hmr update.
  pub importer: Option<String>,
  /// for example, [ResolveKind::Import] for static import (`import a from './a'`)
  pub kind: ResolveKind,
  /// if this hook is called by the compiler, its value is [None]
  /// if this hook is called by other plugins, its value is set by the caller plugins.
  pub caller: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename = "camelCase", default)]
pub struct PluginResolveHookResult {
  /// resolved id, normally a resolved path.
  pub id: String,
  /// whether this module should be external, if true, the module won't present in the final result
  pub external: bool,
  /// whether this module has side effects, affects tree shaking
  pub side_effects: bool,
  /// the package.json of the resolved id, if [None], using root package.json(where farm.config placed) by default
  pub package_json_info: Option<Value>,
  /// the query parsed from specifier, for example, query should be `{ inline: true }` if specifier is `./a.png?inline`
  /// if you custom plugins, your plugin should be responsible for parsing query
  /// if you just want a normal query parsing like the example above, [crate::utils::parse_query] is for you
  pub query: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub struct PluginLoadHookParam<'a> {
  pub id: &'a str,
  pub query: HashMap<String, String>,
  /// if this hook is called by the compiler, its value is [None]
  /// if this hook is called by other plugins, its value is set by the caller plugins.
  pub caller: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub struct PluginLoadHookResult {
  /// the source content of the module
  pub source: String,
  /// the type of the module, for example [ModuleType::Js] stands for a normal javascript file,
  /// usually end with `.js` extension
  pub module_type: ModuleType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub struct PluginTransformHookParam<'a> {
  /// source content after load or transformed result of previous plugin
  pub source: String,
  /// module type after load
  pub module_type: ModuleType,
  pub id: &'a str,
  pub query: HashMap<String, String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename = "camelCase", default)]
pub struct PluginTransformHookResult {
  /// transformed source content, will be passed to next plugin.
  pub source: String,
  /// you can change the module type after transform.
  pub module_type: Option<ModuleType>,
  /// transformed source map, all plugins' transformed source map will be stored as a source map chain.
  pub source_map: Option<String>,
}

pub struct PluginParseHookParam {
  pub id: String,
  pub query: HashMap<String, String>,
  pub module_type: ModuleType,
  pub source: String,
  pub source_map_chain: Vec<String>,
  pub side_effects: bool,
  pub package_json_info: Value,
  /// if this hook is called by the compiler, its value is [None]
  /// if this hook is called by other plugins, its value is set by the caller plugins.
  pub caller: Option<String>,
}

pub struct PluginAnalyzeDepsHookParam<'a> {
  pub module: &'a Module,
  /// analyzed deps from previous plugins, if you want to analyzer more deps, you must push new entries to it.
  pub deps: Vec<PluginAnalyzeDepsHookResultEntry>,
}

#[derive(Debug)]
pub struct PluginAnalyzeDepsHookResultEntry {
  pub specifier: String,
  pub kind: ResolveKind,
}
