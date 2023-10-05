use std::{any::Any, sync::Arc};

use dashmap::DashMap;
use hashbrown::HashMap;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use swc_common::{FilePathMapping, Globals, SourceMap};

use crate::{
  cache::CacheManager,
  config::Config,
  error::Result,
  module::{
    module_graph::ModuleGraph, module_group::ModuleGroupGraph, watch_graph::WatchGraph, ModuleId,
  },
  plugin::{plugin_driver::PluginDriver, Plugin},
  record::RecordManager,
  resource::{resource_pot_map::ResourcePotMap, Resource, ResourceOrigin, ResourceType},
};

use self::log_store::LogStore;

pub mod log_store;

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
  pub record_manager: Box<RecordManager>,
  pub log_store: Box<RwLock<LogStore>>,
}

impl CompilationContext {
  pub fn new(config: Config, plugins: Vec<Arc<dyn Plugin>>) -> Result<Self> {
    Ok(Self {
      watch_graph: Box::new(RwLock::new(WatchGraph::new())),
      module_graph: Box::new(RwLock::new(ModuleGraph::new())),
      module_group_graph: Box::new(RwLock::new(ModuleGroupGraph::new())),
      resource_pot_map: Box::new(RwLock::new(ResourcePotMap::new())),
      resources_map: Box::new(Mutex::new(HashMap::new())),
      plugin_driver: Box::new(PluginDriver::new(plugins, config.record)),
      config: Box::new(config),
      cache_manager: Box::new(CacheManager::new()),
      meta: Box::new(ContextMetaData::new()),
      record_manager: Box::new(RecordManager::new()),
      log_store: Box::new(RwLock::new(LogStore::new())),
    })
  }

  pub fn add_watch_files(&self, from: String, deps: Vec<&String>) -> Result<()> {
    // @import 'variable.scss'
    // @import './variable.scss'
    let mut watch_graph = self.watch_graph.write();

    watch_graph.add_node(from.clone());

    for dep in deps {
      watch_graph.add_node(dep.clone());
      watch_graph.add_edge(&from, dep)?;
    }

    Ok(())
  }

  pub fn emit_file(&self, params: EmitFileParams) {
    let mut resources_map = self.resources_map.lock();

    resources_map.insert(
      params.name.clone(),
      Resource {
        name: params.name,
        bytes: params.content,
        emitted: false,
        resource_type: params.resource_type,
        origin: ResourceOrigin::Module(ModuleId::new(&params.resolved_path, "", &self.config.root)),
      },
    );
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmitFileParams {
  pub resolved_path: String,
  pub name: String,
  pub content: Vec<u8>,
  pub resource_type: ResourceType,
}

#[cfg(test)]
mod tests {

  mod add_watch_files {

    use super::super::CompilationContext;

    #[test]
    fn file_as_root_and_dep() {
      let context = CompilationContext::default();
      let vc = "./v_c".to_string();
      let vd = "./v_d".to_string();
      let a = "./a".to_string();

      context.add_watch_files(a.clone(), vec![&vc, &vd]).unwrap();

      context.add_watch_files(vc.clone(), vec![&vd]).unwrap();

      let watch_graph = context.watch_graph.read();

      assert_eq!(watch_graph.relation_roots(&vc), vec![&a]);
      let mut r = watch_graph.relation_roots(&vd);
      r.sort();
      assert_eq!(r, vec![&a, &vc]);
    }
  }
}
