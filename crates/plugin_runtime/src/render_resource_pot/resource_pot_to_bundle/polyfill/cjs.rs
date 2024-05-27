use farmfe_core::{swc_common::DUMMY_SP, swc_ecma_ast::Expr};

pub fn wrap_require_default(expr: Box<Expr>) -> Box<Expr> {
  Box::new(Expr::Call(farmfe_core::swc_ecma_ast::CallExpr {
    span: DUMMY_SP,
    callee: farmfe_core::swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(
      "_interop_require_default".into(),
    ))),
    args: vec![farmfe_core::swc_ecma_ast::ExprOrSpread { spread: None, expr }],
    type_args: None,
  }))
}

pub fn wrap_require_wildcard(expr: Box<Expr>) -> Box<Expr> {
  Box::new(Expr::Call(farmfe_core::swc_ecma_ast::CallExpr {
    span: DUMMY_SP,
    callee: farmfe_core::swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(
      "_interop_require_wildcard".into(),
    ))),
    args: vec![farmfe_core::swc_ecma_ast::ExprOrSpread { spread: None, expr }],
    type_args: None,
  }))
}
