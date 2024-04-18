use std::{collections::HashSet, sync::Arc};

use farmfe_core::{context::CompilationContext, module::ModuleId};
use farmfe_toolkit::swc_ecma_utils::contains_top_level_await;

pub fn find_async_modules(context: &Arc<CompilationContext>) -> HashSet<ModuleId> {
  let module_graph = context.module_graph.read();
  let (mut topo_sorted_modules, _) = module_graph.toposort();
  topo_sorted_modules.reverse();

  let mut async_modules = HashSet::new();

  for module_id in topo_sorted_modules {
    let module = module_graph.module(&module_id).unwrap();
    if module.module_type.is_script() {
      let ast = &module.meta.as_script().ast;

      if contains_top_level_await(ast) {
        async_modules.insert(module_id);
      } else {
        // if any dependency is async module, then mark the module as async module
        let mut found = false;

        for (dep, _) in module_graph.dependencies(&module_id) {
          if async_modules.contains(&dep) {
            found = true;
            break;
          }
        }

        if found {
          async_modules.insert(module_id);
        }
      }
    }
  }

  async_modules
}
