use std::collections::HashMap;

use farmfe_core::{
  module::{module_graph::ModuleGraph, ModuleId},
  swc_common::{Mark, DUMMY_SP},
  swc_ecma_ast::{
    self, BindingIdent, CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit, MemberExpr,
    Module as EcmaAstModule, ModuleItem, Pat, Stmt, VarDecl, VarDeclarator,
  },
};
use farmfe_toolkit::{
  script::is_commonjs_require,
  swc_ecma_utils::ExprFactory,
  swc_ecma_visit::{Visit, VisitMut, VisitMutWith, VisitWith},
};

use crate::resource_pot_to_bundle::{
  bundle::ModuleGlobalUniqName,
  bundle_external::{ExternalReferenceExport, ReferenceKind},
  modules_analyzer::module_analyzer::ModuleAnalyzer,
  uniq_name::BundleVariable,
};

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

struct CJSReplace<'a> {
  unresolved_mark: Mark,
  top_level_mark: Mark,
  module_graph: &'a ModuleGraph,
  module_id: ModuleId,
  module_global_uniq_name: &'a ModuleGlobalUniqName,
  bundle_variable: &'a BundleVariable,
}

impl<'a> VisitMut for CJSReplace<'a> {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    match expr {
      Expr::Call(call_expr) => {
        if call_expr.args.len() != 1 {
          call_expr.visit_mut_children_with(self);
          return;
        }

        if let Callee::Expr(box Expr::Ident(Ident { sym, .. })) = &call_expr.callee {
          if sym == "require" {
            // TODO: replace require('./moduleA') to moduleA()

            if let ExprOrSpread {
              spread: None,
              expr: box Expr::Lit(Lit::Str(str)),
            } = &mut call_expr.args[0]
            {
              let source = str.value.to_string();
              let id = self
                .module_graph
                .get_dep_by_source(&self.module_id, &source, None);

              // TODO: other bundle | external
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
              } else if let Some(ns) = self.module_global_uniq_name.namespace_name(&id) {
                *expr = Expr::Ident(self.bundle_variable.render_name(ns).as_str().into())
              }
            }
          } else {
            call_expr.visit_mut_children_with(self);
          }
        } else {
          call_expr.visit_mut_children_with(self);
        }
      }
      _ => {
        expr.visit_mut_children_with(self);
      }
    }
  }
}

///
///
/// ```js
/// require("./moduleA");
/// ```
///
pub struct CjsCollector<'a> {
  pub unresolved_mark: Mark,
  pub top_level_mark: Mark,
  pub module_graph: &'a ModuleGraph,
  pub module_id: ModuleId,
  pub deps: Vec<ModuleId>,
}

impl<'a> Visit for CjsCollector<'a> {
  fn visit_call_expr(&mut self, n: &CallExpr) {
    if n.args.len() != 1 {
      n.visit_children_with(self);
      return;
    }

    if is_commonjs_require(self.unresolved_mark, self.top_level_mark, n) {
      if let ExprOrSpread {
        spread: None,
        expr: box Expr::Lit(Lit::Str(str)),
      } = &n.args[0]
      {
        let source = str.value.to_string();
        let id = self
          .module_graph
          .get_dep_by_source(&self.module_id, &source, None);
        self.deps.push(id);
      }
    }
  }
}

#[derive(Default)]
pub struct CjsModuleAnalyzer {
  pub require_modules: Vec<ModuleId>,
  pub commonjs_export: HashMap<ReferenceKind, ExternalReferenceExport>,
}

impl CjsModuleAnalyzer {
  pub fn new() -> Self {
    Self {
      require_modules: vec![],
      commonjs_export: Default::default(),
    }
  }

  pub fn analyze_modules(
    &mut self,
    module_analyzer: &mut ModuleAnalyzer,
    module_graph: &ModuleGraph,
  ) {
    let mut collector = CjsCollector {
      unresolved_mark: module_analyzer.mark.0,
      top_level_mark: module_analyzer.mark.1,
      module_graph,
      module_id: module_analyzer.module_id.clone(),
      deps: vec![],
    };

    module_analyzer.ast.visit_with(&mut collector);

    println!(
      "\n\nanalyze_modules: {:?}\ndeps: {:#?}",
      module_analyzer.module_id.to_string(),
      collector.deps
    );
  }

  pub fn replace_require_require(
    &mut self,
    mark: (Mark, Mark),
    ast: &mut EcmaAstModule,
    module_id: &ModuleId,
    module_graph: &ModuleGraph,
    module_global_uniq_name: &ModuleGlobalUniqName,
    bundle_variable: &BundleVariable,
  ) {
    let mut replacer: CJSReplace = CJSReplace {
      unresolved_mark: mark.0,
      top_level_mark: mark.1,
      module_graph,
      module_id: module_id.clone(),
      module_global_uniq_name,
      bundle_variable,
    };

    ast.visit_mut_with(&mut replacer);
  }

  pub fn build_commonjs_export(
    &self,
    module_id: &ModuleId,
    bundle_variable: &BundleVariable,
    module_analyzer: &ModuleAnalyzer,
    module_global_uniq_name: &ModuleGlobalUniqName,
  ) -> Vec<ModuleItem> {
    let mut result = vec![];
    if let Some(reference_export) = module_analyzer
      .cjs_module_analyzer
      .commonjs_export
      .get(&module_id.clone().into())
    {
      let cjs_name =
        bundle_variable.render_name(module_global_uniq_name.commonjs_name(&module_id).unwrap());

      let mut decls = vec![];

      let cjs_caller = CallExpr {
        span: DUMMY_SP,
        callee: swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(cjs_name.as_str().into()))),
        args: vec![],
        type_args: None,
      };

      if let Some(default) = reference_export.default {
        decls.push(VarDeclarator {
          span: DUMMY_SP,
          name: Pat::Ident(BindingIdent {
            id: Ident::from(bundle_variable.render_name(default).as_str()),
            type_ann: None,
          }),
          init: Some(Box::new(Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Call(cjs_caller.clone())),
            prop: swc_ecma_ast::MemberProp::Ident("default".into()),
          }))),
          definite: false,
        });
      }

      if let Some(ns) = reference_export.namespace {
        decls.push(VarDeclarator {
          span: DUMMY_SP,
          name: Pat::Ident(BindingIdent {
            id: Ident::from(bundle_variable.render_name(ns).as_str()),
            type_ann: None,
          }),
          init: Some(Box::new(Expr::Call(cjs_caller.clone()))),
          definite: false,
        });
      }

      let ordered_keys = reference_export.named.keys().collect::<Vec<_>>();
      for imported in ordered_keys {
        let named = &reference_export.named[imported];

        println!("named: {:#?}", bundle_variable.var_by_index(*named));
        println!("imported: {:#?}", bundle_variable.var_by_index(*imported));

        let imported = bundle_variable.name(*imported);


        decls.push(VarDeclarator {
          span: DUMMY_SP,
          name: Pat::Ident(BindingIdent {
            id: Ident::from(bundle_variable.name(*named).as_str()),
            type_ann: None,
          }),
          init: Some(Box::new(Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Call(cjs_caller.clone())),
            prop: swc_ecma_ast::MemberProp::Ident(imported.as_str().into()),
          }))),
          definite: false,
        });
      }

      result.push(ModuleItem::Stmt(Stmt::Decl(swc_ecma_ast::Decl::Var(
        Box::new(VarDecl {
          span: DUMMY_SP,
          kind: swc_ecma_ast::VarDeclKind::Var,
          declare: false,
          decls,
        }),
      ))));
    }

    result
  }
}
