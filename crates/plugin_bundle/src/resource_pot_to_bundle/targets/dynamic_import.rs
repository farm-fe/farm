use std::mem;

use farmfe_core::{
  module::{Module, ModuleId},
  plugin::ResolveKind,
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    BindingIdent, BlockStmt, CallExpr, Callee, Expr, ExprOrSpread, FnExpr, Function, Ident, Lit,
    MemberExpr, MemberProp, Param, Pat, ReturnStmt, Stmt,
  },
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::resource_pot_to_bundle::{
  bundle::ModuleAnalyzerManager, common::with_bundle_reference_slot_name,
  modules_analyzer::module_analyzer::ModuleAnalyzer, uniq_name::BundleVariable, ShareBundleContext,
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
pub struct ReplaceDynamicVisit<'a, 'b> {
  module_manager: &'b ModuleAnalyzerManager<'b>,
  module_id: &'a ModuleId,
  bundle_variable: &'a BundleVariable,
  context: &'a ShareBundleContext,
}

enum FromType<'a> {
  Module(&'a Module),
  ModuleAnalyzer(&'a ModuleAnalyzer),
}

impl<'a, 'b> ReplaceDynamicVisit<'a, 'b> {
  pub fn is_matched_dynamic_import(&self, expr: &CallExpr) -> bool {
    matches!(expr.callee, Callee::Import(_))
  }

  fn try_find_module(&self, source: &str) -> Option<FromType> {
    if let Some(m) = self.module_manager.module_graph.get_dep_by_source_optional(
      self.module_id,
      source,
      Some(ResolveKind::DynamicImport),
    ) {
      let v = self
        .module_manager
        .module_analyzer(&m)
        .map(FromType::ModuleAnalyzer);

      if v.is_some() {
        return v;
      }

      let v = self
        .module_manager
        .module_graph
        .module(&m)
        .filter(|m| !m.external)
        .map(FromType::Module);

      return v;
    }

    None
  }

  fn dynamic_import_source_str(&self, expr: &CallExpr) -> Option<FromType> {
    let arg = &expr.args[0];

    if arg.spread.is_some() {
      return None;
    }

    let box Expr::Lit(Lit::Str(str)) = &arg.expr else {
      return None;
    };

    self.try_find_module(str.value.as_ref())
  }

  pub fn replace_expr_by_module_analyzer(
    &self,
    call_expr: &mut CallExpr,
    module_analyzer: &ModuleAnalyzer,
  ) -> Option<Box<Expr>> {
    let arg = &mut call_expr.args[0];

    // same bundle
    if self
      .module_manager
      .is_same_bundle(self.module_id, &module_analyzer.module_id)
    {
      let is_commonjs = module_analyzer.is_commonjs();
      let name = self.bundle_variable.name(if is_commonjs {
        self
          .module_manager
          .module_global_uniq_name
          .commonjs_name(&module_analyzer.module_id)
          .unwrap_or_else(|| {
            panic!(
              "not found module {:?} commonjs name as import() arg",
              module_analyzer.module_id
            )
          })
      } else {
        self
          .module_manager
          .module_global_uniq_name
          .namespace_name(&module_analyzer.module_id)
          .unwrap_or_else(|| {
            panic!(
              "not found module {:?} namespace name as import() arg",
              module_analyzer.module_id
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
          with_bundle_reference_slot_name(
            &module_analyzer.bundle_group_id,
            self.context.options.reference_slot,
          )
          .as_str()
          .into(),
        ))),
      };
    }

    None
  }

  pub fn replace_expr_by_module(
    &self,
    call_expr: &mut CallExpr,
    module: &Module,
  ) -> Option<Box<Expr>> {
    let arg = &mut call_expr.args[0];

    *arg = ExprOrSpread {
      spread: None,
      // TODO: id hash
      expr: Box::new(Expr::Lit(Lit::Str(self.context.format(&module.id).into()))),
    };

    None
  }

  ///
  ///
  /// ```js
  /// // foo.js
  /// export default "foo";
  ///
  /// // index.js (other bundle)
  /// import("./foo").then(res => console.log(res.default));
  /// // =>
  /// import("./foo").then(function(r){ return r.foo_ns }).then(res => console.log(res.default));
  /// ```
  ///
  pub fn map_promise_name_for_other_bundle(
    &self,
    expr: &mut Expr,
    module_analyzer: &ModuleAnalyzer,
  ) {
    let is_commonjs = module_analyzer.is_commonjs();

    let name = if is_commonjs {
      self
        .module_manager
        .module_global_uniq_name
        .commonjs_name(&module_analyzer.module_id)
    } else {
      self
        .module_manager
        .module_global_uniq_name
        .namespace_name(&module_analyzer.module_id)
    };

    if let Some(ns) = name {
      let namespace_name = self.bundle_variable.render_name(ns);
      let e: Expr = mem::replace(expr, Expr::Lit(Lit::Bool(false.into())));

      let return_expr = Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(Expr::Ident("r".into())),
        prop: MemberProp::Ident(namespace_name.as_str().into()),
      });

      *expr = Expr::Call(CallExpr {
        ctxt: SyntaxContext::empty(),
        span: DUMMY_SP,
        callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(e),
          prop: MemberProp::Ident("then".into()),
        }))),
        args: vec![ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Fn(FnExpr {
            ident: None,
            function: Box::new(Function {
              ctxt: SyntaxContext::empty(),
              params: vec![Param {
                span: DUMMY_SP,
                decorators: vec![],
                pat: Pat::Ident(BindingIdent {
                  id: "r".into(),
                  type_ann: None,
                }),
              }],
              decorators: vec![],
              span: DUMMY_SP,
              body: Some(BlockStmt {
                ctxt: SyntaxContext::empty(),
                span: DUMMY_SP,
                stmts: vec![Stmt::Return(ReturnStmt {
                  span: DUMMY_SP,
                  arg: Some(Box::new(if is_commonjs {
                    Expr::Call(CallExpr {
                      ctxt: SyntaxContext::empty(),
                      span: DUMMY_SP,
                      callee: Callee::Expr(Box::new(return_expr)),
                      args: vec![],
                      type_args: None,
                    })
                  } else {
                    return_expr
                  })),
                })],
              }),
              is_generator: false,
              is_async: false,
              type_params: None,
              return_type: None,
            }),
          })),
        }],
        type_args: None,
      });
    }
  }
}

