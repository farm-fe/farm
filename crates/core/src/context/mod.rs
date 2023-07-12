use std::{any::Any, path::Path, sync::Arc};

use dashmap::DashMap;
use hashbrown::HashMap;
use parking_lot::{Mutex, RwLock};
use relative_path::RelativePath;
use swc_common::{FilePathMapping, Globals, SourceMap};

use crate::{
  cache::CacheManager,
  config::Config,
  error::Result,
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdge},
    module_group::ModuleGroupGraph,
    watch_graph::WatchGraph,
    Module, ModuleId,
  },
  plugin::{plugin_driver::PluginDriver, Plugin},
  resource::{resource_pot_map::ResourcePotMap, Resource},
};

/// Shared context through the whole compilation.
pub struct CompilationContext {
  pub config: Box<Config>,
  pub watch_graph: Box<RwLock<WatchGraph>>,
  pub module_graph: Box<RwLock<ModuleGraph>>,
  pub module_group_graph: Box<RwLock<ModuleGroupGraph>>,
  pub plugin_driver: Box<PluginDriver>,
  pub resource_pot_map: Box<RwLock<ResourcePotMap>>,
  pub resources_map: Box<Mutex<HashMap<String, Resource>>>,
  pub cache_manager: Box<CacheManager>,
  pub meta: Box<ContextMetaData>,
}

impl CompilationContext {
  pub fn new(config: Config, plugins: Vec<Arc<dyn Plugin>>) -> Result<Self> {
    Ok(Self {
      watch_graph: Box::new(RwLock::new(WatchGraph::new())),
      module_graph: Box::new(RwLock::new(ModuleGraph::new())),
      module_group_graph: Box::new(RwLock::new(ModuleGroupGraph::new())),
      resource_pot_map: Box::new(RwLock::new(ResourcePotMap::new())),
      resources_map: Box::new(Mutex::new(HashMap::new())),
      config: Box::new(config),
      plugin_driver: Box::new(PluginDriver::new(plugins)),
      cache_manager: Box::new(CacheManager::new()),
      meta: Box::new(ContextMetaData::new()),
    })
  }

  pub fn add_watch_files(&self, source: String, deps: Vec<String>) -> Result<()> {
    // @import 'variable.scss'
    // @import './variable.scss'
    let mut watch_graph = self.watch_graph.write();

    for dep in deps {
      let source = source.clone();

      let dep_path = if Path::new(&dep).is_absolute() {
        dep.to_string()
      } else {
        RelativePath::new(&self.config.root)
          .relative(
            RelativePath::new(&source)
              .parent()
              .expect("failed parse dirname")
              .join(&dep),
          )
          .normalize()
          .to_string()
      };

      let cwd = &self.config.root;

      let from = ModuleId::new(&source, "", cwd);
      let to = ModuleId::new(dep_path.as_str(), &"", cwd);

      watch_graph.add_node(from.clone());
      watch_graph.add_node(to.clone());

      watch_graph.add_edge(&from, &to)?;
    }

    Ok(())
  }
}

impl Default for CompilationContext {
  fn default() -> Self {
    Self::new(Config::default(), vec![]).unwrap()
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
