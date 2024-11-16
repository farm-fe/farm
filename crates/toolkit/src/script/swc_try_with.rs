use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  swc_common::{errors::HANDLER, Globals, Mark, SourceMap, SyntaxContext, GLOBALS},
  swc_ecma_ast::Module,
};
use swc_ecma_transforms::helpers::{Helpers, HELPERS};
use swc_ecma_transforms_base::resolver;
use swc_ecma_visit::{VisitMut, VisitMutWith};
use swc_error_reporters::handler::try_with_handler;

pub fn try_with<F>(cm: Arc<SourceMap>, globals: &Globals, op: F) -> Result<()>
where
  F: FnOnce(),
{
  GLOBALS
    .set(globals, || {
      try_with_handler(cm, Default::default(), |handler| {
        HELPERS.set(&Helpers::new(true), || HANDLER.set(handler, op));
        Ok(())
      })
    })
    .map_err(|e| CompilationError::GenericError(e.to_string()))
}

pub struct ResetSpanVisitMut;

impl VisitMut for ResetSpanVisitMut {
  fn visit_mut_syntax_context(&mut self, ctxt: &mut farmfe_core::swc_common::SyntaxContext) {
    *ctxt = SyntaxContext::empty();
  }
}

pub fn resolve_module_mark(
  ast: &mut Module,
  is_typescript: bool,
  context: &Arc<CompilationContext>,
) -> (Mark, Mark) {
  GLOBALS.set(&context.meta.script.globals, || {
    // clear ctxt
    ast.visit_mut_with(&mut ResetSpanVisitMut);

    let unresolved_mark = Mark::new();
    let top_level_mark = Mark::new();

    ast.visit_mut_with(&mut resolver(
      unresolved_mark,
      top_level_mark,
      is_typescript,
    ));

    (unresolved_mark, top_level_mark)
  })
}
