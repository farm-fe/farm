use farmfe_core::{
  module::{module_graph::ModuleGraph, ModuleId},
  parking_lot::Mutex,
  rayon::iter::{IntoParallelIterator, ParallelIterator},
  swc_common::GLOBALS,
  HashMap,
};

use crate::module::TreeShakeModule;

pub fn init_tree_shake_module_map(
  module_graph: &mut ModuleGraph,
  context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
) -> HashMap<ModuleId, TreeShakeModule> {
  let tree_shake_modules_map =
    Mutex::new(HashMap::<ModuleId, TreeShakeModule>::default());
  module_graph
    .modules_mut()
    .into_par_iter()
    .for_each(|module| {
      if !module.module_type.is_script() || module.external {
        return;
      }

      GLOBALS.set(&context.meta.script.globals, || {
        let tree_shake_module = TreeShakeModule::new(module);
        tree_shake_modules_map
          .lock()
          .insert(module.id.clone(), tree_shake_module);
      });
    });
  tree_shake_modules_map.into_inner()
}
