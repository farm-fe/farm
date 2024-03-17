use farmfe_core::{
  swc_common::DUMMY_SP,
  swc_ecma_ast::{
    self, CallExpr, Callee, EmptyStmt, Expr, Ident, IfStmt, MemberExpr, MemberProp, MetaPropExpr,
    MetaPropKind, Module, Stmt,
  },
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn remove_useless_hot_update_stmts(module: &mut Module) -> &mut Module {
  let mut remover = UselessHotUpdateStmtRemover;
  module.visit_mut_with(&mut remover);
  module
}

pub struct UselessHotUpdateStmtRemover;

impl VisitMut for UselessHotUpdateStmtRemover {
  // detects whether an if statement contains module.meta.hot
  fn visit_mut_if_stmt(&mut self, if_stmt: &mut swc_ecma_ast::IfStmt) {
    println!("if_stmt: {:?}", if_stmt);
    if let Expr::Member(MemberExpr {
      obj:
        box Expr::Member(MemberExpr {
          obj: box Expr::Ident(Ident { sym: module, .. }),
          prop: MemberProp::Ident(Ident { sym: meta, .. }),
          ..
        }),
      prop: MemberProp::Ident(Ident { sym: hot, .. }),
      ..
    }) = &*if_stmt.test
    {
      if module.to_string() == "module" && meta.to_string() == "meta" && hot.to_string() == "hot" {
        // set test tuning to false
        *if_stmt.test = Expr::Ident(Ident {
          sym: "false".into(),
          span: DUMMY_SP,
          optional: false,
        });
      }
    }
  }
}
