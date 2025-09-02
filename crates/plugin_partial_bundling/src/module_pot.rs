use farmfe_core::module::{ModuleId, ModuleType};
use farmfe_core::HashSet;

use crate::generate_module_pots::ModulePotSourceType;

#[derive(Debug, Clone)]
/// A ModulePot is a collection of modules in the same ModuleBucket that satisfy following rules:
/// 1. Modules matched partialBundling.groups will be in the same ModulePot.
/// 2. Modules in the same immutable package are in the same ModulePot. For example, A, B are both in ModuleBucket_A_B and they are also in the same immutable package, then A, B would be in the same Module Pot.
/// 3. For other modules, a module is a module pot
pub struct ModulePot {
  pub name: String,
  pub source_type: ModulePotSourceType,
  pub size: usize,
  pub module_type: ModuleType,
  pub immutable: bool,
  pub execution_order: usize,
  pub modules: HashSet<ModuleId>,
}

impl ModulePot {
  pub fn new(
    name: String,
    source_type: ModulePotSourceType,
    module_type: ModuleType,
    immutable: bool,
  ) -> Self {
    Self {
      name,
      source_type,
      modules: HashSet::default(),
      size: 0,
      module_type,
      immutable,
      execution_order: usize::MAX,
    }
  }

  pub fn gen_id(name: &str, module_type: ModuleType, immutable: bool) -> String {
    format!("{}_{}_{}", name, module_type.to_string(), immutable)
  }

  pub fn add_module(&mut self, module_id: ModuleId, size: usize, execution_order: usize) {
    self.modules.insert(module_id);
    self.size += size;
    self.execution_order = self.execution_order.min(execution_order);
  }

  pub fn modules(&self) -> &HashSet<ModuleId> {
    &self.modules
  }

  pub fn take_modules(self) -> HashSet<ModuleId> {
    self.modules
  }
}
