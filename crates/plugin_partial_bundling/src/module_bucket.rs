use farmfe_core::{
  module::{module_group::ModuleGroupId, Module, ModuleId, ModuleType},
  HashSet,
};

/// A ModuleBucket is a collection of modules whose module_groups field is equal.
/// For example, if there are two ModuleGroups A and B. if module c is in ModuleGroup A and ModuleGroup B, module d is only in ModuleGroup A, then c and d are in the different ModuleBucket.
///
/// A ModuleBucket can generate multiple ResourcePots.
#[derive(Debug)]
pub struct ModuleBucket {
  pub id: String,
  /// The size of this ModuleBucket. It's the sum of the modules' size.
  pub size: usize,
  /// The type of this ModuleBucket. All modules in this ModuleBucket have the same type.
  pub module_type: ModuleType,
  /// Whether this ModuleBucket is immutable. All modules in this ModuleBucket have the same immutable value.
  pub immutable: bool,

  /// The modules whose ModuleGroups is the same.
  modules: HashSet<ModuleId>,
  /// The ModuleGroups
  module_groups: HashSet<ModuleGroupId>,
}

impl ModuleBucket {
  pub fn new(id: String, module: &Module) -> Self {
    // The fields will be filled later when add modules to this ModuleBucket.
    Self {
      id,
      modules: HashSet::from_iter([module.id.clone()]),
      module_groups: module.module_groups.clone(),
      size: module.size,
      module_type: module.module_type.clone(),
      immutable: module.immutable,
    }
  }

  /// Generate the key of a ModuleBucket.
  pub fn id(module: &Module) -> String {
    let mut group_key = module
      .module_groups
      .iter()
      .map(|module_group_id| module_group_id.to_string())
      .collect::<Vec<String>>();
    // Sort the module_groups to make sure the key is stable.
    group_key.sort();
    let key = group_key.join("_");

    // The key is formatted by module_type, immutable and group_key.
    format!(
      "{}_{}_{}",
      module.module_type.to_string(),
      module.immutable,
      key
    )
  }

  pub fn modules(&self) -> &HashSet<ModuleId> {
    &self.modules
  }

  pub fn module_groups(&self) -> &HashSet<ModuleGroupId> {
    &self.module_groups
  }

  pub fn add_module(&mut self, module: &Module) {
    let module_id = module.id.clone();
    let size = module.size;
    self.size += size;

    self.modules.insert(module_id);
  }
}
