use farmfe_core::{
  swc_common::{Mark, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    CallExpr, Callee, Expr, ExprOrSpread, Ident, IdentName, Lit, MemberExpr, MemberProp,
  },
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};

///
/// transform when cjs and library
/// ```ts
/// import.meta.url
/// // =>
/// `require('node:url').pathToFileURL(__filename).href`
/// ```
///
struct ImportMetaURLVisitor {
  unresolved_mark: Mark,
}

impl ImportMetaURLVisitor {
  fn replace_import_meta_url(&self, n: &mut Expr) -> bool {
    if let Expr::Member(member) = n {
      if let box Expr::MetaProp(_) = member.obj {
        if let MemberProp::Ident(ident) = &member.prop {
          if ident.sym == "url" {
            // `require('node:url').pathToFileURL(__filename).href`
            *n = Expr::Member(MemberExpr {
              span: DUMMY_SP,
              obj: Box::new(Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
                  span: DUMMY_SP,
                  obj: Box::new(Expr::Call(CallExpr {
                    span: DUMMY_SP,
                    ctxt: SyntaxContext::empty(),
                    callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                      "require".into(),
                      DUMMY_SP,
                      SyntaxContext::empty().apply_mark(self.unresolved_mark),
                    )))),
                    args: vec![ExprOrSpread {
                      spread: None,
                      expr: Box::new(Expr::Lit(Lit::Str("node:url".into()))),
                    }],
                    type_args: None,
                  })),
                  prop: MemberProp::Ident(IdentName::new("pathToFileURL".into(), DUMMY_SP)),
                }))),
                args: vec![ExprOrSpread {
                  spread: None,
                  expr: Box::new(Expr::Ident(Ident::new(
                    "__filename".into(),
                    DUMMY_SP,
                    SyntaxContext::empty().apply_mark(self.unresolved_mark),
                  ))),
                }],
                type_args: None,
                ctxt: SyntaxContext::empty(),
              })),
              prop: MemberProp::Ident(IdentName::new("href".into(), DUMMY_SP)),
            });

            return true;
          }
        }
      }
    }

    false
  }
}

impl VisitMut for ImportMetaURLVisitor {
  fn visit_mut_expr(&mut self, n: &mut Expr) {
    if !self.replace_import_meta_url(n) {
      n.visit_mut_children_with(self);
    }
  }
}

pub fn replace_import_meta_url(ast: &mut farmfe_core::swc_ecma_ast::Module, unresolved_mark: Mark) {
  ast.visit_mut_with(&mut ImportMetaURLVisitor { unresolved_mark });
}
