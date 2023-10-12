use farmfe_core::module::{module_graph::ModuleGraph, ModuleId};

use crate::module_pot::ModulePot;

pub fn create_test_module_pot(
  module_graph: &mut ModuleGraph,
  module_id: &ModuleId,
  size: usize,
  immutable: bool,
) -> ModulePot {
  let module_b = module_graph.module_mut(module_id).unwrap();
  module_b.size = size;
  module_b.immutable = immutable;

  let mut module_pot = ModulePot::new(
    module_b.id.to_string(),
    module_b.module_type.clone(),
    module_b.immutable,
  );

  module_pot.add_module(module_b.id.clone(), module_b.size, module_b.execution_order);

  module_pot
}
