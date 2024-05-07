use std::{
  cell::RefMut,
  collections::{HashMap, HashSet},
  path::PathBuf,
  sync::Arc,
};

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{module_graph::ModuleGraph, Module, ModuleId},
  resource::resource_pot::ResourcePotId,
  swc_common::SourceMap,
  swc_ecma_ast::{
    self, ExportDecl, Expr, Id, Ident, Module as EcmaAstModule, ModuleDecl, ModuleExportName,
    ModuleItem, Stmt,
  },
};
use farmfe_toolkit::{
  common::{create_swc_source_map, Source},
  swc_ecma_visit::VisitWith,
};

use crate::resource_pot_to_bundle::{
  defined_idents_collector::DefinedIdentsCollector, uniq_name::BundleVariable, Var,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum StmtAction {
  StripExport(usize),
  StripDefaultExport(usize, usize),
  StripImport(usize),
  DeclDefaultExpr(usize, usize),
  RemoveImport(usize),
  RemoveExport(usize),
  PatchNamespaceDecl(usize),
}

impl StmtAction {
  pub fn index(&self) -> Option<usize> {
    match self {
      StmtAction::StripExport(index) => Some(*index),
      StmtAction::StripDefaultExport(index, _) => Some(*index),
      StmtAction::StripImport(index) => Some(*index),
      StmtAction::DeclDefaultExpr(index, _) => Some(*index),
      StmtAction::RemoveImport(index) => Some(*index),
      StmtAction::RemoveExport(index) => Some(*index),
      StmtAction::PatchNamespaceDecl(_) => None,
    }
  }
}

pub type StatementId = usize;

#[derive(Debug, Clone)]
// export { foo as bar }; Variable(foo, Some(bar))
// import { foo as bar }; Variable(bar, Some(foo))
pub struct Variable(pub usize, pub Option<usize>);

impl From<usize> for Variable {
  fn from(value: usize) -> Self {
    Variable(value, None)
  }
}

impl From<(usize, Option<usize>)> for Variable {
  fn from(value: (usize, Option<usize>)) -> Self {
    Variable(value.0, value.1)
  }
}

impl Variable {
  pub fn export_as(&self) -> usize {
    self.1.unwrap_or(self.0)
  }

  pub fn import_origin(&self) -> usize {
    self.1.unwrap_or(self.0)
  }

  pub fn local(&self) -> usize {
    self.0
  }

  pub fn rev(&self) -> Self {
    if let Some(b) = self.1 {
      Variable(b, Some(self.0))
    } else {
      Variable(self.0, None)
    }
  }
}

#[derive(Debug, Clone)]
pub struct ImportInfo {
  pub source: ModuleId,
  pub specifiers: Vec<ImportSpecifierInfo>,
  pub stmt_id: StatementId,
}

// collect all exports and gathering them into a simpler structure
#[derive(Debug, Clone)]
pub enum ExportSpecifierInfo {
  /// ```js
  /// export * from 'foo';
  /// ```
  All(Option<Vec<usize>>),
  /// ```js
  /// // (default, Some(zoo))
  /// export { foo, bar, default as zoo } from 'foo';
  ///
  /// export { foo, bar };
  ///
  /// export const foo = 'foo';
  /// ```
  Named(Variable),
  /// ```js
  /// export default n;
  /// export default 1 + 1;
  /// ```
  Default(usize),
  /// ```js
  /// export * as foo from 'foo';
  /// ```
  Namespace(usize),
}

#[derive(Debug, Clone)]
pub struct ExportInfo {
  pub source: Option<ModuleId>,
  pub specifiers: Vec<ExportSpecifierInfo>,
  pub stmt_id: StatementId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportSpecifierInfo {
  /// ```js
  /// import * as foo from 'foo';
  /// ```
  Namespace(usize),
  /// ```js
  /// // local bar
  /// // imported Some(foo)
  /// import { foo as bar } from 'foo';
  ///
  /// // local foo
  /// // imported None
  /// import { foo } from 'foo';
  /// ```
  Named {
    local: usize,
    /// as foo
    imported: Option<usize>,
  },
  /// ```js
  /// import xxx from 'foo';
  /// ```
  Default(usize),
}

#[derive(Debug, Clone)]
pub struct Statement {
  pub id: StatementId,
  pub import: Option<ImportInfo>,
  pub export: Option<ExportInfo>,
  pub defined: Vec<usize>,
}

pub struct ModuleAnalyzer {
  pub statements: Vec<Statement>,
  pub statement_actions: HashSet<StmtAction>,
  pub cm: Arc<SourceMap>,
  pub ast: EcmaAstModule,
  pub module_id: ModuleId,
  pub resource_pot_id: ResourcePotId,
  pub export_names: Option<Vec<(ExportInfo, ModuleId)>>,
  pub entry: bool,
  pub external: bool,
  pub dynamic: bool,
  pub is_runtime: bool,
}

impl ModuleAnalyzer {
  pub fn new(
    module: &Module,
    context: &Arc<CompilationContext>,
    resource_pot_id: ResourcePotId,
    is_entry: bool,
    is_dynamic: bool,
    is_runtime: bool,
  ) -> Result<Self> {
    let ast = module.meta.as_script().ast.clone();

    let (cm, _) = create_swc_source_map(Source {
      path: PathBuf::from(module.id.resolved_path_with_query(&context.config.root)),
      content: module.content.clone(),
    });

    Ok(Self {
      statements: vec![],
      statement_actions: HashSet::new(),
      cm,
      ast,
      module_id: module.id.clone(),
      export_names: None,
      resource_pot_id,
      external: module.external,
      entry: is_entry,
      dynamic: is_dynamic,
      is_runtime,
    })
  }

  pub fn extract_statement(
    &mut self,
    module_graph: &ModuleGraph,
    bundle_variable: &mut RefMut<BundleVariable>,
  ) -> Result<()> {
    for (statement_id, stmt) in self.ast.body.iter().enumerate() {
      let statement = analyze_imports_and_exports(
        statement_id,
        stmt,
        &self.module_id,
        module_graph,
        &mut |ident, strict| bundle_variable.register_var(&self.module_id, ident, strict),
      )?;

      if statement.export.is_none() && statement.import.is_none() && statement.defined.is_empty() {
        continue;
      }

      self.statements.push(statement);
    }

    Ok(())
  }

  pub fn exports_stmts(&self) -> Vec<&ExportInfo> {
    self
      .statements
      .iter()
      .filter_map(|stmt| stmt.export.as_ref())
      .collect()
  }

  pub fn variables(&self) -> HashSet<usize> {
    let mut variables = HashSet::new();

    for statement in &self.statements {
      for defined in &statement.defined {
        variables.insert(*defined);
      }
    }

    variables
  }

  pub fn namespace_importers(&self) -> Vec<(ModuleId, usize)> {
    self
      .statements
      .iter()
      .filter_map(|stmt| {
        if let Some(import) = &stmt.import {
          if let Some(ImportSpecifierInfo::Namespace(namespace_import)) = import
            .specifiers
            .iter()
            .find(|specify| matches!(specify, ImportSpecifierInfo::Namespace(_)))
          {
            return Some((import.source.clone(), *namespace_import));
          }
        }

        if let Some(export) = &stmt.export {
          if let Some(ExportSpecifierInfo::Namespace(namespace_export)) = export
            .specifiers
            .iter()
            .find(|specify| matches!(specify, ExportSpecifierInfo::Namespace(_)))
          {
            return Some((
              export
                .source
                .as_ref()
                .expect("export namespace should have source")
                .clone(),
              *namespace_export,
            ));
          }
        }

        None
      })
      .collect()
  }

  pub fn build_rename_map<'a>(
    &self,
    bundle_variable: &'a BundleVariable,
  ) -> HashMap<&'a Id, &'a Var> {
    self
      .statements
      .iter()
      .flat_map(|statement| {
        statement
          .export
          .as_ref()
          .map(|item| {
            let mut idents: Vec<usize> = vec![];
            for specify in &item.specifiers {
              idents.extend(match specify {
                ExportSpecifierInfo::All(_) => {
                  vec![]
                }
                ExportSpecifierInfo::Named(var) => vec![var.local()],
                ExportSpecifierInfo::Default(index) => {
                  vec![*index]
                }
                ExportSpecifierInfo::Namespace(ns) => {
                  vec![*ns]
                }
              })
            }
            idents
          })
          .unwrap_or_default()
          .into_iter()
          .chain(statement.defined.iter().cloned())
          .chain(
            statement
              .import
              .as_ref()
              .map(|item| {
                let mut idents = vec![];
                for specify in &item.specifiers {
                  match specify {
                    ImportSpecifierInfo::Namespace(local) => {
                      idents.push(*local);
                    }
                    ImportSpecifierInfo::Named { local, imported: _ } => {
                      idents.push(*local);
                    }
                    ImportSpecifierInfo::Default(local) => {
                      idents.push(*local);
                    }
                  }
                }
                idents
              })
              .unwrap_or_default()
              .into_iter(),
          )
          .map(|item| bundle_variable.var_by_index(item))
          .filter(|item| item.rename.is_some())
          .map(|item| (&item.var, item))
      })
      .collect::<HashMap<_, _>>()
  }
}

pub fn analyze_imports_and_exports(
  id: StatementId,
  stmt: &ModuleItem,
  module_id: &ModuleId,
  module_graph: &ModuleGraph,
  register_var: &mut impl FnMut(&Ident, bool) -> usize,
) -> Result<Statement> {
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
            if let Some(ident) = &class_expr.ident {
              specify.push(ExportSpecifierInfo::Default(register_var(&ident, false)));
            }
          }

          swc_ecma_ast::DefaultDecl::Fn(fn_decl) => {
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
