use crate::{module::ModuleId, resource::ResourceType, HashMap};

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
  #[serde(rename = "added")]
  Added,
  // updated a module
  #[serde(rename = "updated")]
  Updated,
  // removed a module
  #[serde(rename = "removed")]
  Removed,
}

impl From<String> for UpdateType {
  fn from(s: String) -> Self {
    match s.as_str() {
      "added" => Self::Added,
      "updated" => Self::Updated,
      "removed" => Self::Removed,
      _ => unreachable!("invalid update type"),
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginUpdateModulesHookParam {
  pub paths: Vec<(String, UpdateType)>,
}
