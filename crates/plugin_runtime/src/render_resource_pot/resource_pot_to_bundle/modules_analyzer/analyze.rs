use std::collections::HashSet;

use farmfe_core::{
  error::{CompilationError, Result}, farm_profile_function, module::{module_graph::ModuleGraph, ModuleId}, swc_common::Mark, swc_ecma_ast::{self, ExportDecl, Expr, Ident, ModuleDecl, ModuleExportName, ModuleItem, Stmt}
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
    if n.span.ctxt.outer() == self.unresolved_mark || self.unresolved_ident.contains(n.sym.as_str())
    {
      self.unresolved_ident.insert(n.sym.to_string());
    }
  }
}

pub fn analyze_imports_and_exports(
  id: StatementId,
  stmt: &ModuleItem,
  module_id: &ModuleId,
  module_graph: &ModuleGraph,
  register_var: &mut impl FnMut(&Ident, bool) -> usize,
) -> Result<Statement> {
  farm_profile_function!("");
  let mut defined_idents: HashSet<usize> = HashSet::new();

  let mut imports: Option<ImportInfo> = None;
  let mut exports = None;
  let get_module_id_by_source = |source: &str| {
    module_graph
      .get_dep_by_source_optional(module_id, source, None)
      .map(Ok)
      .unwrap_or_else(|| {
        Err(CompilationError::GenericError(
          "module_id should be found by source".to_string(),
        ))
      })
  };

  let get_module_id_by_option_source = |source: Option<&str>| {
    if let Some(source) = source {
      get_module_id_by_source(source).map(|r| Some(r))
    } else {
      Ok(None)
    }
  };

  match stmt {
    ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl { decl, .. }))
    | ModuleItem::Stmt(Stmt::Decl(decl)) => {
      let is_export = matches!(stmt, ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(_)));
      match decl {
        swc_ecma_ast::Decl::Class(class_decl) => {
          if is_export {
            exports = Some(ExportInfo {
              source: None,
              specifiers: vec![ExportSpecifierInfo::Named(Variable(
                register_var(&class_decl.ident, false),
                None,
              ))],
              stmt_id: id,
            });
          } else {
            defined_idents.insert(register_var(&class_decl.ident, false));
          }
        }
        swc_ecma_ast::Decl::Fn(fn_decl) => {
          if is_export {
            exports = Some(ExportInfo {
              source: None,
              specifiers: vec![ExportSpecifierInfo::Named(
                register_var(&fn_decl.ident, false).into(),
              )],
              stmt_id: id,
            })
          } else {
            defined_idents.insert(register_var(&fn_decl.ident, false));
          }
          // analyze_and_insert_used_idents(&fn_decl.function, Some(fn_decl.ident.to_string()));
        }
        swc_ecma_ast::Decl::Var(var_decl) => {
          let mut specifiers = vec![];

          for v_decl in &var_decl.decls {
            let mut defined_idents_collector = DefinedIdentsCollector::new();
            v_decl.name.visit_with(&mut defined_idents_collector);

            for defined_ident in defined_idents_collector.defined_idents {
              if is_export {
                specifiers.push(ExportSpecifierInfo::Named(
                  register_var(&Ident::from(defined_ident), false).into(),
                ));
              } else {
                defined_idents.insert(register_var(&Ident::from(defined_ident), false));
              }
            }
          }

          if is_export {
            exports = Some(ExportInfo {
              source: None,
              specifiers,
              stmt_id: id,
            });
          }
        }
        _ => {
          unreachable!("export_decl.decl should not be anything other than a class, function, or variable declaration");
        }
      }
    }

    ModuleItem::ModuleDecl(module_decl) => match module_decl {
      swc_ecma_ast::ModuleDecl::Import(import_decl) => {
        let source = get_module_id_by_source(import_decl.src.value.as_str())?;
        let mut specifiers = vec![];

        for specifier in &import_decl.specifiers {
          match specifier {
            swc_ecma_ast::ImportSpecifier::Namespace(ns) => {
              specifiers.push(ImportSpecifierInfo::Namespace(register_var(
                &ns.local, false,
              )));
            }
            swc_ecma_ast::ImportSpecifier::Named(named) => {
              specifiers.push(ImportSpecifierInfo::Named {
                local: register_var(&named.local, false),
                imported: named.imported.as_ref().map(|i| match i {
                  ModuleExportName::Ident(i) => register_var(&i, true),
                  _ => panic!("non-ident imported is not supported when tree shaking"),
                }),
              });
            }
            swc_ecma_ast::ImportSpecifier::Default(default) => {
              specifiers.push(ImportSpecifierInfo::Default(register_var(
                &default.local,
                false,
              )));
            }
          }
        }

        imports = Some(ImportInfo {
          source,
          specifiers,
          stmt_id: id,
        });
      }
      swc_ecma_ast::ModuleDecl::ExportAll(export_all) => {
        exports = Some(ExportInfo {
          source: Some(get_module_id_by_source(export_all.src.value.as_str())?),
          specifiers: vec![ExportSpecifierInfo::All(None)],
          stmt_id: id,
        })
      }
      swc_ecma_ast::ModuleDecl::ExportDefaultDecl(export_default_decl) => {
        let mut specify = vec![];

        match &export_default_decl.decl {
          swc_ecma_ast::DefaultDecl::Class(class_expr) => {
            // TODO: no ident case
            if let Some(ident) = &class_expr.ident {
              specify.push(ExportSpecifierInfo::Default(register_var(&ident, false)));
            }
          }

          swc_ecma_ast::DefaultDecl::Fn(fn_decl) => {
            // TODO: no ident case
            if let Some(ident) = &fn_decl.ident {
              specify.push(ExportSpecifierInfo::Default(register_var(&ident, false)));
            }
          }

          _ => unreachable!(
            "export_default_decl.decl should not be anything other than a class, function"
          ),
        }

        exports = Some(ExportInfo {
          source: None,
          specifiers: specify,
          stmt_id: id,
        });
      }
      swc_ecma_ast::ModuleDecl::ExportDefaultExpr(export_default_expr) => {
        match &export_default_expr.expr {
          box Expr::Ident(ident) => {
            exports = Some(ExportInfo {
              source: None,
              specifiers: vec![ExportSpecifierInfo::Default(register_var(&ident, false))],
              stmt_id: id,
            });
          }
          _ => {
            exports = Some(ExportInfo {
              source: None,
              specifiers: vec![ExportSpecifierInfo::Default(register_var(
                &Ident::from("default"),
                false,
              ))],
              stmt_id: id,
            });
          }
        }
      }
      swc_ecma_ast::ModuleDecl::ExportNamed(export_named) => {
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
                  register_var(&local, false),
                  named.exported.as_ref().map(|i| match i {
                    ModuleExportName::Ident(i) => register_var(&i, false),
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
                ModuleExportName::Ident(ident) => register_var(&ident, false),
                ModuleExportName::Str(_) => unreachable!("exporting a string is not supported"),
              };

              specifiers.push(ExportSpecifierInfo::Namespace(ident));
            }
          }
        }

        exports = Some(ExportInfo {
          source: get_module_id_by_option_source(
            export_named.src.as_ref().map(|s| s.value.as_str()),
          )?,
          specifiers,
          stmt_id: id,
        });
      }
      _ => {}
    },
    _ => {}
  };

  Ok(Statement {
    id,
    import: imports,
    export: exports,
    defined: defined_idents.into_iter().collect(),
  })
}
