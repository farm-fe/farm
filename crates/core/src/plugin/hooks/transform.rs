use crate::{module::ModuleType, HashMap};
use std::sync::Arc;

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
