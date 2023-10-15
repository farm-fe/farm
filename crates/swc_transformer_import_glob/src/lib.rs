//! Transform `import.meta.glob` like Vite. See https://vitejs.dev/guide/features.html#glob-import
//! for example:
//! ```js
//! const modules = import.meta.glob('./dir/*.js', { eager: true })
//! ```
//! will be transformed to:
//! ```js
//! const modules = {
//!   './dir/foo.js': 'export default "foo"\n',
//!   './dir/bar.js': 'export default "bar"\n',
//! }
//! ````
#![feature(box_patterns)]

use swc_common::DUMMY_SP;
use swc_ecma_ast::{Expr, Ident, MemberExpr, MemberProp, MetaPropExpr, MetaPropKind};

use swc_ecma_visit::{VisitMut, VisitMutWith};

pub struct ImportGlobVisitor {}

impl ImportGlobVisitor {
  pub fn new() -> Self {
    Self {}
  }
}

impl VisitMut for ImportGlobVisitor {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    match expr {
      Expr::Member(MemberExpr {
        obj:
          box Expr::MetaProp(MetaPropExpr {
            kind: MetaPropKind::ImportMeta,
            ..
          }),
        prop: MemberProp::Ident(Ident { sym: glob, .. }),
        ..
      }) => {}

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
