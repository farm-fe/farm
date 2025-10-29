use std::sync::Arc;

use farmfe_core::{
  error::{CompilationError, Result},
  swc_common::{errors::HANDLER, Globals, Mark, SourceMap, SyntaxContext, DUMMY_SP, GLOBALS},
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
  try_with_custom_helper(cm, globals, true, op)
}

pub fn try_with_custom_helper<F>(
  cm: Arc<SourceMap>,
  globals: &Globals,
  external_helper: bool,
  op: F,
) -> Result<()>
where
  F: FnOnce(),
{
  GLOBALS
    .set(globals, || {
      try_with_handler(cm, Default::default(), |handler| {
        HELPERS.set(&Helpers::new(external_helper), || HANDLER.set(handler, op));
        Ok(())
      })
    })
    .map_err(|e| CompilationError::GenericError(e.to_string()))
}

pub struct ResetSyntaxContextVisitMut;

impl VisitMut for ResetSyntaxContextVisitMut {
  fn visit_mut_syntax_context(&mut self, ctxt: &mut farmfe_core::swc_common::SyntaxContext) {
    *ctxt = SyntaxContext::empty();
  }
}

pub fn resolve_module_mark(
  ast: &mut Module,
  is_typescript: bool,
  globals: &Globals,
) -> (Mark, Mark) {
  GLOBALS.set(globals, || call_swc_resolver(ast, is_typescript))
}

fn call_swc_resolver(ast: &mut Module, is_typescript: bool) -> (Mark, Mark) {
  // clear ctxt
  ast.visit_mut_with(&mut ResetSyntaxContextVisitMut);

  let unresolved_mark = Mark::new();
  let top_level_mark = Mark::new();

  ast.visit_mut_with(&mut resolver(
    unresolved_mark,
    top_level_mark,
    is_typescript,
  ));

  (unresolved_mark, top_level_mark)
}

pub struct ResetSpanVisitMut;

impl VisitMut for ResetSpanVisitMut {
  fn visit_mut_span(&mut self, span: &mut farmfe_core::swc_common::Span) {
    *span = DUMMY_SP;
  }
}
