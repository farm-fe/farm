use std::{
  any::Any,
  path::{Path, PathBuf},
  sync::Arc,
};

use dashmap::{mapref::one::Ref, DashMap};
use parking_lot::{Mutex, RwLock};
use rayon::{ThreadPool, ThreadPoolBuilder};
use regex::Regex;
use serde::{Deserialize, Serialize};
use swc_common::{FileName, Globals, SourceFile, SourceMap};

use crate::{
  cache::CacheManager,
  config::{persistent_cache::PersistentCacheConfig, Config},
  error::Result,
  module::{
    module_graph::ModuleGraph, module_group::ModuleGroupGraph, watch_graph::WatchGraph, ModuleId,
  },
  plugin::{plugin_driver::PluginDriver, Plugin, PluginResolveHookParam, PluginResolveHookResult},
  resource::{
    resource_pot::ResourcePotId, resource_pot_map::ResourcePotMap, Resource, ResourceOrigin,
    ResourceType,
  },
  stats::Stats,
  HashMap,
};

use self::log_store::LogStore;

pub mod log_store;
pub(crate) const EMPTY_STR: &str = "";

lazy_static::lazy_static! {
  pub static ref REPLACE_FILENAME_REGEX: Regex = Regex::new(r"[^a-zA-Z0-9._/\\]").unwrap();
}

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
  pub thread_pool: Arc<ThreadPool>,
  pub meta: Box<ContextMetaData>,
  /// Record stats for the compilation, for example, compilation time, plugin hook time, etc.
  pub stats: Box<Stats>,
  pub log_store: Box<Mutex<LogStore>>,
  pub resolve_cache: Box<Mutex<HashMap<PluginResolveHookParam, PluginResolveHookResult>>>,
  pub custom: Box<DashMap<String, Box<dyn Any + Send + Sync>>>,
}

impl CompilationContext {
  pub fn new(mut config: Config, plugins: Vec<Arc<dyn Plugin>>) -> Result<Self> {
    let (cache_dir, namespace) = Self::normalize_persistent_cache_config(&mut config);

    Ok(Self {
      watch_graph: Box::new(RwLock::new(WatchGraph::new())),
      module_graph: Box::new(RwLock::new(ModuleGraph::new())),
      module_group_graph: Box::new(RwLock::new(ModuleGroupGraph::new())),
      resource_pot_map: Box::new(RwLock::new(ResourcePotMap::new())),
      resources_map: Box::new(Mutex::new(HashMap::default())),
      plugin_driver: Box::new(Self::create_plugin_driver(plugins, config.record)),
      cache_manager: Box::new(CacheManager::new(&cache_dir, &namespace, config.mode)),
      thread_pool: Arc::new(
        ThreadPoolBuilder::new()
          .num_threads(num_cpus::get())
          .build()
          .unwrap(),
      ),
      config: Box::new(config),
      meta: Box::new(ContextMetaData::new()),
      stats: Box::new(Stats::new()),
      log_store: Box::new(Mutex::new(LogStore::new())),
      resolve_cache: Box::new(Mutex::new(HashMap::default())),
      custom: Box::new(DashMap::default()),
    })
  }

  pub fn create_plugin_driver(plugins: Vec<Arc<dyn Plugin>>, record: bool) -> PluginDriver {
    PluginDriver::new(plugins, record)
  }

  pub fn normalize_persistent_cache_config(config: &mut Config) -> (String, String) {
    if config.persistent_cache.enabled() {
      let cache_config_obj = config.persistent_cache.as_obj(&config.root);
      let (cache_dir, namespace) = (
        cache_config_obj.cache_dir.clone(),
        cache_config_obj.namespace.clone(),
      );
      config.persistent_cache = Box::new(PersistentCacheConfig::Obj(cache_config_obj));

      (cache_dir, namespace)
    } else {
      (EMPTY_STR.to_string(), EMPTY_STR.to_string())
    }
  }

