use farmfe_core::{
  module::ModuleId,
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit, MemberExpr, MemberProp},
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::resource_pot_to_bundle::{
  bundle::ModuleAnalyzerManager, common::with_bundle_reference_slot_name, uniq_name::BundleVariable,
};

///
/// ```ts
/// // same bundle
/// import("./dynamic")
/// // =>
/// Promise.resolve(dynamic_ns)
///
///
/// import("./dynamic").then((ns) => ns.default)
/// // =>
/// Promise.resolve(dynamic_ns).then(ns => ns.default)
///
/// const foo = () => import("./dynamic")
/// // =>
/// const foo = () => Promise.resolve(dynamic_ns)
///
/// // other bundle
/// import("./dynamic")
/// // =>
/// import("./bundle-1")
/// ```
///
///
pub struct ReplaceDynamicVisit<'a> {
  module_manager: &'a ModuleAnalyzerManager<'a>,
  module_id: &'a ModuleId,
  bundle_variable: &'a BundleVariable,
}

impl<'a> ReplaceDynamicVisit<'a> {
  pub fn replace_expr(&self, call_expr: &mut CallExpr) -> Option<Box<Expr>> {
    if !matches!(call_expr.callee, Callee::Import(_)) {
      return None;
    }

    let arg = &mut call_expr.args[0];

    if arg.spread.is_some() {
      return None;
    }

    let box Expr::Lit(Lit::Str(str)) = &arg.expr else {
      return None;
    };

    let Some(value) = self
      .module_manager
      .module_analyzer_by_source(self.module_id, str.value.as_ref())
    else {
      // external
      return None;
    };

    // same bundle
    if self
      .module_manager
      .is_same_bundle(self.module_id, &value.module_id)
    {
      let is_commonjs = value.is_commonjs();
      let name = self.bundle_variable.name(if is_commonjs {
        self
          .module_manager
          .module_global_uniq_name
          .commonjs_name(&value.module_id)
          .unwrap_or_else(|| {
            panic!(
              "not found module {:?} commonjs name as import() arg",
              value.module_id
            )
          })
      } else {
        self
          .module_manager
          .module_global_uniq_name
          .namespace_name(&value.module_id)
          .unwrap_or_else(|| {
            panic!(
              "not found module {:?} namespace name as import() arg",
              value.module_id
            )
          })
      });

      let expr = Box::new(Expr::Ident(Ident::from(name.as_str())));
      return Some(Box::new(Expr::Call(CallExpr {
        span: DUMMY_SP,
        callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(Expr::Ident("Promise".into())),
          prop: MemberProp::Ident("resolve".into()),
        }))),
        args: vec![ExprOrSpread {
          spread: None,
          expr: if is_commonjs {
            Box::new(Expr::Call(CallExpr {
              span: DUMMY_SP,
              callee: Callee::Expr(expr),
              args: vec![],
              type_args: None,
              ctxt: SyntaxContext::empty(),
            }))
          } else {
            expr
          },
        }],
        type_args: None,
        ctxt: SyntaxContext::empty(),
      })));
    } else {
      // other bundle
      *arg = ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Lit(Lit::Str(
          with_bundle_reference_slot_name(&value.resource_pot_id)
            .as_str()
            .into(),
        ))),
      };
    }

    None
  }
}

impl<'a> VisitMut for ReplaceDynamicVisit<'a> {
  fn visit_mut_expr(&mut self, n: &mut Expr) {
    let mut visited = false;
    if let Expr::Call(ref mut call_expr) = n {
      if let Some(expr) = self.replace_expr(call_expr) {
        visited = true;
        *n = *expr;
      }
    }

    if !visited {
      n.visit_mut_children_with(self);
    }
  }
}

pub fn replace_dynamic_import<'a>(
  module_manager: &'a ModuleAnalyzerManager<'a>,
  module_id: &'a ModuleId,
  bundle_variable: &'a BundleVariable,
) -> ReplaceDynamicVisit<'a> {
  ReplaceDynamicVisit {
    module_manager,
    module_id,
    bundle_variable,
  }
}
