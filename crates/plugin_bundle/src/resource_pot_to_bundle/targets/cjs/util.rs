use farmfe_core::{
  config::{external::ExternalConfig, Config, ModuleFormat},
  module::{module_graph::ModuleGraph, ModuleId},
  swc_common::{Mark, DUMMY_SP},
  swc_ecma_ast::{CallExpr, Callee, Expr, ExprOrSpread, Lit, MemberExpr, MemberProp},
};
use farmfe_toolkit::{
  script::is_commonjs_require,
  swc_ecma_visit::{VisitMut, VisitMutWith},
};

use crate::resource_pot_to_bundle::{
  bundle::ModuleGlobalUniqName, uniq_name::BundleVariable, Polyfill, SimplePolyfill,
};

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
  pub config: &'a Config,
  pub polyfill: &'a mut SimplePolyfill,
  pub external_config: &'a ExternalConfig,
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
            if self.module_graph.module(&id).is_some_and(|m| m.external) {
              if self.config.output.target_env.is_library()
                && self.config.output.target_env.is_node()
              {
                // node esm
                if matches!(self.config.output.format, ModuleFormat::EsModule) {
                  self.polyfill.add(Polyfill::NodeEsmGlobalRequireHelper);
                  call_expr.callee = Callee::Expr(Box::new(Expr::Member(MemberExpr {
                    span: DUMMY_SP,
                    obj: Box::new(Expr::Ident("global".into())),
                    prop: MemberProp::Ident("nodeRequire".into()),
                  })));
                }
              } else {
                // browser
                self.polyfill.add(Polyfill::BrowserExternalRequire);

                let replace_source = self
                  .external_config
                  .find_match(&source)
                  .map(|v| v.source(&source))
                  // it's maybe from plugin
                  .unwrap_or(source.clone());

                call_expr.callee =
                  Callee::Expr(Box::new(Expr::Ident("loadExternalRequire".into())));
                call_expr.args = vec![ExprOrSpread {
                  spread: None,
                  expr: Box::new(Expr::Lit(Lit::Str(replace_source.into()))),
                }];
                call_expr.span = DUMMY_SP;
              }
            } else if let Some(commonjs_name) = self.module_global_uniq_name.commonjs_name(&id) {
              *call_expr = CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(
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
          // TODO: other bundle
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