  pub fn add_watch_files(&self, from: ModuleId, deps: Vec<ModuleId>) -> Result<()> {
    // @import 'variable.scss'
    // @import './variable.scss'
    let mut watch_graph = self.watch_graph.write();

    watch_graph.add_node(from.clone());

    for dep in deps {
      watch_graph.add_node(dep.clone());
      watch_graph.add_edge(&from, &dep)?;
    }

    Ok(())
  }

  /// get module id from string
  /// 1. if resolved_path is a absolute path, try generate module id from it
  /// 2. if resolved_path is a relative path, treat it as module id
  pub fn str_to_module_id(&self, id: &str) -> ModuleId {
    let is_absolute = Path::new(id).is_absolute();
    if is_absolute {
      let (resolved_path, query) = id.split_once('?').unwrap_or((id, EMPTY_STR));
      ModuleId::new(resolved_path, query, &self.config.root)
    } else {
      ModuleId::from(id)
    }
  }

  pub fn emit_file(&self, params: EmitFileParams) {
    let mut resources_map = self.resources_map.lock();

    let module_id = self.str_to_module_id(&params.resolved_path);

    resources_map.insert(
      params.name.clone(),
      Resource {
        name: params.name,
        bytes: params.content,
        emitted: false,
        should_transform_output_filename: true,
        resource_type: params.resource_type,
        origin: ResourceOrigin::Module(module_id),
        meta: Default::default(),
      },
    );
  }

  pub fn sourcemap_enabled(&self, id: &str) -> bool {
    let immutable = self
      .config
      .partial_bundling
      .immutable_modules
      .iter()
      .any(|im| im.is_match(id));

    self.config.sourcemap.enabled(immutable)
  }

  pub fn get_resolve_cache(
    &self,
    param: &PluginResolveHookParam,
  ) -> Option<PluginResolveHookResult> {
    let resolve_cache = self.resolve_cache.lock();
    resolve_cache.get(param).cloned()
  }

  pub fn set_resolve_cache(&self, param: PluginResolveHookParam, result: PluginResolveHookResult) {
    let mut resolve_cache = self.resolve_cache.lock();
    resolve_cache.insert(param, result);
  }

  pub fn invalidate_module(&self, module_id: &ModuleId) {
    self.cache_manager.module_cache.invalidate_cache(module_id);
  }

  pub fn clear_log_store(&self) {
    let mut log_store = self.log_store.lock();
    log_store.clear();
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
  /// shared meta by plugins
  pub script: ScriptContextMetaData,
  pub css: CssContextMetaData,
  pub html: HtmlContextMetaData,

  /// shared swc sourcemap cache for module
  pub module_source_maps: DashMap<ModuleId, Arc<SourceMap>>,
  /// shared swc sourcemap cache for hoisted modules
  pub hoisted_modules_source_maps: DashMap<ModuleId, Arc<SourceMap>>,
  /// shared swc sourcemap cache for resource pot
  pub resource_pot_source_maps: DashMap<ResourcePotId, Arc<SourceMap>>,

  /// Globals map for each module
  globals_map: DashMap<ModuleId, Globals>,

  /// custom meta map
  pub custom: DashMap<String, Box<dyn Any + Send + Sync>>,
}

impl ContextMetaData {
  pub fn new() -> Self {
    Self {
      script: ScriptContextMetaData::new(),
      css: CssContextMetaData::new(),
      html: HtmlContextMetaData::new(),
      module_source_maps: DashMap::new(),
      hoisted_modules_source_maps: DashMap::new(),
      resource_pot_source_maps: DashMap::new(),
      globals_map: DashMap::new(),
      custom: DashMap::new(),
    }
  }

  /// get swc source map from module id
  pub fn get_module_source_map(&self, module_id: &ModuleId) -> Arc<SourceMap> {
    self
      .module_source_maps
      .get(module_id)
      .map(|value| value.clone())
      .unwrap_or_else(|| panic!("no source map found for module {:?}", module_id))
  }

  /// set swc source map for module id
  /// this should be called after every time the module is parsed and updated to the module graph
  pub fn set_module_source_map(&self, module_id: &ModuleId, cm: Arc<SourceMap>) {
    self.module_source_maps.insert(module_id.clone(), cm);
  }

