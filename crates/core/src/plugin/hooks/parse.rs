use std::sync::Arc;

use crate::module::{ModuleId, ModuleType};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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
