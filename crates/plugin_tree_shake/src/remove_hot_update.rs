use farmfe_core::{
  module::module_graph::ModuleGraph,
  swc_common::DUMMY_SP,
  swc_ecma_ast::{EmptyStmt, Expr, Ident, MemberExpr, MemberProp, Module, Stmt},
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn remove_useless_hot_update_stmts(module_graph: &mut ModuleGraph) {
  let mut remover = UselessHotUpdateStmtRemover;

  module_graph.modules_mut().iter_mut().for_each(|module| {
    if !module.module_type.is_script() || module.external {
      return;
    }
    let ast = &mut module.meta.as_script_mut().take_ast();
    ast.visit_mut_with(&mut remover);
    module.meta.as_script_mut().set_ast(ast.clone());
  });
}

pub struct UselessHotUpdateStmtRemover;

impl VisitMut for UselessHotUpdateStmtRemover {
  // detects whether an if statement contains module.meta.hot
  fn visit_mut_stmt(&mut self, stmt: &mut Stmt) {
    match stmt {
      Stmt::If(if_stmt) => {
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
          if module.to_string() == "module"
            && meta.to_string() == "meta"
            && hot.to_string() == "hot"
          {
            // set test tuning to false
            *stmt = Stmt::Empty(EmptyStmt { span: DUMMY_SP })
          }
        }
      }
      _ => (),
    }
    stmt.visit_mut_children_with(self);
  }
}