  pub fn get_hoisted_modules_source_map(&self, module_id: &ModuleId) -> Arc<SourceMap> {
    self
      .hoisted_modules_source_maps
      .get(module_id)
      .map(|value| value.clone())
      .unwrap()
  }

  pub fn set_hoisted_modules_source_map(&self, module_id: &ModuleId, cm: Arc<SourceMap>) {
    self
      .hoisted_modules_source_maps
      .insert(module_id.clone(), cm);
  }

  pub fn get_resource_pot_source_map(&self, resource_pot_id: &ResourcePotId) -> Arc<SourceMap> {
    self
      .resource_pot_source_maps
      .get(resource_pot_id)
      .map(|value| value.clone())
      .unwrap_or_else(|| panic!("no source map found for resource pot {:?}", resource_pot_id))
  }

  /// set swc source map for resource pot
  /// this should be called after every time the resource pot is parsed and updated to the resource pot map
  pub fn set_resource_pot_source_map(&self, resource_pot_id: &ResourcePotId, cm: Arc<SourceMap>) {
    self
      .resource_pot_source_maps
      .insert(resource_pot_id.clone(), cm);
  }

  pub fn set_globals(&self, module_id: &ModuleId, globals: Globals) {
    self.globals_map.insert(module_id.clone(), globals);
  }

  pub fn get_globals(&self, module_id: &ModuleId) -> Ref<ModuleId, Globals> {
    let globals = self
      .globals_map
      .get(module_id)
      .unwrap_or_else(|| panic!("no globals found for module {:?}", module_id));

    globals
  }
}

impl Default for ContextMetaData {
  fn default() -> Self {
    Self::new()
  }
}

/// get swc source map filename from module id.
/// you can get module id from sourcemap filename too, by
pub fn get_swc_sourcemap_filename(module_id: &ModuleId) -> FileName {
  // replace all invalid characters to _ to make sure the filename is valid
  let module_id = module_id.to_string();
  let module_id = REPLACE_FILENAME_REGEX.replace_all(&module_id, "_");
  FileName::Real(PathBuf::from(module_id.to_string()))
}

/// create a swc source map from a source
pub fn create_swc_source_map(
  id: &ModuleId,
  content: Arc<String>,
) -> (Arc<SourceMap>, Arc<SourceFile>) {
  let cm = Arc::new(SourceMap::default());
  let sf = cm.new_source_file_from(Arc::new(get_swc_sourcemap_filename(id)), content);

  (cm, sf)
}

/// Shared script meta data used for [swc]
pub struct ScriptContextMetaData {}

impl ScriptContextMetaData {
  pub fn new() -> Self {
    Self {}
  }
}

impl Default for ScriptContextMetaData {
  fn default() -> Self {
    Self::new()
  }
}

pub struct CssContextMetaData {}

impl CssContextMetaData {
  pub fn new() -> Self {
    Self {}
  }
}

impl Default for CssContextMetaData {
  fn default() -> Self {
    Self::new()
  }
}

pub struct HtmlContextMetaData {}

impl HtmlContextMetaData {
  pub fn new() -> Self {
    Self {}
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

    use crate::module::ModuleId;

    use super::super::CompilationContext;

    #[test]
    fn file_as_root_and_dep() {
      let context = CompilationContext::default();
      let vc: ModuleId = "./v_c".into();
      let vd: ModuleId = "./v_d".into();
      let a: ModuleId = "./a".into();

      context
        .add_watch_files(a.clone(), vec![vc.clone(), vd.clone()])
        .unwrap();

      context
        .add_watch_files(vc.clone(), vec![vd.clone()])
        .unwrap();

      let watch_graph = context.watch_graph.read();

      assert_eq!(watch_graph.relation_roots(&vc), vec![&a]);
      let mut r = watch_graph.relation_roots(&vd);
      r.sort();
      assert_eq!(r, vec![&a, &vc]);
    }
  }
}
