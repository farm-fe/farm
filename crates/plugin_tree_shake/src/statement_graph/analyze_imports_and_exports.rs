use farmfe_core::{
  hashbrown::HashMap,
  swc_ecma_ast::{self, Ident, ModuleExportName, ModuleItem},
};
use farmfe_toolkit::swc_ecma_visit::VisitWith;

use super::{
  used_idents_collector::{self, UsedIdentsCollector},
  ExportInfo, ExportSpecifierInfo, ImportInfo, ImportSpecifierInfo, StatementId,
};

pub fn analyze_imports_and_exports(
  id: &StatementId,
  stmt: &ModuleItem,
  used_defined_idents: Option<Vec<String>>,
) -> (
  Option<ImportInfo>,
  Option<ExportInfo>,
  Vec<Ident>,
  Vec<Ident>,
  HashMap<Ident, Vec<Ident>>,
) {
  let mut defined_idents = vec![];
  let mut used_idents = vec![];
  let mut defined_idents_map = HashMap::new();

  let mut imports = None;
  let mut exports = None;

  let mut analyze_and_insert_used_idents =
    |stmt: &dyn VisitWith<UsedIdentsCollector>, ident: Option<Ident>| {
      let mut used_idents_collector = used_idents_collector::UsedIdentsCollector::new();
      stmt.visit_with(&mut used_idents_collector);

      if let Some(ident) = ident {
        defined_idents_map.insert(ident, used_idents_collector.used_idents.clone());
      }

      used_idents.extend(used_idents_collector.used_idents);
    };

  let is_ident_used = |ident: &Ident| {
    if let Some(used_defined_idents) = &used_defined_idents {
      return used_defined_idents.contains(&ident.to_string());
    }

    return true;
  };

  match stmt {
    ModuleItem::ModuleDecl(module_decl) => match module_decl {
      swc_ecma_ast::ModuleDecl::Import(import_decl) => {
        let source = import_decl.src.value.to_string();
        let mut specifiers = vec![];

        for specifier in &import_decl.specifiers {
          match specifier {
            swc_ecma_ast::ImportSpecifier::Namespace(ns) => {
              if !is_ident_used(&ns.local) {
                continue;
              }

              specifiers.push(ImportSpecifierInfo::Namespace(ns.local.clone()));
              defined_idents.push(ns.local.clone());
            }
            swc_ecma_ast::ImportSpecifier::Named(named) => {
              if !is_ident_used(&named.local) {
                continue;
              }

              specifiers.push(ImportSpecifierInfo::Named {
                local: named.local.clone(),
                imported: named.imported.as_ref().map(|i| match i {
                  ModuleExportName::Ident(i) => i.clone(),
                  _ => panic!("non-ident imported is not supported when tree shaking"),
                }),
              });
              defined_idents.push(named.local.clone());
            }
            swc_ecma_ast::ImportSpecifier::Default(default) => {
              if !is_ident_used(&default.local) {
                continue;
              }

              specifiers.push(ImportSpecifierInfo::Default(default.local.clone()));
              defined_idents.push(default.local.clone());
            }
          }
        }

        imports = Some(ImportInfo {
          source,
          specifiers,
          stmt_id: id.clone(),
        });
      }
      swc_ecma_ast::ModuleDecl::ExportAll(export_all) => {
        exports = Some(ExportInfo {
          source: Some(export_all.src.value.to_string()),
          specifiers: vec![ExportSpecifierInfo::All(None)],
          stmt_id: id.clone(),
        })
      }
      swc_ecma_ast::ModuleDecl::ExportDecl(export_decl) => {
        match &export_decl.decl {
          swc_ecma_ast::Decl::Class(class_decl) => {
            exports = Some(ExportInfo {
              source: None,
              specifiers: vec![ExportSpecifierInfo::Named { local: class_decl.ident.clone(), exported: None }],
              stmt_id: id.clone(),
            });
            defined_idents.push(class_decl.ident.clone());
            analyze_and_insert_used_idents(&class_decl.class, Some(class_decl.ident.clone()));
          },
          swc_ecma_ast::Decl::Fn(fn_decl) => {
            exports = Some(ExportInfo {
              source: None,
              specifiers: vec![ExportSpecifierInfo::Named { local: fn_decl.ident.clone(), exported: None }],
              stmt_id: id.clone(),
            });
            defined_idents.push(fn_decl.ident.clone());
            analyze_and_insert_used_idents(&fn_decl.function, Some(fn_decl.ident.clone()));
          },
          swc_ecma_ast::Decl::Var(var_decl) => {
            let mut specifiers = vec![];

            for v_decl in &var_decl.decls {
              match &v_decl.name {
                swc_ecma_ast::Pat::Ident(ident) => {
                  if !is_ident_used(&ident.id) {
                    continue;
                  }

                  specifiers.push(ExportSpecifierInfo::Named { local: ident.id.clone(), exported: None });
                  defined_idents.push(ident.id.clone());

                  if let Some(init) = &v_decl.init {
                    analyze_and_insert_used_idents(init, Some(ident.id.clone()));
                  }
                }
                // TODO: support other patterns
                _ => unreachable!("non-ident export_decl.decl is not supported when tree shaking"),
              }
            }

            exports = Some(ExportInfo {
              source: None,
              specifiers,
              stmt_id: id.clone(),
            });
          },
          _ => unreachable!("export_decl.decl should not be anything other than a class, function, or variable declaration"),
        }
      }
      swc_ecma_ast::ModuleDecl::ExportDefaultDecl(export_default_decl) => {
        exports = Some(ExportInfo {
          source: None,
          specifiers: vec![ExportSpecifierInfo::Default],
          stmt_id: id.clone(),
        });
        match &export_default_decl.decl {
          swc_ecma_ast::DefaultDecl::Class(class_expr) => {
            if let Some(ident) = &class_expr.ident {
              defined_idents.push(ident.clone());
            }
            analyze_and_insert_used_idents(&class_expr.class, class_expr.ident.clone());
          }
          swc_ecma_ast::DefaultDecl::Fn(fn_decl) => {
            if let Some(ident) = &fn_decl.ident {
              defined_idents.push(ident.clone());
            }
            analyze_and_insert_used_idents(&fn_decl.function, fn_decl.ident.clone());
          }
          _ => unreachable!(
            "export_default_decl.decl should not be anything other than a class, function"
          ),
        }
      }
      swc_ecma_ast::ModuleDecl::ExportDefaultExpr(export_default_expr) => {
        exports = Some(ExportInfo {
          source: None,
          specifiers: vec![ExportSpecifierInfo::Default],
          stmt_id: id.clone(),
        });
        analyze_and_insert_used_idents(&export_default_expr.expr, None);
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

              if !is_ident_used(&local) {
                continue;
              }

              if export_named.src.is_none() {
                used_idents.push(local.clone());
                defined_idents_map.insert(local.clone(), vec![local.clone()]);
              }

              specifiers.push(ExportSpecifierInfo::Named {
                local,
                exported: named.exported.as_ref().map(|i| match i {
                  ModuleExportName::Ident(i) => i.clone(),
                  _ => panic!("non-ident exported is not supported when tree shaking"),
                }),
              });
            }
            swc_ecma_ast::ExportSpecifier::Default(_) => {
              unreachable!("ExportSpecifier::Default is not valid esm syntax")
            }
            swc_ecma_ast::ExportSpecifier::Namespace(ns) => {
              let ident = match &ns.name {
                ModuleExportName::Ident(ident) => ident.clone(),
                ModuleExportName::Str(_) => unreachable!("exporting a string is not supported"),
              };

              specifiers.push(ExportSpecifierInfo::Namespace(ident));
            }
          }
        }

        exports = Some(ExportInfo {
          source: export_named.src.as_ref().map(|s| s.value.to_string()),
          specifiers,
          stmt_id: id.clone(),
        });
      }
      _ => {}
    },
    ModuleItem::Stmt(stmt) => match stmt {
      swc_ecma_ast::Stmt::Block(block) => {
        analyze_and_insert_used_idents(block, None);
      }
      swc_ecma_ast::Stmt::Empty(_) => {}
      swc_ecma_ast::Stmt::Debugger(_) => {}
      swc_ecma_ast::Stmt::With(with) => analyze_and_insert_used_idents(with, None),
      swc_ecma_ast::Stmt::Return(_) => {
        unreachable!("return statement should not be present in a module root")
      }
      swc_ecma_ast::Stmt::Labeled(label) => analyze_and_insert_used_idents(label, None),
      swc_ecma_ast::Stmt::Break(_) => {
        unreachable!("break statement should not be present in a module root")
      }
      swc_ecma_ast::Stmt::Continue(_) => {
        unreachable!("continue statement should not be present in a module root")
      }
      swc_ecma_ast::Stmt::If(if_stmt) => analyze_and_insert_used_idents(if_stmt, None),
      swc_ecma_ast::Stmt::Switch(switch_stmt) => analyze_and_insert_used_idents(switch_stmt, None),
      swc_ecma_ast::Stmt::Throw(throw) => analyze_and_insert_used_idents(throw, None),
      swc_ecma_ast::Stmt::Try(try_stmt) => analyze_and_insert_used_idents(try_stmt, None),
      swc_ecma_ast::Stmt::While(while_stmt) => analyze_and_insert_used_idents(while_stmt, None),
      swc_ecma_ast::Stmt::DoWhile(do_while) => analyze_and_insert_used_idents(do_while, None),
      swc_ecma_ast::Stmt::For(for_stmt) => analyze_and_insert_used_idents(for_stmt, None),
      swc_ecma_ast::Stmt::ForIn(for_in) => analyze_and_insert_used_idents(for_in, None),
      swc_ecma_ast::Stmt::ForOf(for_of) => analyze_and_insert_used_idents(for_of, None),
      swc_ecma_ast::Stmt::Decl(decl) => match decl {
        swc_ecma_ast::Decl::Class(class_decl) => {
          defined_idents.push(class_decl.ident.clone());
          analyze_and_insert_used_idents(&class_decl.class, Some(class_decl.ident.clone()));
        }
        swc_ecma_ast::Decl::Fn(fn_decl) => {
          defined_idents.push(fn_decl.ident.clone());
          analyze_and_insert_used_idents(&fn_decl.function, Some(fn_decl.ident.clone()));
        }
        swc_ecma_ast::Decl::Var(var_decl) => {
          for v_decl in &var_decl.decls {
            match &v_decl.name {
              swc_ecma_ast::Pat::Ident(ident) => {
                defined_idents.push(ident.id.clone());

                if let Some(init) = &v_decl.init {
                  analyze_and_insert_used_idents(init, Some(ident.id.clone()));
                }
              }
              _ => unreachable!("var_decl.decl should not be anything other than an ident"),
            }
          }
        }
        _ => unreachable!(
          "decl should not be anything other than a class, function, or variable declaration"
        ),
      },
      swc_ecma_ast::Stmt::Expr(expr) => analyze_and_insert_used_idents(expr, None),
    },
  };

  (
    imports,
    exports,
    defined_idents,
    used_idents,
    defined_idents_map,
  )
}

#[cfg(test)]
mod test;
