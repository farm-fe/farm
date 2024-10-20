use std::collections::HashSet;

use farmfe_core::{
  error::{CompilationError, Result},
  farm_profile_function,
  module::{module_graph::ModuleGraph, ModuleId},
  swc_common::Mark,
  swc_ecma_ast::{
    self, DefaultDecl, ExportDecl, Expr, Ident, ImportSpecifier, ModuleDecl, ModuleExportName,
    ModuleItem, Pat,
  },
};
use farmfe_toolkit::swc_ecma_visit::{Visit, VisitWith};

use crate::resource_pot_to_bundle::defined_idents_collector::DefinedIdentsCollector;

use super::module_analyzer::{
  ExportInfo, ExportSpecifierInfo, ImportInfo, ImportSpecifierInfo, Statement, StatementId,
  Variable,
};

pub struct CollectUnresolvedIdent {
  pub unresolved_ident: HashSet<String>,
  unresolved_mark: Mark,
}

impl CollectUnresolvedIdent {
  pub fn new(unresolved_mark: Mark) -> Self {
    CollectUnresolvedIdent {
      unresolved_ident: HashSet::new(),
      unresolved_mark,
    }
  }
}

impl Visit for CollectUnresolvedIdent {
  fn visit_ident(&mut self, n: &Ident) {
    if n.ctxt.outer() == self.unresolved_mark || self.unresolved_ident.contains(n.sym.as_str()) {
      self.unresolved_ident.insert(n.sym.to_string());
    }
  }
}

type RegisterVarHandle<'a> = Box<&'a mut dyn FnMut(&Ident, bool, bool) -> usize>;

struct AnalyzeModuleItem<'a> {
  id: StatementId,
  import: Option<ImportInfo>,
  export: Option<ExportInfo>,
  defined_idents: HashSet<usize>,
  module_id: &'a ModuleId,
  module_graph: &'a ModuleGraph,
  _register_var: RegisterVarHandle<'a>,
  is_in_export: bool,
  is_collect_ident: bool,
  top_level_mark: Mark,
}

impl<'a> AnalyzeModuleItem<'a> {
  fn new<F: FnMut(&Ident, bool, bool) -> usize>(
    id: StatementId,
    module_graph: &'a ModuleGraph,
    module_id: &'a ModuleId,
    register_var: &'a mut F,
    top_level_mark: Mark,
  ) -> Self {
    Self {
      id,
      import: None,
      export: None,
      defined_idents: HashSet::new(),
      module_id,
      module_graph,
      _register_var: Box::new(register_var),
      is_in_export: false,
      top_level_mark,
      is_collect_ident: false,
    }
  }

  fn into_statement(self) -> Statement {
    Statement {
      id: self.id,
      import: self.import,
      export: self.export,
      defined: self.defined_idents.into_iter().collect(),
    }
  }

  fn get_module_id_by_source(&self, source: &str) -> Result<ModuleId> {
    self
      .module_graph
      .get_dep_by_source_optional(self.module_id, source, None)
      .map(Ok)
      .unwrap_or_else(|| {
        Err(CompilationError::GenericError(
          "module_id should be found by source".to_string(),
        ))
      })
  }

  fn get_module_id_by_option_source(&self, source: Option<&str>) -> Result<Option<ModuleId>> {
    if let Some(source) = source {
      self.get_module_id_by_source(source).map(Some)
    } else {
      Ok(None)
    }
  }

  fn with_in_export<F: Fn(&mut Self)>(&mut self, v: bool, f: F) {
    let is_in_export = self.is_in_export;
    self.is_in_export = v;
    f(self);
    self.is_in_export = is_in_export;
  }

  fn with_collect_ident<F: Fn(&mut Self)>(&mut self, v: bool, f: F) {
    let is_collect_ident = self.is_collect_ident;
    self.is_collect_ident = v;
    f(self);
    self.is_collect_ident = is_collect_ident;
  }

  fn is_strict(&self, ident: &Ident, default_strict: bool) -> bool {
    default_strict || ident.ctxt.outer() != self.top_level_mark
  }

  fn register_var(&mut self, ident: &Ident, strict: bool) -> usize {
    let strict = self.is_strict(ident, strict);
    self._register_var.as_mut()(ident, strict, false)
  }

  fn register_placeholder(&mut self, ident: &Ident) -> usize {
    let strict = self.is_strict(ident, false);
    self._register_var.as_mut()(ident, strict, true)
  }
}

