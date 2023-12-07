use farmfe_core::{
  swc_common::DUMMY_SP,
  swc_ecma_ast::{CallExpr, Callee, Expr, Ident, MemberExpr, MemberProp, MetaPropKind},
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};

/// transform `import.meta.xxx` to `module.meta.xxx`
pub struct ImportMetaVisitor {}

impl ImportMetaVisitor {
  pub fn new() -> Self {
    Self {}
  }
}

impl VisitMut for ImportMetaVisitor {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    match expr {
      Expr::MetaProp(meta_prop) => {
        if matches!(meta_prop.kind, MetaPropKind::ImportMeta) {
          // check if it's hmr accepted
          *expr = Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(Ident::new("module".into(), DUMMY_SP))),
            prop: MemberProp::Ident(Ident::new("meta".into(), DUMMY_SP)),
          });
        }
      }
      _ => {
        expr.visit_mut_children_with(self);
      }
    }
  }
}

pub struct HmrAcceptedVisitor {
  pub is_hmr_accepted: bool,
}

impl HmrAcceptedVisitor {
  pub fn new() -> Self {
    Self {
      is_hmr_accepted: false,
    }
  }
}

impl VisitMut for HmrAcceptedVisitor {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    // detect hmr based on `module.meta.hot.accept`
    if let Expr::Call(CallExpr {
      callee:
        Callee::Expr(box Expr::Member(MemberExpr {
          obj:
            box Expr::Member(MemberExpr {
              obj:
                box Expr::Member(MemberExpr {
                  obj: box Expr::Ident(Ident { sym: module, .. }),
                  prop: MemberProp::Ident(Ident { sym: meta, .. }),
                  ..
                }),
              prop: MemberProp::Ident(Ident { sym: hot, .. }),
              ..
            }),
          prop: MemberProp::Ident(Ident { sym: accept, .. }),
          ..
        })),
      ..
    }) = expr
    {
      if &module.to_string() == "module"
        && &meta.to_string() == "meta"
        && &hot.to_string() == "hot"
        && &accept.to_string() == "accept"
      {
        self.is_hmr_accepted = true;
      }
    } else {
      expr.visit_mut_children_with(self);
    }
  }
}
