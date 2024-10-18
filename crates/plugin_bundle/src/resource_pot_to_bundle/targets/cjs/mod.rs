use std::collections::HashMap;

use farmfe_core::{
  error::Result,
  farm_profile_function,
  module::{module_graph::ModuleGraph, ModuleId},
  swc_common::{Mark, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    self, BindingIdent, CallExpr, ComputedPropName, Expr, ExprOrSpread, ExprStmt, Ident, Lit,
    MemberExpr, MemberProp, Module as EcmaAstModule, ModuleItem, Pat, Stmt, VarDecl, VarDeclarator,
  },
};
use farmfe_toolkit::{
  itertools::Itertools,
  script::is_commonjs_require,
  swc_ecma_visit::{Visit, VisitWith},
};

use crate::resource_pot_to_bundle::{
  bundle::{
    bundle_reference::{
      CommonJsImportMap, ExternalReferenceExport, ExternalReferenceImport, ReferenceKind,
    },
    ModuleGlobalUniqName,
  },
  polyfill::SimplePolyfill,
  targets::util::{wrap_require_default, wrap_require_wildcard},
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
    bundle_variable: &BundleVariable,
    import_map: &CommonJsImportMap,
    module_global_uniq_name: &ModuleGlobalUniqName,
    polyfill: &mut SimplePolyfill,
  ) -> Result<Vec<ModuleItem>> {
    let mut result = vec![];

    let mut generate_import_specifies: HashMap<String, CommonJsDeclareResult> = HashMap::new();

    for source in import_map.keys().sorted() {
      let import = &import_map[source];
      let Some((name, r)) = Self::redeclare_commonjs_export_item(
        bundle_variable,
        (source, &import),
        module_global_uniq_name,
        polyfill,
      )?
      else {
        continue;
      };

      generate_import_specifies.insert(name, r);
    }

    let mut generate_import_ordered = generate_import_specifies
      .keys()
      .cloned()
      .collect::<Vec<_>>();
    generate_import_ordered.sort();

    for name in generate_import_ordered {
      let decls = generate_import_specifies.remove(&name).unwrap();

      match decls {
        CommonJsDeclareResult::VarDecl(decls) => result.push(ModuleItem::Stmt(Stmt::Decl(
          swc_ecma_ast::Decl::Var(Box::new(VarDecl {
            ctxt: SyntaxContext::empty(),
            span: DUMMY_SP,
            kind: swc_ecma_ast::VarDeclKind::Var,
            declare: false,
            decls: decls.clone(),
          })),
        ))),
        CommonJsDeclareResult::Execute(expr) => {
          result.push(expr);
        }
      }
    }

    Ok(result)
  }

  pub fn redeclare_commonjs_export_item(
    bundle_variable: &BundleVariable,
    (source, import_map): (&ReferenceKind, &ExternalReferenceImport),
    module_global_uniq_name: &ModuleGlobalUniqName,
    polyfill: &mut SimplePolyfill,
  ) -> Result<Option<(String, CommonJsDeclareResult)>> {
    let module_id = match source {
      ReferenceKind::Bundle(_) => return Ok(None),
      ReferenceKind::Module(m) => m,
    };

    let cjs_name =
      bundle_variable.render_name(module_global_uniq_name.commonjs_name(module_id).unwrap());

    let cjs_caller = CallExpr {
      ctxt: SyntaxContext::empty(),
      span: DUMMY_SP,
      callee: swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(cjs_name.as_str().into()))),
      args: vec![],
      type_args: None,
    };

    if import_map.is_empty() {
      return Ok(Some((
        cjs_name,
        CommonJsDeclareResult::Execute(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
          span: DUMMY_SP,
          expr: Box::new(Expr::Call(cjs_caller)),
        }))),
      )));
    }

    let mut decls = vec![];

    if let Some(default) = import_map.default {
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

    if let Some(ns) = import_map.namespace {
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

    for imported in import_map.named.keys().sorted() {
      let named_index = &import_map.named[imported];
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

    Ok(Some((cjs_name, CommonJsDeclareResult::VarDecl(decls))))
  }
}

#[derive(Debug)]
pub enum CommonJsDeclareResult {
  VarDecl(Vec<VarDeclarator>),
  Execute(ModuleItem),
}

impl CommonJsDeclareResult {
  fn merge(&mut self, other: CommonJsDeclareResult) {
    match self {
      CommonJsDeclareResult::VarDecl(ref mut decls) => {
        if let CommonJsDeclareResult::VarDecl(other_decls) = other {
          decls.extend(other_decls);
        }
      }
      CommonJsDeclareResult::Execute(_) => {
        if matches!(other, CommonJsDeclareResult::VarDecl(_)) {
          *self = other;
        }
      }
    }
  }

  pub fn to_module_item(self) -> ModuleItem {
    match self {
      CommonJsDeclareResult::VarDecl(decls) => {
        ModuleItem::Stmt(Stmt::Decl(swc_ecma_ast::Decl::Var(Box::new(VarDecl {
          ctxt: SyntaxContext::empty(),
          span: DUMMY_SP,
          kind: swc_ecma_ast::VarDeclKind::Var,
          declare: false,
          decls: decls.clone(),
        }))))
      }
      CommonJsDeclareResult::Execute(expr) => expr,
    }
  }
}