impl<'a> Visit for AnalyzeModuleItem<'a> {
  fn visit_module_decl(&mut self, module_decl: &ModuleDecl) {
    match module_decl {
      ModuleDecl::Import(import_decl) => {
        let source = self
          .get_module_id_by_source(import_decl.src.value.as_str())
          .unwrap();
        let mut specifiers = vec![];

        for specifier in &import_decl.specifiers {
          match specifier {
            ImportSpecifier::Namespace(ns) => {
              specifiers.push(ImportSpecifierInfo::Namespace(
                self.register_var(&ns.local, false),
              ));
            }
            ImportSpecifier::Named(named) => {
              specifiers.push(ImportSpecifierInfo::Named {
                local: self.register_var(&named.local, false),
                imported: named.imported.as_ref().map(|i| match i {
                  ModuleExportName::Ident(i) => self.register_var(i, true),
                  _ => panic!("non-ident imported is not supported when tree shaking"),
                }),
              });
            }
            ImportSpecifier::Default(default) => {
              specifiers.push(ImportSpecifierInfo::Default(
                self.register_var(&default.local, false),
              ));
            }
          }
        }

        self.import = Some(ImportInfo {
          source,
          specifiers,
          stmt_id: self.id,
        });
      }

      ModuleDecl::ExportAll(export_all) => {
        self.export = Some(ExportInfo {
          source: Some(
            self
              .get_module_id_by_source(export_all.src.value.as_str())
              .unwrap(),
          ),
          specifiers: vec![ExportSpecifierInfo::All(None)],
          stmt_id: self.id,
        })
      }

      ModuleDecl::ExportDefaultDecl(export_default_decl) => {
        let mut specify = vec![];

        match &export_default_decl.decl {
          DefaultDecl::Class(class_expr) => {
            if let Some(ident) = &class_expr.ident {
              specify.push(ExportSpecifierInfo::Default(
                self.register_var(ident, false),
              ));
            } else {
              specify.push(ExportSpecifierInfo::Default(
                self.register_var(&"default".into(), false),
              ))
            };

            self.with_in_export(false, |this| class_expr.class.visit_with(this));
          }

          DefaultDecl::Fn(fn_decl) => {
            if let Some(ident) = &fn_decl.ident {
              specify.push(ExportSpecifierInfo::Default(
                self.register_var(ident, false),
              ));
            } else {
              specify.push(ExportSpecifierInfo::Default(
                self.register_var(&"default".into(), false),
              ))
            }
            self.with_in_export(false, |this| fn_decl.function.visit_with(this));
          }

          _ => unreachable!(
            "export_default_decl.decl should not be anything other than a class, function"
          ),
        }

        self.export = Some(ExportInfo {
          source: None,
          specifiers: specify,
          stmt_id: self.id,
        });
      }

      ModuleDecl::ExportDefaultExpr(export_default_expr) => match &export_default_expr.expr {
        box Expr::Ident(ref ident) => {
          self.export = Some(ExportInfo {
            source: None,
            specifiers: vec![ExportSpecifierInfo::Default(
              self.register_var(ident, false),
            )],
            stmt_id: self.id,
          });
        }

        _ => {
          self.export = Some(ExportInfo {
            source: None,
            specifiers: vec![ExportSpecifierInfo::Default(
              self.register_var(&Ident::from("default"), false),
            )],
            stmt_id: self.id,
          });
        }
      },

      ModuleDecl::ExportNamed(export_named) => {
        let mut specifiers = vec![];

        for specifier in &export_named.specifiers {
          match specifier {
            swc_ecma_ast::ExportSpecifier::Named(named) => {
              let local = match &named.orig {
                ModuleExportName::Ident(i) => i.clone(),
                ModuleExportName::Str(_) => unimplemented!("exporting a string is not supported"),
              };

              specifiers.push(ExportSpecifierInfo::Named(
                (
                  self.register_var(&local, false),
                  named.exported.as_ref().map(|i| match i {
                    ModuleExportName::Ident(i) => self.register_var(i, false),
                    _ => panic!("non-ident exported is not supported when tree shaking"),
                  }),
                )
                  .into(),
              ));
            }
            swc_ecma_ast::ExportSpecifier::Default(_) => {
              unreachable!("ExportSpecifier::Default is not valid esm syntax")
            }
            swc_ecma_ast::ExportSpecifier::Namespace(ns) => {
              let ident = match &ns.name {
                ModuleExportName::Ident(ident) => self.register_var(ident, false),
                ModuleExportName::Str(_) => unreachable!("exporting a string is not supported"),
              };

              specifiers.push(ExportSpecifierInfo::Namespace(ident));
            }
          }
        }

        self.export = Some(ExportInfo {
          source: self
            .get_module_id_by_option_source(export_named.src.as_ref().map(|s| s.value.as_str()))
            .unwrap(),
          specifiers,
          stmt_id: self.id,
        });
      }

      ModuleDecl::ExportDecl(ExportDecl { decl, .. }) => {
        self.with_in_export(true, |mut this| decl.visit_with(&mut this))
      }

      _ => {
        module_decl.visit_children_with(self);
      }
    }
  }

