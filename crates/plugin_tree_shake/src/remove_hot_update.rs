pub struct UselessHotUpdateStmtRemover;

impl VisitMut for UselessHotUpdateStmtRemover {
  fn visit_mut_stmt(&mut self, stmt: &mut swc_ecma_ast::Stmt) {
    // 判断该语句是否包含成员表达式的调用
    match stmt {
      Stmt::Expr(expr_stmt) => {
        if is_import_meta_hot_call(&expr_stmt.expr) {
          *stmt = Stmt::Empty(EmptyStmt { span: DUMMY_SP })
        }
      }
      _ => (),
    }
    stmt.visit_mut_children_with(self)
  }
}

fn is_import_meta_hot_call(expr: &Expr) -> bool {
  if let Expr::Call(CallExpr {
    callee: Callee::Expr(expr_callee),
    ..
  }) = expr
  {
    if let Expr::Member(expr_member) = &**expr_callee {
      if let MemberExpr {
        obj:
          box Expr::Member(MemberExpr {
            obj: box Expr::MetaProp(MetaPropExpr { kind, .. }),
            prop: MemberProp::Ident(Ident { sym: hot_sym, .. }),
            ..
          }),
        ..
      } = expr_member
      {
        return *kind == MetaPropKind::ImportMeta && hot_sym == "hot";
      }
    }
  }
  false
}
