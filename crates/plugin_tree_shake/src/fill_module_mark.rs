use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::module_graph::ModuleGraph,
  rayon::iter::{IntoParallelIterator, ParallelIterator},
};
use farmfe_toolkit::script::swc_try_with::resolve_module_mark;

pub fn fill_module_mark(module_graph: &mut ModuleGraph, context: &Arc<CompilationContext>) {
  module_graph
    .modules_mut()
    .into_par_iter()
    .filter(|m| m.module_type.is_script() && !m.external)
    .for_each(|module| {
      let meta = module.meta.as_script_mut();

      if meta.top_level_mark != 0 || meta.unresolved_mark != 0 {
        return;
      }

      let ast = &mut meta.ast;

      let (unresolved_mark, top_level_mark) =
        resolve_module_mark(ast, module.module_type.is_typescript(), context);

      module.meta.as_script_mut().unresolved_mark = unresolved_mark.as_u32();
      module.meta.as_script_mut().top_level_mark = top_level_mark.as_u32();
    });
}
