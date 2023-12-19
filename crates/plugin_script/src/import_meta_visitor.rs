use std::collections::HashSet;

use farmfe_core::{
  module::ModuleId,
  swc_common::DUMMY_SP,
  swc_ecma_ast::{
    CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit, MemberExpr, MemberProp, MetaPropKind,
  },
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
  pub is_hmr_self_accepted: bool,
  pub hmr_accepted_deps: HashSet<ModuleId>,
}

impl HmrAcceptedVisitor {
  pub fn new() -> Self {
    Self {
      is_hmr_self_accepted: false,
      hmr_accepted_deps: HashSet::new(),
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
      args,
      ..
    }) = expr
    {
      if &module.to_string() == "module"
        && &meta.to_string() == "meta"
        && &hot.to_string() == "hot"
        && &accept.to_string() == "accept"
      {
        // if args is empty or the first arg is a function expression, then it's hmr self accepted
        if args.is_empty()
          || matches!(args[0], ExprOrSpread {
            expr: box Expr::Fn(..) | box Expr::Arrow(..), ..
          })
        {
          self.is_hmr_self_accepted = true;
        } else if !args.is_empty() {
          // if args is not empty and the first arg is a literal, then it's hmr accepted deps
          if let ExprOrSpread {
            expr: box Expr::Lit(Lit::Str(s)),
            ..
          } = &args[0]
          {
            // string literal
            self.hmr_accepted_deps.push(s.value.to_string());
          } else if let ExprOrSpread {
            expr: box Expr::Array(arr),
            ..
          } = &args[0]
          {
            // array literal
            for expr in arr.elems.iter() {
              if let Some(ExprOrSpread {
                expr: box Expr::Lit(Lit::Str(s)),
                ..
              }) = expr
              {
                self.hmr_accepted_deps.push(s.value.to_string());
              }
            }
          }
        }
      }
    } else {
      expr.visit_mut_children_with(self);
    }
  }
}
