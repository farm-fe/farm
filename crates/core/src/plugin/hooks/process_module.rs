use std::sync::Arc;

use crate::module::{ModuleId, ModuleMetaData, ModuleType};

pub struct PluginProcessModuleHookParam<'a> {
  pub module_id: &'a ModuleId,
  pub module_type: &'a ModuleType,
  pub content: &'a mut Arc<String>,
  pub meta: &'a mut ModuleMetaData,
  pub source_map_chain: &'a mut Vec<Arc<String>>,
}
