use crate::module::{module_graph::ModuleDepsDiffResult, ModuleId};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginModuleGraphUpdatedHookParam {
  pub added_modules_ids: Vec<ModuleId>,
  pub removed_modules_ids: Vec<ModuleId>,
  pub updated_modules_ids: Vec<ModuleId>,
  pub deps_changes: Vec<(ModuleId, ModuleDepsDiffResult)>,
}
