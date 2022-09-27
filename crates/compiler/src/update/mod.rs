use farmfe_core::module::ModuleId;

/// The output after the updating process
#[derive(Debug, Default)]
pub struct UpdateOutput {
  pub added_module_ids: Vec<ModuleId>,
  pub updated_module_ids: Vec<ModuleId>,
  pub removed_module_ids: Vec<ModuleId>,
  pub resources: String,
}
