use farmfe_core::swc_ecma_ast::{self, Ident, ModuleExportName, ModuleItem};
use farmfe_toolkit::swc_ecma_visit::VisitWith;

use super::{
  used_idents_collector::{self, UsedIdentsCollector},
  ExportInfo, ExportSpecifierInfo, ImportInfo, ImportSpecifierInfo, StatementId,
};

pub fn analyze_imports_and_exports(
  id: &StatementId,
  stmt: &ModuleItem,
) -> (
  Option<ImportInfo>,
  Option<ExportInfo>,
  Vec<Ident>,
  Vec<Ident>,
) {
  let mut defined_idents = vec![];
  let mut used_idents = vec![];

  let mut imports = None;
  let mut exports = None;

  let mut analyze_and_insert_used_idents = |stmt: &dyn VisitWith<UsedIdentsCollector>| {
    let mut used_idents_collector = used_idents_collector::UsedIdentsCollector::new();
    stmt.visit_with(&mut used_idents_collector);
    used_idents.extend(used_idents_collector.used_idents);
  };

  match stmt {
    ModuleItem::ModuleDecl(module_decl) => match module_decl {
      swc_ecma_ast::ModuleDecl::Import(import_decl) => {
        let source = import_decl.src.value.to_string();
        let mut specifiers = vec![];

        for specifier in &import_decl.specifiers {
          match specifier {
            swc_ecma_ast::ImportSpecifier::Namespace(ns) => {
              specifiers.push(ImportSpecifierInfo::Namespace(ns.local.clone()));
              defined_idents.push(ns.local.clone());
            }
            swc_ecma_ast::ImportSpecifier::Named(named) => {
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
          specifiers: vec![ExportSpecifierInfo::All],
        })
      }
      swc_ecma_ast::ModuleDecl::ExportDecl(export_decl) => {
        match &export_decl.decl {
          swc_ecma_ast::Decl::Class(class_decl) => {
            exports = Some(ExportInfo {
              source: None,
              specifiers: vec![ExportSpecifierInfo::Named { local: class_decl.ident.clone(), exported: None }]
            });
            defined_idents.push(class_decl.ident.clone());
            analyze_and_insert_used_idents(&class_decl.class);
          },
          swc_ecma_ast::Decl::Fn(fn_decl) => {
            exports = Some(ExportInfo {
              source: None,
              specifiers: vec![ExportSpecifierInfo::Named { local: fn_decl.ident.clone(), exported: None }]
            });
            defined_idents.push(fn_decl.ident.clone());
            analyze_and_insert_used_idents(&fn_decl.function);
          },
          swc_ecma_ast::Decl::Var(var_decl) => {
            let mut specifiers = vec![];

            for v_decl in &var_decl.decls {
              match &v_decl.name {
                swc_ecma_ast::Pat::Ident(ident) => {
                  specifiers.push(ExportSpecifierInfo::Named { local: ident.id.clone(), exported: None });
                  defined_idents.push(ident.id.clone());

                  if let Some(init) = &v_decl.init {
                    analyze_and_insert_used_idents(init);
                  }
                }
                _ => unreachable!("export_decl.decl should not be anything other than an ident")
              }
            }

            exports = Some(ExportInfo {
              source: None,
              specifiers
            });
          },
          _ => unreachable!("export_decl.decl should not be anything other than a class, function, or variable declaration"),
        }
      }
      swc_ecma_ast::ModuleDecl::ExportDefaultDecl(export_default_decl) => {
        exports = Some(ExportInfo {
          source: None,
          specifiers: vec![ExportSpecifierInfo::Default]
        });
        match &export_default_decl.decl {
          swc_ecma_ast::DefaultDecl::Class(class_expr) => {
            if let Some(ident) = &class_expr.ident {
              defined_idents.push(ident.clone());
            }
            analyze_and_insert_used_idents(&class_expr.class);
          },
          swc_ecma_ast::DefaultDecl::Fn(fn_decl) => {
            if let Some(ident) = &fn_decl.ident {
              defined_idents.push(ident.clone());
            }
            analyze_and_insert_used_idents(&fn_decl.function);
          },
          _ => unreachable!("export_default_decl.decl should not be anything other than a class, function, or interface declaration"),
        }
      }
      swc_ecma_ast::ModuleDecl::ExportDefaultExpr(export_default_expr) => {
        exports = Some(ExportInfo {
          source: None,
          specifiers: vec![ExportSpecifierInfo::Default]
        });
        analyze_and_insert_used_idents(&export_default_expr.expr);
      }
      swc_ecma_ast::ModuleDecl::ExportNamed(export_named) => {
        let mut specifiers = vec![];

        for specifier in &export_named.specifiers {
          match specifier {
            swc_ecma_ast::ExportSpecifier::Named(named) => {
              specifiers.push(ExportSpecifierInfo::Named {
                local: match &named.orig {
                  ModuleExportName::Ident(i) => i.clone(),
                  ModuleExportName::Str(_) => unimplemented!("exporting a string is not supported"),
              },
                exported: named.exported.as_ref().map(|i| match i {
                  ModuleExportName::Ident(i) => i.clone(),
                  _ => panic!("non-ident exported is not supported when tree shaking"),
                }),
              });
            }
            swc_ecma_ast::ExportSpecifier::Default(_) => {
              specifiers.push(ExportSpecifierInfo::Default);
            }
            swc_ecma_ast::ExportSpecifier::Namespace(ns) => {
              specifiers.push(ExportSpecifierInfo::Named { local: match &ns.name {
                  ModuleExportName::Ident(ident) => ident.clone(),
                  ModuleExportName::Str(_) => unreachable!("exporting a string is not supported"),
              }, exported: None });
            }
          }
        }

        exports = Some(ExportInfo {
          source: export_named.src.as_ref().map(|s| s.value.to_string()),
          specifiers,
        });
      }
      _ => {}
    },
    ModuleItem::Stmt(stmt) => {
      match stmt {
        swc_ecma_ast::Stmt::Block(block) => {
          analyze_and_insert_used_idents(block);
        },
        swc_ecma_ast::Stmt::Empty(_) => {},
        swc_ecma_ast::Stmt::Debugger(_) => {},
        swc_ecma_ast::Stmt::With(with) => analyze_and_insert_used_idents(with),
        swc_ecma_ast::Stmt::Return(_) => unreachable!("return statement should not be present in a module root"),
        swc_ecma_ast::Stmt::Labeled(label) => analyze_and_insert_used_idents(label),
        swc_ecma_ast::Stmt::Break(_) => unreachable!("break statement should not be present in a module root"),
        swc_ecma_ast::Stmt::Continue(_) => unreachable!("continue statement should not be present in a module root"),
        swc_ecma_ast::Stmt::If(if_stmt) => analyze_and_insert_used_idents(if_stmt),
        swc_ecma_ast::Stmt::Switch(switch_stml) => analyze_and_insert_used_idents(switch_stml),
        swc_ecma_ast::Stmt::Throw(throw) => analyze_and_insert_used_idents(throw),
        swc_ecma_ast::Stmt::Try(try_stmt) => analyze_and_insert_used_idents(try_stmt),
        swc_ecma_ast::Stmt::While(while_stml) => analyze_and_insert_used_idents(while_stml),
        swc_ecma_ast::Stmt::DoWhile(do_while) => analyze_and_insert_used_idents(do_while),
        swc_ecma_ast::Stmt::For(for_stmt) => analyze_and_insert_used_idents(for_stmt),
        swc_ecma_ast::Stmt::ForIn(for_in) => analyze_and_insert_used_idents(for_in),
        swc_ecma_ast::Stmt::ForOf(for_of) => analyze_and_insert_used_idents(for_of),
        swc_ecma_ast::Stmt::Decl(decl) => {
          match decl {
            swc_ecma_ast::Decl::Class(class_decl) => {
              defined_idents.push(class_decl.ident.clone());
              analyze_and_insert_used_idents(&class_decl.class);
            },
            swc_ecma_ast::Decl::Fn(fn_decl) => {
              defined_idents.push(fn_decl.ident.clone());
              analyze_and_insert_used_idents(&fn_decl.function);
            },
            swc_ecma_ast::Decl::Var(var_decl) => {
              for v_decl in &var_decl.decls {
                match &v_decl.name {
                  swc_ecma_ast::Pat::Ident(ident) => {
                    defined_idents.push(ident.id.clone());

                    if let Some(init) = &v_decl.init {
                      analyze_and_insert_used_idents(init);
                    }
                  }
                  _ => unreachable!("var_decl.decl should not be anything other than an ident")
                }
              }
            },
            _ => unreachable!("decl should not be anything other than a class, function, or variable declaration"),
          }
        },
        swc_ecma_ast::Stmt::Expr(expr) => analyze_and_insert_used_idents(expr),
    }
    }
  };

  (imports, exports, defined_idents, used_idents)
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use farmfe_core::{
    swc_common::{FilePathMapping, SourceMap},
    swc_ecma_ast::ModuleItem,
    swc_ecma_parser::Syntax,
  };
  use farmfe_toolkit::script::parse_module;

  use crate::statement_graph::{ExportSpecifierInfo, ImportSpecifierInfo};

  use super::analyze_imports_and_exports;

  fn parse_module_item(stmt: &str) -> ModuleItem {
    let module = parse_module(
      "any",
      stmt,
      Syntax::Es(Default::default()),
      farmfe_core::swc_ecma_ast::EsVersion::Es2015,
      Arc::new(SourceMap::new(FilePathMapping::empty())),
    )
    .unwrap();
    module.body[0].clone()
  }

  #[test]
  fn import_default() {
    let stmt = parse_module_item(r#"import a from 'a'"#);

    let (import_info, export_info, defined_idents, used_idents) =
      analyze_imports_and_exports(&0, &stmt);

    assert!(import_info.is_some());
    let import_info = import_info.unwrap();
    assert_eq!(import_info.source, "a".to_string());
    assert!(matches!(
      import_info.specifiers[0],
      ImportSpecifierInfo::Default(_)
    ));

    if let ImportSpecifierInfo::Default(ident) = &import_info.specifiers[0] {
      assert_eq!(ident.sym.to_string(), "a".to_string());
    }

    assert!(export_info.is_none());
    assert_eq!(defined_idents.len(), 1);
    assert_eq!(defined_idents[0].sym.to_string(), "a".to_string());
    assert_eq!(used_idents.len(), 0);
  }

  #[test]
  fn import_named() {
    let stmt = parse_module_item(r#"import { a, b, c as nc } from 'a'"#);

    let (import_info, export_info, defined_idents, used_idents) =
      analyze_imports_and_exports(&0, &stmt);

    assert!(import_info.is_some());
    let import_info = import_info.unwrap();
    assert_eq!(import_info.source, "a".to_string());
    assert!(matches!(
      import_info.specifiers[0],
      ImportSpecifierInfo::Named { .. }
    ));

    if let ImportSpecifierInfo::Named { local, imported } = &import_info.specifiers[0] {
      assert_eq!(local.sym.to_string(), "a".to_string());
      assert!(imported.is_none());
    }

    assert!(matches!(
      import_info.specifiers[1],
      ImportSpecifierInfo::Named { .. }
    ));

    if let ImportSpecifierInfo::Named { local, imported } = &import_info.specifiers[1] {
      assert_eq!(local.sym.to_string(), "b".to_string());
      assert!(imported.is_none());
    }

    assert!(matches!(
      import_info.specifiers[2],
      ImportSpecifierInfo::Named { .. }
    ));

    if let ImportSpecifierInfo::Named { local, imported } = &import_info.specifiers[2] {
      assert_eq!(local.sym.to_string(), "nc".to_string());
      assert!(imported.is_some());
      assert_eq!(imported.as_ref().unwrap().sym.to_string(), "c".to_string());
    }

    assert!(export_info.is_none());
    assert_eq!(defined_idents.len(), 3);
    assert_eq!(defined_idents[0].sym.to_string(), "a".to_string());
    assert_eq!(defined_idents[1].sym.to_string(), "b".to_string());
    assert_eq!(defined_idents[2].sym.to_string(), "nc".to_string());
    assert_eq!(used_idents.len(), 0);
  }

  #[test]
  fn import_namespace() {
    let stmt = parse_module_item(r#"import * as a from 'a'"#);

    let (import_info, export_info, defined_idents, used_idents) =
      analyze_imports_and_exports(&0, &stmt);

    assert!(import_info.is_some());
    let import_info = import_info.unwrap();
    assert_eq!(import_info.source, "a".to_string());
    assert!(matches!(
      import_info.specifiers[0],
      ImportSpecifierInfo::Namespace(_)
    ));

    if let ImportSpecifierInfo::Namespace(ident) = &import_info.specifiers[0] {
      assert_eq!(ident.sym.to_string(), "a".to_string());
    }

    assert!(export_info.is_none());
    assert_eq!(defined_idents.len(), 1);
    assert_eq!(defined_idents[0].sym.to_string(), "a".to_string());
    assert_eq!(used_idents.len(), 0);
  }

  #[test]
  fn export_default_expr() {
    let stmt = parse_module_item(r#"export default a"#);

    let (import_info, export_info, defined_idents, used_idents) =
      analyze_imports_and_exports(&0, &stmt);

    assert!(import_info.is_none());
    assert!(export_info.is_some());
    let export_info = export_info.unwrap();

    assert_eq!(export_info.specifiers.len(), 1);
    assert!(matches!(
      export_info.specifiers[0],
      ExportSpecifierInfo::Default
    ));

    assert_eq!(defined_idents.len(), 0);
    assert_eq!(used_idents.len(), 1);
    assert_eq!(used_idents[0].sym.to_string(), "a".to_string());
  }

  #[test]
  fn export_default_decl() {
    let stmt = parse_module_item(r#"export default function a() { return b; }"#);

    let (import_info, export_info, defined_idents, used_idents) =
      analyze_imports_and_exports(&0, &stmt);

    assert!(import_info.is_none());
    assert!(export_info.is_some());
    let export_info = export_info.unwrap();

    assert_eq!(export_info.specifiers.len(), 1);
    assert!(matches!(
      export_info.specifiers[0],
      ExportSpecifierInfo::Default
    ));

    assert_eq!(defined_idents.len(), 1);
    assert_eq!(defined_idents[0].sym.to_string(), "a".to_string());
    assert_eq!(used_idents.len(), 1);
    assert_eq!(used_idents[0].sym.to_string(), "b".to_string());
  }

  #[test]
  fn export_decl() {
    let stmt = parse_module_item(r#"export function a() { return b; }"#);

    let (import_info, export_info, defined_idents, used_idents) =
      analyze_imports_and_exports(&0, &stmt);

    assert!(import_info.is_none());
    assert!(export_info.is_some());
    let export_info = export_info.unwrap();

    assert_eq!(export_info.specifiers.len(), 1);
    assert!(matches!(
      export_info.specifiers[0],
      ExportSpecifierInfo::Named { .. }
    ));

    if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[0] {
      assert_eq!(local.sym.to_string(), "a".to_string());
      assert!(exported.is_none());
    }

    assert_eq!(defined_idents.len(), 1);
    assert_eq!(defined_idents[0].sym.to_string(), "a".to_string());
    assert_eq!(used_idents.len(), 1);
    assert_eq!(used_idents[0].sym.to_string(), "b".to_string());
  }

  #[test]
  fn export_all() {
    let stmt = parse_module_item(r#"export * from 'a'"#);

    let (import_info, export_info, defined_idents, used_idents) =
      analyze_imports_and_exports(&0, &stmt);

    assert!(import_info.is_none());
    assert!(export_info.is_some());
    let export_info = export_info.unwrap();

    assert_eq!(export_info.specifiers.len(), 1);
    assert!(matches!(
      export_info.specifiers[0],
      ExportSpecifierInfo::All
    ));
    assert_eq!(export_info.source, Some("a".to_string()));

    assert_eq!(defined_idents.len(), 0);
    assert_eq!(used_idents.len(), 0);
  }
}
