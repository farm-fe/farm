use farmfe_core::{
  module::module_graph::ModuleGraph,
  swc_common::DUMMY_SP,
  swc_ecma_ast::{EmptyStmt, Expr, Ident, MemberExpr, MemberProp, Stmt},
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn remove_useless_hot_update_stmts(module_graph: &mut ModuleGraph) {
  let mut remover = UselessHotUpdateStmtRemover;

  module_graph.modules_mut().iter_mut().for_each(|module| {
    if !module.module_type.is_script() || module.external {
      return;
    }
    let script_meta_data = module.meta.as_script_mut();
    let ast = &mut script_meta_data.take_ast();
    ast.visit_mut_with(&mut remover);
    script_meta_data.set_ast(ast.clone());
  });
}

pub struct UselessHotUpdateStmtRemover;

impl VisitMut for UselessHotUpdateStmtRemover {
  // detects whether an if statement contains module.meta.hot
  fn visit_mut_stmt(&mut self, stmt: &mut Stmt) {
    match stmt {
      Stmt::If(if_stmt) => {
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
          // use `&` to circumvent additional heap allocations
          if &module.to_string() == "module"
            && &meta.to_string() == "meta"
            && &hot.to_string() == "hot"
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