  fn visit_var_decl(&mut self, n: &swc_ecma_ast::VarDecl) {
    let is_export = self.is_in_export;
    let mut specifiers = vec![];

    for v_decl in &n.decls {
      let mut defined_idents_collector = DefinedIdentsCollector::new();
      v_decl.name.visit_with(&mut defined_idents_collector);

      for defined_ident in defined_idents_collector.defined_idents {
        if is_export {
          specifiers.push(ExportSpecifierInfo::Named(
            self.register_var(&Ident::from(defined_ident), false).into(),
          ));
        } else {
          let index = self.register_var(&Ident::from(defined_ident), false);
          self.defined_idents.insert(index);
        }
      }

      self.with_in_export(false, |this| v_decl.init.visit_with(this));
    }

    if is_export {
      self.export = Some(ExportInfo {
        source: None,
        specifiers,
        stmt_id: self.id,
      });
    }
  }

  fn visit_class_decl(&mut self, n: &swc_ecma_ast::ClassDecl) {
    let is_export = self.is_in_export;
    if is_export {
      self.export = Some(ExportInfo {
        source: None,
        specifiers: vec![ExportSpecifierInfo::Named(Variable(
          self.register_var(&n.ident, false),
          None,
        ))],
        stmt_id: self.id,
      });
    } else {
      let index = self.register_var(&n.ident, false);
      self.defined_idents.insert(index);
    }

    self.with_in_export(false, |this| n.class.visit_with(this));
  }

  fn visit_fn_decl(&mut self, n: &swc_ecma_ast::FnDecl) {
    let is_export = self.is_in_export;
    if is_export {
      self.export = Some(ExportInfo {
        source: None,
        specifiers: vec![ExportSpecifierInfo::Named(
          self.register_var(&n.ident, false).into(),
        )],
        stmt_id: self.id,
      })
    } else {
      let index = self.register_var(&n.ident, false);
      self.defined_idents.insert(index);
    };

    self.with_in_export(false, |this| n.function.visit_with(this));
  }

  fn visit_fn_expr(&mut self, n: &swc_ecma_ast::FnExpr) {
    if let Some(ref x) = n.ident {
      let index = self.register_var(x, false);
      self.defined_idents.insert(index);
    }

    n.function.visit_with(self);
  }

  fn visit_class_expr(&mut self, n: &swc_ecma_ast::ClassExpr) {
    if let Some(ref x) = n.ident {
      let index = self.register_var(x, false);
      self.defined_idents.insert(index);
    }

    n.class.visit_with(self);
  }

  // fn visit_arrow_expr(&mut self, n: &swc_ecma_ast::ArrowExpr) {}

  fn visit_ident(&mut self, n: &Ident) {
    if self.is_collect_ident {
      let index = self.register_placeholder(n);
      self.defined_idents.insert(index);
    }
  }

  fn visit_pat(&mut self, n: &swc_ecma_ast::Pat) {
    match n {
      Pat::Assign(assign) => {
        self.with_collect_ident(true, |this| assign.left.visit_with(this));
      }

      Pat::Ident(ident) => {
        self.with_collect_ident(true, |this| ident.visit_with(this));
      }

      _ => {
        n.visit_children_with(self);
      }
    }
  }
}

pub fn analyze_imports_and_exports<F: FnMut(&Ident, bool, bool) -> usize>(
  id: StatementId,
  stmt: &ModuleItem,
  module_id: &ModuleId,
  module_graph: &ModuleGraph,
  top_level_mark: Mark,
  register_var: &mut F,
) -> Result<Statement> {
  farm_profile_function!();

  let mut m = AnalyzeModuleItem::new(id, module_graph, module_id, register_var, top_level_mark);

  stmt.visit_with(&mut m);

  Ok(m.into_statement())
}
