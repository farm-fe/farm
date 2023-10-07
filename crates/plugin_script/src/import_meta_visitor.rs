use farmfe_core::{
  swc_common::DUMMY_SP,
  swc_ecma_ast::{Expr, Ident, MemberExpr, MemberProp, MetaPropKind},
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};

/// transform `import.meta.xxx` to `module.meta.xxx`
pub struct ImportMetaVisitor {}

impl VisitMut for ImportMetaVisitor {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    match expr {
      Expr::MetaProp(meta_prop) => {
        if matches!(meta_prop.kind, MetaPropKind::ImportMeta) {
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
