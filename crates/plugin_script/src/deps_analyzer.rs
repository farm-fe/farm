use farmfe_core::{
  module::ModuleId,
  plugin::{PluginAnalyzeDepsHookResultEntry, ResolveKind},
  swc_common::Mark,
  swc_ecma_ast::{
    CallExpr, ExportAll, Expr, Lit, Module, ModuleDecl, ModuleItem, NamedExport,
    TsExternalModuleRef, TsImportEqualsDecl,
  },
};

use farmfe_toolkit::{
  script::{is_commonjs_require, is_dynamic_import},
  swc_ecma_visit::{Visit, VisitWith},
};

pub struct DepsAnalyzer<'a> {
  module_id: &'a ModuleId,
  ast: &'a Module,
  deps: Option<Vec<PluginAnalyzeDepsHookResultEntry>>,
  unresolved_mark: Mark,
  top_level_mark: Mark,
}

impl<'a> DepsAnalyzer<'a> {
  pub fn new(
    module_id: &'a ModuleId,
    ast: &'a Module,
    unresolved_mark: Mark,
    top_level_mark: Mark,
  ) -> Self {
    Self {
      module_id,
      ast,
      deps: None,
      unresolved_mark,
      top_level_mark,
    }
  }

  pub fn analyze_deps(&mut self) -> Vec<PluginAnalyzeDepsHookResultEntry> {
    self.ast.visit_with(self);
    self.deps.take().unwrap_or_default()
  }

  fn insert_dep(&mut self, dep: PluginAnalyzeDepsHookResultEntry) {
    if let Some(deps) = &mut self.deps {
      deps.push(dep);
    } else {
      self.deps.replace(vec![dep]);
    }
  }
}

impl Visit for DepsAnalyzer<'_> {
  fn visit_module_item(&mut self, n: &ModuleItem) {
    match n {
      ModuleItem::ModuleDecl(decl) => match decl {
        ModuleDecl::Import(import) => {
          self.insert_dep(PluginAnalyzeDepsHookResultEntry {
            source: import.src.value.to_string(),
            kind: ResolveKind::Import,
          });
        }
        ModuleDecl::ExportAll(ExportAll { src, .. }) => {
          self.insert_dep(PluginAnalyzeDepsHookResultEntry {
            source: src.value.to_string(),
            kind: ResolveKind::ExportFrom,
          });
        }
        ModuleDecl::ExportNamed(NamedExport { src, .. }) => {
          if let Some(src) = src {
            self.insert_dep(PluginAnalyzeDepsHookResultEntry {
              source: src.value.to_string(),
              kind: ResolveKind::ExportFrom,
            });
          }
        }
        ModuleDecl::TsImportEquals(box TsImportEqualsDecl { module_ref, .. }) => match module_ref {
          farmfe_core::swc_ecma_ast::TsModuleRef::TsEntityName(_) => {
            panic!("TsEntityName not supported in {:?}", self.module_id)
          }
          farmfe_core::swc_ecma_ast::TsModuleRef::TsExternalModuleRef(TsExternalModuleRef {
            expr,
            ..
          }) => self.insert_dep(PluginAnalyzeDepsHookResultEntry {
            source: expr.value.to_string(),
            // treat TsImportEquals as require cause it only works in commonjs
            kind: ResolveKind::Require,
          }),
        },
        _ => {
          n.visit_children_with(self);
        }
      },
      _ => {
        n.visit_children_with(self);
      }
    }
  }

  fn visit_call_expr(&mut self, call_expr: &CallExpr) {
    if call_expr.args.len() != 1 {
      call_expr.visit_children_with(self);
      return;
    }

    if is_commonjs_require(self.unresolved_mark, self.top_level_mark, call_expr) {
      if let box Expr::Lit(Lit::Str(str)) = &call_expr.args[0].expr {
        self.insert_dep(PluginAnalyzeDepsHookResultEntry {
          source: str.value.to_string(),
          kind: ResolveKind::Require,
        })
      }
    } else if is_dynamic_import(call_expr) {
      if let box Expr::Lit(Lit::Str(str)) = &call_expr.args[0].expr {
        self.insert_dep(PluginAnalyzeDepsHookResultEntry {
          source: str.value.to_string(),
          kind: ResolveKind::DynamicImport,
        })
      }
    }

    call_expr.visit_children_with(self);
  }
}
