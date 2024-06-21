use farmfe_core::{
  module::{module_graph::ModuleGraph, ModuleId},
  swc_common::{Mark, DUMMY_SP},
  swc_ecma_ast::{CallExpr, Expr, ExprOrSpread, Lit},
};
use farmfe_toolkit::{
  script::is_commonjs_require,
  swc_ecma_visit::{VisitMut, VisitMutWith},
};

use crate::resource_pot_to_bundle::{bundle::ModuleGlobalUniqName, uniq_name::BundleVariable};

enum ReplaceType {
  None,
  Call,
  Ident(usize),
}

impl ReplaceType {
  fn is_replaced(&self) -> bool {
    !matches!(self, ReplaceType::None)
  }
}

///
/// cjs
///
/// ```js
/// // polyfill for module
///
/// // from vite polyfill
///
/// var __getOwnPropNames = Object.getOwnPropertyNames;
///
/// var __commonJS = (cb, mod) => function __require() {
///  return mod || (0, cb[__getOwnPropNames(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
/// };
///
/// __commonJS((exports, module, require) => {});
///
/// ```
///
/// ```js
/// // moduleA.js
/// const moduleA = require('./moduleA');
/// ```
///
pub struct CJSReplace<'a> {
  pub unresolved_mark: Mark,
  pub top_level_mark: Mark,
  pub module_graph: &'a ModuleGraph,
  pub module_id: ModuleId,
  pub module_global_uniq_name: &'a ModuleGlobalUniqName,
  pub bundle_variable: &'a BundleVariable,
}

impl<'a> VisitMut for CJSReplace<'a> {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    let mut replaced: ReplaceType = ReplaceType::None;

    if let Expr::Call(call_expr) = expr {
      if call_expr.args.len() != 1 {
        expr.visit_mut_children_with(self);
        return;
      }

      if is_commonjs_require(self.unresolved_mark, self.top_level_mark, call_expr) {
        if let ExprOrSpread {
          spread: None,
          expr: box Expr::Lit(Lit::Str(str)),
        } = &mut call_expr.args[0]
        {
          let source = str.value.to_string();

          if let Some(id) =
            self
              .module_graph
              .get_dep_by_source_optional(&self.module_id, &source, None)
          {
            if let Some(commonjs_name) = self.module_global_uniq_name.commonjs_name(&id) {
              *call_expr = CallExpr {
                span: DUMMY_SP,
                callee: farmfe_core::swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(
                  self
                    .bundle_variable
                    .render_name(commonjs_name)
                    .as_str()
                    .into(),
                ))),
                args: vec![],
                type_args: None,
              };
              replaced = ReplaceType::Call;
            } else if let Some(ns) = self.module_global_uniq_name.namespace_name(&id) {
              replaced = ReplaceType::Ident(ns);
            }
          }
          // TODO: other bundle | external
        }
      }

      if let ReplaceType::Ident(ns) = &replaced {
        *expr = Expr::Ident(self.bundle_variable.render_name(*ns).as_str().into())
      }
    };

    if !replaced.is_replaced() {
      expr.visit_mut_children_with(self);
    }
  }
}
