use std::{any::Any, sync::Arc};

use dashmap::DashMap;
use hashbrown::HashMap;
use parking_lot::{Mutex, RwLock};
use swc_common::{FilePathMapping, Globals, SourceMap};

use crate::{
  cache::CacheManager,
  config::Config,
  error::Result,
  module::{module_graph::ModuleGraph, module_group::ModuleGroupGraph},
  plugin::{plugin_driver::PluginDriver, Plugin},
  resource::{resource_pot_map::ResourcePotMap, Resource},
};

/// Shared context through the whole compilation.
pub struct CompilationContext {
  pub config: Config,
  pub module_graph: RwLock<ModuleGraph>,
  pub module_group_graph: RwLock<ModuleGroupGraph>,
  pub plugin_driver: PluginDriver,
  pub resource_pot_map: RwLock<ResourcePotMap>,
  pub resources_map: Mutex<HashMap<String, Resource>>,
  pub cache_manager: CacheManager,
  pub meta: ContextMetaData,
}

impl CompilationContext {
  pub fn new(config: Config, plugins: Vec<Arc<dyn Plugin>>) -> Result<Self> {
    Ok(Self {
      module_graph: RwLock::new(ModuleGraph::new()),
      module_group_graph: RwLock::new(ModuleGroupGraph::new()),
      resource_pot_map: RwLock::new(ResourcePotMap::new()),
      resources_map: Mutex::new(HashMap::new()),
      config,
      plugin_driver: PluginDriver::new(plugins),
      cache_manager: CacheManager::new(),
      meta: ContextMetaData::new(),
    })
  }
}

/// Shared meta info for the core and core plugins, for example, shared swc [SourceMap]
/// The **custom** field can be used for custom plugins to store shared meta data across compilation
pub struct ContextMetaData {
  // shared meta by core plugins
  pub script: ScriptContextMetaData,
  pub css: CssContextMetaData,
  pub html: HtmlContextMetaData,
  // custom meta map
  pub custom: DashMap<String, Box<dyn Any + Send + Sync>>,
}

impl ContextMetaData {
  pub fn new() -> Self {
    Self {
      script: ScriptContextMetaData::new(),
      css: CssContextMetaData::new(),
      html: HtmlContextMetaData::new(),
      custom: DashMap::new(),
    }
  }
}

impl Default for ContextMetaData {
  fn default() -> Self {
    Self::new()
  }
}

/// Shared script meta data used for [swc]
pub struct ScriptContextMetaData {
  pub cm: Arc<SourceMap>,
  pub globals: Globals,
  pub runtime_ast: RwLock<Option<swc_ecma_ast::Module>>,
}

impl ScriptContextMetaData {
  pub fn new() -> Self {
    Self {
      cm: Arc::new(SourceMap::new(FilePathMapping::empty())),
      globals: Globals::new(),
      runtime_ast: RwLock::new(None),
    }
  }
}

impl Default for ScriptContextMetaData {
  fn default() -> Self {
    Self::new()
  }
}

pub struct CssContextMetaData {
  pub cm: Arc<SourceMap>,
  pub globals: Globals,
}

impl CssContextMetaData {
  pub fn new() -> Self {
    Self {
      cm: Arc::new(SourceMap::new(FilePathMapping::empty())),
      globals: Globals::new(),
    }
  }
}

impl Default for CssContextMetaData {
  fn default() -> Self {
    Self::new()
  }
}

pub struct HtmlContextMetaData {
  pub cm: Arc<SourceMap>,
  pub globals: Globals,
}

impl HtmlContextMetaData {
  pub fn new() -> Self {
    Self {
      cm: Arc::new(SourceMap::new(FilePathMapping::empty())),
      globals: Globals::new(),
    }
  }
}

impl Default for HtmlContextMetaData {
  fn default() -> Self {
    Self::new()
  }
}