impl<'a, 'b> VisitMut for ReplaceDynamicVisit<'a, 'b> {
  fn visit_mut_expr(&mut self, n: &mut Expr) {
    let mut visited = false;

    if let Expr::Call(ref mut call_expr) = n {
      if self.is_matched_dynamic_import(&call_expr) {
        if let Some(from_type) = self.dynamic_import_source_str(&call_expr) {
          match from_type {
            FromType::Module(module) => {
              self.replace_expr_by_module(call_expr, module);
            }
            FromType::ModuleAnalyzer(module_analyzer) => {
              if let Some(expr) = self.replace_expr_by_module_analyzer(call_expr, module_analyzer) {
                visited = true;
                *n = *expr;
              }

              if !self
                .module_manager
                .is_same_bundle(&self.module_id, &module_analyzer.module_id)
              {
                self.map_promise_name_for_other_bundle(n, module_analyzer);
              }
            }
          }
        }
      }
    }

    if !visited {
      n.visit_mut_children_with(self);
    }
  }
}

pub fn replace_dynamic_import<'a, 'b>(
  module_analyzer_manager: &'b ModuleAnalyzerManager<'b>,
  module_id: &'a ModuleId,
  bundle_variable: &'a BundleVariable,
  options: &'a ShareBundleContext,
) -> ReplaceDynamicVisit<'a, 'b> {
  ReplaceDynamicVisit {
    module_manager: module_analyzer_manager,
    module_id,
    bundle_variable,
    context: options,
  }
}
