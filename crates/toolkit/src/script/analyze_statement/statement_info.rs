use crate::swc_ecma_visit::VisitWith;
use farmfe_core::module::meta_data::script::statement::{
  ExportInfo, ExportSpecifierInfo, ImportInfo, ImportSpecifierInfo, Statement, StatementId, SwcId,
};
use farmfe_core::swc_ecma_ast::{self, Module, ModuleExportName, ModuleItem, VarDeclOrExpr};
use farmfe_core::HashSet;
use swc_ecma_utils::contains_top_level_await;

use crate::script::idents_collector::DefinedIdentsCollector;

#[derive(Debug)]
pub struct AnalyzedStatementInfo {
  pub id: StatementId,
  pub import_info: Option<ImportInfo>,
  pub export_info: Option<ExportInfo>,
  pub defined_idents: HashSet<SwcId>,
  pub top_level_await: bool,
}

impl Into<Statement> for AnalyzedStatementInfo {
  fn into(self) -> Statement {
    Statement::new(
      self.id,
      self.export_info,
      self.import_info,
      self.defined_idents,
      self.top_level_await,
    )
  }
}

pub fn analyze_statement_info(id: &StatementId, stmt: &ModuleItem) -> AnalyzedStatementInfo {
  let mut defined_idents = HashSet::default();
  let mut import_info = None;
  let mut export_info = None;

  match stmt {
    ModuleItem::ModuleDecl(module_decl) => match module_decl {
      swc_ecma_ast::ModuleDecl::Import(import_decl) => {
        let source = import_decl.src.value.to_string();
        let mut specifiers = vec![];

        for specifier in &import_decl.specifiers {
          match specifier {
            swc_ecma_ast::ImportSpecifier::Namespace(ns) => {
              specifiers.push(ImportSpecifierInfo::Namespace(ns.local.to_id().into()));
              defined_idents.insert(ns.local.to_id().into());
            }
            swc_ecma_ast::ImportSpecifier::Named(named) => {
              specifiers.push(ImportSpecifierInfo::Named {
                local: named.local.to_id().into(),
                imported: named.imported.as_ref().map(|i| match i {
                  ModuleExportName::Ident(i) => i.to_id().into(),
                  _ => panic!("non-ident imported is not supported when tree shaking"),
                }),
              });
              defined_idents.insert(named.local.to_id().into());
            }
            swc_ecma_ast::ImportSpecifier::Default(default) => {
              specifiers.push(ImportSpecifierInfo::Default(default.local.to_id().into()));
              defined_idents.insert(default.local.to_id().into());
            }
          }
        }

        import_info = Some(ImportInfo {
          source,
          specifiers,
          stmt_id: *id,
        });
      }
      swc_ecma_ast::ModuleDecl::ExportAll(export_all) => {
        export_info = Some(ExportInfo {
          source: Some(export_all.src.value.to_string()),
          specifiers: vec![ExportSpecifierInfo::All],
          stmt_id: *id,
        })
      }
      swc_ecma_ast::ModuleDecl::ExportDecl(export_decl) => match &export_decl.decl {
        swc_ecma_ast::Decl::Class(class_decl) => {
          export_info = Some(ExportInfo {
            source: None,
            specifiers: vec![ExportSpecifierInfo::Named {
              local: class_decl.ident.to_id().into(),
              exported: None,
            }],
            stmt_id: *id,
          });
          defined_idents.insert(class_decl.ident.to_id().into());
        }
        swc_ecma_ast::Decl::Fn(fn_decl) => {
          export_info = Some(ExportInfo {
            source: None,
            specifiers: vec![ExportSpecifierInfo::Named {
              local: fn_decl.ident.to_id().into(),
              exported: None,
            }],
            stmt_id: *id,
          });
          defined_idents.insert(fn_decl.ident.to_id().into());
        }
        swc_ecma_ast::Decl::Var(var_decl) => {
          let mut specifiers = vec![];

          let var_defined_idents = get_defined_idents_from_var_decl(var_decl);

          for ident in &var_defined_idents {
            specifiers.push(ExportSpecifierInfo::Named {
              local: ident.clone().into(),
              exported: None,
            });
          }

          defined_idents.extend(var_defined_idents);

          export_info = Some(ExportInfo {
            source: None,
            specifiers,
            stmt_id: *id,
          });
        }
        _ => {
          unreachable!("export_decl.decl should not be anything other than a class, function, or variable declaration");
        }
      },
      swc_ecma_ast::ModuleDecl::ExportDefaultDecl(export_default_decl) => {
        export_info = Some(ExportInfo {
          source: None,
          specifiers: vec![ExportSpecifierInfo::Default],
          stmt_id: *id,
        });
        match &export_default_decl.decl {
          swc_ecma_ast::DefaultDecl::Class(class_expr) => {
            if let Some(ident) = &class_expr.ident {
              defined_idents.insert(ident.to_id().into());
            }
          }
          swc_ecma_ast::DefaultDecl::Fn(fn_decl) => {
            if let Some(ident) = &fn_decl.ident {
              defined_idents.insert(ident.to_id().into());
            }
          }
          _ => unreachable!(
            "export_default_decl.decl should not be anything other than a class, function"
          ),
        }
      }
      swc_ecma_ast::ModuleDecl::ExportDefaultExpr(_) => {
        export_info = Some(ExportInfo {
          source: None,
          specifiers: vec![ExportSpecifierInfo::Default],
          stmt_id: *id,
        });
      }
      swc_ecma_ast::ModuleDecl::ExportNamed(export_named) => {
        let mut specifiers = vec![];

        for specifier in &export_named.specifiers {
          match specifier {
            swc_ecma_ast::ExportSpecifier::Named(named) => {
              let local = match &named.orig {
                ModuleExportName::Ident(i) => i.to_id(),
                ModuleExportName::Str(_) => unimplemented!("exporting a string is not supported"),
              };
              let exported = named.exported.as_ref().map(|i| match i {
                ModuleExportName::Ident(i) => i.to_id(),
                _ => panic!("non-ident exported is not supported when tree shaking"),
              });

              if let Some(exported) = &exported {
                defined_idents.insert(exported.clone().into());
              } else {
                defined_idents.insert(local.clone().into());
              }

              specifiers.push(ExportSpecifierInfo::Named {
                local: local.into(),
                exported: exported.map(|e| e.into()),
              });
            }
            swc_ecma_ast::ExportSpecifier::Default(_) => {
              unreachable!("ExportSpecifier::Default is not valid esm syntax")
            }
            swc_ecma_ast::ExportSpecifier::Namespace(ns) => {
              let ident = match &ns.name {
                ModuleExportName::Ident(ident) => ident.to_id(),
                ModuleExportName::Str(_) => unreachable!("exporting a string is not supported"),
              };
              defined_idents.insert(ident.clone().into());
              specifiers.push(ExportSpecifierInfo::Namespace(ident.into()));
            }
          }
        }

        export_info = Some(ExportInfo {
          source: export_named.src.as_ref().map(|s| s.value.to_string()),
          specifiers,
          stmt_id: *id,
        });
      }
      _ => {}
    },
    ModuleItem::Stmt(stmt) => match stmt {
      swc_ecma_ast::Stmt::Decl(decl) => match decl {
        swc_ecma_ast::Decl::Class(class_decl) => {
          defined_idents.insert(class_decl.ident.to_id().into());
        }
        swc_ecma_ast::Decl::Fn(fn_decl) => {
          defined_idents.insert(fn_decl.ident.to_id().into());
        }
        swc_ecma_ast::Decl::Var(var_decl) => {
          defined_idents.extend(get_defined_idents_from_var_decl(var_decl));
        }
        _ => unreachable!(
          "decl should not be anything other than a class, function, or variable declaration"
        ),
      },
      swc_ecma_ast::Stmt::For(for_stmt) => {
        if let Some(VarDeclOrExpr::VarDecl(var_decl)) = &for_stmt.init {
          defined_idents.extend(get_defined_idents_from_var_decl(var_decl));
        }
      }
      swc_ecma_ast::Stmt::ForIn(for_in_stmt) => {
        if let swc_ecma_ast::ForHead::VarDecl(var_decl) = &for_in_stmt.left {
          defined_idents.extend(get_defined_idents_from_var_decl(var_decl));
        }
      }
      swc_ecma_ast::Stmt::ForOf(for_of_stmt) => {
        if let swc_ecma_ast::ForHead::VarDecl(var_decl) = &for_of_stmt.left {
          defined_idents.extend(get_defined_idents_from_var_decl(var_decl));
        }
      }
      // other statements do not define any idents
      _ => {}
    },
  };

  AnalyzedStatementInfo {
    id: *id,
    import_info,
    export_info,
    defined_idents: defined_idents.into_iter().map(|i| i.into()).collect(),
    top_level_await: contains_top_level_await(stmt),
  }
}

pub fn get_defined_idents_from_var_decl(var_decl: &swc_ecma_ast::VarDecl) -> HashSet<SwcId> {
  let mut defined_idents = HashSet::default();

  for decl in &var_decl.decls {
    let mut defined_idents_collector = DefinedIdentsCollector::new();
    decl.name.visit_with(&mut defined_idents_collector);

    for defined_ident in defined_idents_collector.defined_idents {
      defined_idents.insert(defined_ident.clone());
    }
  }

  defined_idents
}

pub fn analyze_statements(ast: &Module) -> Vec<Statement> {
  let mut statements = vec![];

  for (id, item) in ast.body.iter().enumerate() {
    let analyzed_statement_info = analyze_statement_info(&id, item);
    statements.push(analyzed_statement_info.into());
  }

  statements
}

#[cfg(test)]
mod test;
