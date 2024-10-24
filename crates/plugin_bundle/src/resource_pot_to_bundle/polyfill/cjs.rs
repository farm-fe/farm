use farmfe_core::{
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{CallExpr, Callee, Expr, ExprOrSpread},
};

use super::{Polyfill, SimplePolyfill};

pub fn wrap_require_default(expr: Box<Expr>, polyfill: &mut SimplePolyfill) -> Box<Expr> {
  polyfill.add(Polyfill::InteropRequireDefault);
  Box::new(Expr::Call(farmfe_core::swc_ecma_ast::CallExpr {
    span: DUMMY_SP,
    callee: farmfe_core::swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(
      "_interop_require_default".into(),
    ))),
    args: vec![farmfe_core::swc_ecma_ast::ExprOrSpread { spread: None, expr }],
    type_args: None,
    ctxt: SyntaxContext::empty(),
  }))
}

pub fn wrap_require_wildcard(expr: Box<Expr>, polyfill: &mut SimplePolyfill) -> Box<Expr> {
  polyfill.add(Polyfill::Wildcard);
  Box::new(Expr::Call(farmfe_core::swc_ecma_ast::CallExpr {
    span: DUMMY_SP,
    callee: farmfe_core::swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(
      "_interop_require_wildcard".into(),
    ))),
    args: vec![farmfe_core::swc_ecma_ast::ExprOrSpread { spread: None, expr }],
    type_args: None,
    ctxt: SyntaxContext::empty(),
  }))
}

pub fn wrap_export_star(args: Vec<ExprOrSpread>, polyfill: &mut SimplePolyfill) -> Box<Expr> {
  polyfill.add(Polyfill::ExportStar);
  Box::new(Expr::Call(CallExpr {
    span: DUMMY_SP,
    callee: Callee::Expr(Box::new(Expr::Ident("_export_star".into()))),
    args,
    type_args: None,
    ctxt: SyntaxContext::empty(),
  }))
}

pub fn wrap_commonjs(args: Vec<ExprOrSpread>, polyfill: &mut SimplePolyfill) -> Box<Expr> {
  polyfill.add(Polyfill::WrapCommonJs);
  Box::new(Expr::Call(CallExpr {
    span: DUMMY_SP,
    callee: Callee::Expr(Box::new(Expr::Ident(("__commonJs").into()))),
    args,
    type_args: None,
    ctxt: SyntaxContext::empty(),
  }))
}
