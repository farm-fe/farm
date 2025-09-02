use crate::{module::ModuleType, HashMap};

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
