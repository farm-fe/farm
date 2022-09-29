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
  module::{module_graph::ModuleGraph, ModuleId, ModuleSystem},
  swc_common::{Mark, DUMMY_SP},
  swc_ecma_ast::{CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit, Str, AwaitExpr, ArrayLit, MemberExpr, MemberProp},
};
use farmfe_toolkit::{swc_ecma_visit::{VisitMut, VisitMutWith}, script::is_commonjs_require};

/// replace all `require('./xxx')` to the actual id and transform require('./xxx') to async. for example:
/// ```js
/// // a.js is originally a commonjs module
/// const { b } = require('./b');
/// // after transform
/// const { b } = await require("xxx"); // xxx is b's id.
/// ```
pub struct SourceReplacer<'a> {
  unresolved_mark: Mark,
  module_graph: &'a ModuleGraph,
  module_id: ModuleId,
  module_system: ModuleSystem,
  mode: Mode,
}

impl<'a> SourceReplacer<'a> {
  pub fn new(
    unresolved_mark: Mark,
    module_graph: &'a ModuleGraph,
    module_id: ModuleId,
    module_system: ModuleSystem,
    mode: Mode,
  ) -> Self {
    Self {
      unresolved_mark,
      module_graph,
      module_id,
      module_system,
      mode,
    }
  }
}

impl<'a> VisitMut for SourceReplacer<'a> {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    if let Expr::Call(call_expr) = expr {
      let is_replaced = self.replace_source_with_id(call_expr);

      if matches!(is_replaced, SourceReplaceResult::Replaced) {
        // if this module is a commonjs module, transform require('xxx') to `await require('xxx')`.
        if matches!(self.module_system, ModuleSystem::CommonJs | ModuleSystem::Hybrid) {
          *expr = Expr::Await(AwaitExpr {
            span: DUMMY_SP,
            arg: Box::new(expr.clone()),
          });
        }
      }
    } else {
      expr.visit_mut_children_with(self);
    }
  }
}

enum SourceReplaceResult {
  NotReplaced,
  Replaced,
  /// the source is not a script module
  NotScriptModule,
}

impl SourceReplacer<'_> {
  fn replace_source_with_id(&mut self, call_expr: &mut CallExpr) -> SourceReplaceResult {
    if call_expr.args.len() == 1 && is_commonjs_require(self.unresolved_mark, &*call_expr) {
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
          call_expr.visit_mut_children_with(self);
          return SourceReplaceResult::Replaced;
        } else {
          // replace require('./index.css') with an noop()
          *call_expr = CallExpr {
            span: DUMMY_SP,
            callee: Callee::Expr(Box::new(Expr::Ident(Ident {
              span: DUMMY_SP,
              sym: "noop".into(),
              optional: false,
            }))),
            args: vec![],
            type_args: None,
          };
          call_expr.visit_mut_children_with(self);
          return SourceReplaceResult::NotScriptModule;
        }
      }
    }

    call_expr.visit_mut_children_with(self);
    SourceReplaceResult::NotReplaced
  }
}