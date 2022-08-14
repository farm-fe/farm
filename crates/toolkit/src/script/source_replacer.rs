//! replace all `require('./xxx')` and `import(./xxx)`(TODO)'s argument to the actual id. for example
//! ```js
//! const { b } = require('./b');
//! ```
//! will be replaced to
//! ```js
//! const { b } = require("xxx"); // xxx is b's id.
//! ```

use farmfe_core::{
  config::Mode,
  module::{module_graph::ModuleGraph, ModuleId},
  swc_common::{Mark, DUMMY_SP},
  swc_ecma_ast::{CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit, Str},
};
use swc_ecma_visit::VisitMut;

pub struct SourceReplacer<'a> {
  unresolved_mark: Mark,
  module_graph: &'a ModuleGraph,
  module_id: ModuleId,
  mode: Mode,
}

impl<'a> SourceReplacer<'a> {
  pub fn new(
    unresolved_mark: Mark,
    module_graph: &'a ModuleGraph,
    module_id: ModuleId,
    mode: Mode,
  ) -> Self {
    Self {
      unresolved_mark,
      module_graph,
      module_id,
      mode,
    }
  }
}

impl<'a> VisitMut for SourceReplacer<'a> {
  fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
    if call_expr.args.len() != 1 {
      return;
    }

    if let Callee::Expr(box Expr::Ident(Ident { span, sym, .. })) = &call_expr.callee {
      if sym == "require" && span.ctxt.outer() == self.unresolved_mark {
        if let ExprOrSpread {
          spread: None,
          expr: box Expr::Lit(Lit::Str(Str { value, .. })),
        } = &mut call_expr.args[0]
        {
          let source = value.to_string();
          let id = self
            .module_graph
            .get_dep_by_source(&self.module_id, &source);
          // only execute script module
          let dep_module = self.module_graph.module(&id).unwrap();

          if dep_module.module_type.is_script() {
            *value = id.id(self.mode.clone()).into();
          } else {
            // replace with an noop()
            *call_expr = CallExpr {
              span: DUMMY_SP,
              callee: Callee::Expr(Box::new(Expr::Ident(Ident {
                span: DUMMY_SP,
                sym: "noop".into(),
                optional: false,
              }))),
              args: vec![],
              type_args: None,
            }
          }
        }
      }
    }
  }
}
