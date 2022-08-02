use std::{any::Any, sync::Arc};

use dashmap::DashMap;
use parking_lot::RwLock;

use crate::{
  cache::CacheManager,
  config::Config,
  module::{module_graph::ModuleGraph, module_group::ModuleGroupMap},
  plugin::{plugin_driver::PluginDriver, Plugin},
  resource::resource_pot_graph::ResourcePotGraph,
};

/// Shared context through the whole compilation.
pub struct CompilationContext {
  pub config: Config,
  pub module_graph: RwLock<ModuleGraph>,
  pub module_group_map: ModuleGroupMap,
  pub plugin_driver: PluginDriver,
  pub resource_pot_graph: RwLock<ResourcePotGraph>,
  pub cache_manager: CacheManager,
  pub meta: ContextMetaData,
}

impl CompilationContext {
  pub fn new(config: Config, plugins: Vec<Arc<dyn Plugin>>) -> Self {
    Self {
      module_graph: RwLock::new(ModuleGraph::new()),
      module_group_map: ModuleGroupMap::new(),
      resource_pot_graph: RwLock::new(ResourcePotGraph::new()),
      config,
      plugin_driver: PluginDriver::new(plugins),
      cache_manager: CacheManager::new(),
      meta: ContextMetaData::new(),
    }
  }
}

/// Shared meta info for the core and core plugins, for example, shared swc [SourceMap]
/// The **custom** field can be used for custom plugins to store shared meta data across compilation
pub struct ContextMetaData {
  // shared meta by core plugins
  pub script: ContextScriptMetaData,
  // custom meta map
  pub custom: DashMap<String, Box<dyn Any + Send + Sync>>,
}

impl ContextMetaData {
  pub fn new() -> Self {
    Self {
      script: ContextScriptMetaData::new(),
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
pub struct ContextScriptMetaData {
  pub cm: String,
  pub globals: String,
}

impl ContextScriptMetaData {
  pub fn new() -> Self {
    Self {
      cm: String::new(),
      globals: String::new(),
    }
  }
}

impl Default for ContextScriptMetaData {
  fn default() -> Self {
    Self::new()
  }
}
