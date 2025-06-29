use farmfe_core::{
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{Expr, ExprOrSpread, Lit, MemberExpr, MemberProp, NewExpr},
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};

///
/// transform when cjs and library
/// ```ts
/// import.meta.url
/// // =>
/// new URL(__filename, 'file:').href
/// ```
///
struct ImportMetaURLVisitor {}

impl ImportMetaURLVisitor {
  fn replace_import_meta_url(&self, n: &mut Expr) -> bool {
    if let Expr::Member(member) = n {
      if let box Expr::MetaProp(_) = member.obj {
        if let MemberProp::Ident(ident) = &member.prop {
          if ident.sym == "url" {
            *n = Expr::Member(MemberExpr {
              span: DUMMY_SP,
              obj: Box::new(Expr::New(NewExpr {
                span: DUMMY_SP,
                callee: Box::new(Expr::Ident("URL".into())),
                args: Some(vec![
                  ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Ident("__filename".into())),
                  },
                  ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Lit(Lit::Str("file:".into()))),
                  },
                ]),
                type_args: None,
                ctxt: SyntaxContext::empty(),
              })),
              prop: MemberProp::Ident("href".into()),
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

pub fn replace_import_meta_url(ast: &mut farmfe_core::swc_ecma_ast::Module) {
  ast.visit_mut_with(&mut ImportMetaURLVisitor {});
}
