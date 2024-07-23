use std::collections::HashMap;

use farmfe_core::{
  farm_profile_function,
  module::{module_graph::ModuleGraph, ModuleId},
  swc_common::{Mark, DUMMY_SP},
  swc_ecma_ast::{
    self, BindingIdent, CallExpr, ComputedPropName, Expr, ExprOrSpread, ExprStmt, Ident, Lit,
    MemberExpr, MemberProp, Module as EcmaAstModule, ModuleItem, Pat, Stmt, VarDecl, VarDeclarator,
  },
};
use farmfe_toolkit::{
  script::is_commonjs_require,
  swc_ecma_visit::{Visit, VisitWith},
};

use crate::resource_pot_to_bundle::{
  bundle::{
    bundle_external::{ExternalReferenceExport, ExternalReferenceImport, ReferenceKind},
    ModuleGlobalUniqName,
  },
  polyfill::{
    cjs::{self, wrap_require_default, wrap_require_wildcard},
    SimplePolyfill,
  },
  uniq_name::BundleVariable,
};

pub mod generate;
pub mod patch;
mod util;
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
  fn visit_expr(&mut self, n: &Expr) {
    let mut is_collect = false;
    if let Expr::Call(call_expr) = n {
      if call_expr.args.len() != 1 {
        return;
      }

      if is_commonjs_require(self.unresolved_mark, self.top_level_mark, call_expr) {
        is_collect = true;
        if let ExprOrSpread {
          spread: None,
          expr: box Expr::Lit(Lit::Str(str)),
        } = &call_expr.args[0]
        {
          let source = str.value.to_string();
          let id = self
            .module_graph
            .get_dep_by_source(&self.module_id, &source, None);
          self.deps.push(id);
        }
      }
    };

    if !is_collect {
      n.visit_children_with(self);
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
    &self,
    module_id: &ModuleId,
    unresolved_mark: Mark,
    top_level_mark: Mark,
    ast: &EcmaAstModule,
    module_graph: &ModuleGraph,
  ) -> Vec<ModuleId> {
    farm_profile_function!("cjs module analyzer:analyzer modules");
    let mut collector = CjsCollector {
      unresolved_mark,
      top_level_mark,
      module_graph,
      module_id: module_id.clone(),
      deps: vec![],
    };

    ast.visit_with(&mut collector);

    collector.deps
  }

  /** when use esm export commonjs module */
  pub fn redeclare_commonjs_export(
    module_id: &ModuleId,
    bundle_variable: &BundleVariable,
    module_global_uniq_name: &ModuleGlobalUniqName,
    reference_import: &ExternalReferenceImport,
    polyfill: &mut SimplePolyfill,
  ) -> Vec<ModuleItem> {
    let mut result = vec![];

    let cjs_name =
      bundle_variable.render_name(module_global_uniq_name.commonjs_name(module_id).unwrap());

    let mut decls = vec![];

    let cjs_caller = CallExpr {
      span: DUMMY_SP,
      callee: swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(cjs_name.as_str().into()))),
      args: vec![],
      type_args: None,
    };

    if reference_import.is_empty() {
      result.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(Expr::Call(cjs_caller)),
      })));
      return result;
    }

    if let Some(default) = reference_import.default {
      decls.push(VarDeclarator {
        span: DUMMY_SP,
        name: Pat::Ident(BindingIdent {
          id: Ident::from(bundle_variable.render_name(default).as_str()),
          type_ann: None,
        }),
        init: Some(Box::new(Expr::Member(MemberExpr {
          span: DUMMY_SP,
          obj: wrap_require_default(Box::new(Expr::Call(cjs_caller.clone())), polyfill),
          prop: MemberProp::Ident("default".into()),
        }))),
        definite: false,
      });
    }

    if let Some(ns) = reference_import.namespace {
      decls.push(VarDeclarator {
        span: DUMMY_SP,
        name: Pat::Ident(BindingIdent {
          id: Ident::from(bundle_variable.render_name(ns).as_str()),
          type_ann: None,
        }),
        init: Some(wrap_require_wildcard(
          Box::new(Expr::Call(cjs_caller.clone())),
          polyfill,
        )),
        definite: false,
      });
    }

    let mut ordered_keys = reference_import.named.keys().collect::<Vec<_>>();
    ordered_keys.sort();

    for imported in ordered_keys {
      let named_index = &reference_import.named[imported];
      let require_name = bundle_variable.name(*named_index);

      let is_require_default = require_name == "default";
      let init_expr = Box::new(Expr::Call(cjs_caller.clone()));

      decls.push(VarDeclarator {
        span: DUMMY_SP,
        name: Pat::Ident(BindingIdent {
          id: Ident::from(bundle_variable.render_name(*named_index).as_str()),
          type_ann: None,
        }),
        init: Some(Box::new(Expr::Member(MemberExpr {
          span: DUMMY_SP,
          obj: if is_require_default {
            wrap_require_default(init_expr, polyfill)
          } else {
            init_expr
          },
          prop: swc_ecma_ast::MemberProp::Computed(ComputedPropName {
            span: DUMMY_SP,
            expr: Box::new(Expr::Lit(Lit::Str(imported.as_str().into()))),
          }),
        }))),
        definite: false,
      });
    }

    if !decls.is_empty() {
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
