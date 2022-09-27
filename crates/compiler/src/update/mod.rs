use farmfe_core::module::ModuleId;

/// The output after the updating process
#[derive(Debug, Default)]
pub struct UpdateOutput {
  added_module_ids: Vec<ModuleId>,
  updated_module_ids: Vec<ModuleId>,
  removed_module_ids: Vec<ModuleId>,
  resources: String,
}
