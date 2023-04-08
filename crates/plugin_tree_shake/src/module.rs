use farmfe_core::{
  hashbrown::HashMap,
  module::{Module, ModuleId, ModuleSystem},
  swc_ecma_ast::Ident,
};

use crate::statement_graph::{
  ExportInfo, ExportSpecifierInfo, ImportInfo, StatementGraph, StatementId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsedIdent {
  /// Local ident
  SwcIdent(Ident),
  /// Default ident
  Default,
  /// This ident is used and may be exported from other module
  InExportAll(String),
}

impl ToString for UsedIdent {
  fn to_string(&self) -> String {
    match self {
      UsedIdent::SwcIdent(ident) => ident.sym.to_string(),
      UsedIdent::Default => "default".to_string(),
      UsedIdent::InExportAll(ident) => ident.to_string(),
    }
  }
}

#[derive(Debug, Clone)]
pub enum UsedExports {
  All,
  Partial(Vec<String>),
}

impl UsedExports {
  pub fn add_used_export(&mut self, used_export: &dyn ToString) {
    match self {
      UsedExports::All => {
        *self = UsedExports::All;
      }
      UsedExports::Partial(self_used_exports) => self_used_exports.push(used_export.to_string()),
    }
  }

  pub fn is_empty(&self) -> bool {
    match self {
      UsedExports::All => false,
      UsedExports::Partial(self_used_exports) => self_used_exports.is_empty(),
    }
  }
}

pub struct TreeShakeModule {
  pub module_id: ModuleId,
  pub side_effects: bool,
  pub stmt_graph: StatementGraph,
  // used exports will be analyzed when tree shaking
  pub used_exports: UsedExports,
  pub module_system: ModuleSystem,
}

impl TreeShakeModule {
  pub fn new(module: &Module) -> Self {
    // 1. generate statement graph
    let ast = &module.meta.as_script().ast;
    let stmt_graph = StatementGraph::new(ast);

    // 2. set default used exports
    let used_exports = if module.side_effects {
      UsedExports::All
    } else {
      UsedExports::Partial(vec![])
    };

    Self {
      module_id: module.id.clone(),
      stmt_graph,
      used_exports,
      side_effects: module.side_effects,
      module_system: module.meta.as_script().module_system.clone(),
    }
  }

  pub fn imports(&self) -> Vec<ImportInfo> {
    let mut imports = vec![];

    for stmt in self.stmt_graph.stmts() {
      if let Some(import) = &stmt.import_info {
        imports.push(import.clone());
      }
    }

    imports
  }

  pub fn exports(&self) -> Vec<ExportInfo> {
    let mut exports = vec![];

    for stmt in self.stmt_graph.stmts() {
      if let Some(export) = &stmt.export_info {
        exports.push(export.clone());
      }
    }

    return exports;
  }

  pub fn used_statements(&self) -> Vec<(StatementId, Vec<String>)> {
    // 1. get used exports
    let used_exports_idents = self.used_exports_idents();
    let mut stmt_used_idents_map = HashMap::new();

    for (used_ident, stmt_id) in used_exports_idents {
      let used_idents = stmt_used_idents_map.entry(stmt_id).or_insert(vec![]);
      used_idents.push(used_ident);
    }

    // 2. analyze used statements starting from used exports
    let used_statements = self
      .stmt_graph
      .analyze_used_statements_and_idents(stmt_used_idents_map);

    return used_statements;
  }

  pub fn used_exports_idents(&self) -> Vec<(UsedIdent, StatementId)> {
    match &self.used_exports {
      UsedExports::All => {
        // all exported identifiers are used
        let mut used_idents = vec![];

        for export_info in self.exports() {
          // only deal with local exports instead of re-exports
          if export_info.source.is_none() {
            for sp in export_info.specifiers {
              match sp {
                ExportSpecifierInfo::Default => {
                  used_idents.push((UsedIdent::Default, export_info.stmt_id));
                }
                ExportSpecifierInfo::Named { local, .. } => {
                  used_idents.push((UsedIdent::SwcIdent(local.clone()), export_info.stmt_id));
                }
                ExportSpecifierInfo::All(_) | ExportSpecifierInfo::Namespace(_) => unreachable!(),
              }
            }
          }
        }

        used_idents
      }
      UsedExports::Partial(idents) => {
        let mut used_idents = vec![];

        for ident in idents {
          // find the export info that contains the ident
          let export_info = self.exports().into_iter().find(|export_info| {
            export_info.specifiers.iter().any(|sp| match sp {
              ExportSpecifierInfo::Default => ident == "default",
              ExportSpecifierInfo::Named { local, exported } => {
                if let Some(exported) = exported {
                  ident == &exported.sym.to_string()
                } else {
                  ident == &local.sym.to_string()
                }
              }
              ExportSpecifierInfo::Namespace(ns) => ident == &ns.sym.to_string(),
              ExportSpecifierInfo::All(_) => {
                /* Deal with All later */
                false
              }
            })
          });

          if let Some(export_info) = export_info {
            for sp in export_info.specifiers {
              match sp {
                ExportSpecifierInfo::Default => {
                  if ident == "default" {
                    used_idents.push((UsedIdent::Default, export_info.stmt_id));
                  }
                }
                ExportSpecifierInfo::Named { local, exported } => {
                  if let Some(exported) = exported {
                    if ident == &exported.sym.to_string() {
                      used_idents.push((UsedIdent::SwcIdent(local.clone()), export_info.stmt_id));
                    }
                  } else {
                    if ident == &local.sym.to_string() {
                      used_idents.push((UsedIdent::SwcIdent(local.clone()), export_info.stmt_id));
                    }
                  }
                }
                ExportSpecifierInfo::Namespace(ns) => {
                  if ident == &ns.sym.to_string() {
                    used_idents.push((UsedIdent::SwcIdent(ns.clone()), export_info.stmt_id));
                  }
                }
                ExportSpecifierInfo::All(_) => unreachable!(),
              }
            }
          } else {
            // if export info is not found, and there are ExportSpecifierInfo::All, then the ident may be exported by `export * from 'xxx'`
            // otherwise, the ident is not exported and we should throw a warning
            let mut found = false;

            for export_info in self.exports() {
              if export_info.specifiers.iter().any(|sp| match sp {
                ExportSpecifierInfo::All(_) => true,
                _ => false,
              }) {
                found = true;
                let stmt_id = export_info.stmt_id;
                used_idents.push((UsedIdent::InExportAll(ident.to_string()), stmt_id));
              }
            }

            if !found {
              println!(
                "[Farm Tree Shake] Warning: {} is not exported by {:?}",
                ident, self.module_id
              );
            }
          }
        }

        // // remove unused export specifiers in export info
        // for export_info in self.exports() {
        //   let stmt = self.stmt_graph.stmt_mut(&export_info.stmt_id);
        //   // remove unused export specifiers in export info
        //   let mut new_specifiers = vec![];
        //   let original_export_info = stmt.export_info.as_mut().unwrap();

        //   for sp in &original_export_info.specifiers {
        //     match sp {
        //       ExportSpecifierInfo::Default => {
        //         if idents.contains(&"default".to_string()) {
        //           new_specifiers.push(ExportSpecifierInfo::Default);
        //         }
        //       }
        //       ExportSpecifierInfo::Named { local, exported } => {
        //         if let Some(exported) = exported {
        //           if idents.contains(&exported.sym.to_string()) {
        //             new_specifiers.push(ExportSpecifierInfo::Named {
        //               local: local.clone(),
        //               exported: Some(exported.clone()),
        //             });
        //           }
        //         } else if idents.contains(&local.sym.to_string()) {
        //           new_specifiers.push(ExportSpecifierInfo::Named {
        //             local: local.clone(),
        //             exported: None,
        //           });
        //         }
        //       }
        //       ExportSpecifierInfo::All => {
        //         new_specifiers.push(ExportSpecifierInfo::All);
        //       }
        //       ExportSpecifierInfo::Namespace(ns) => {
        //         new_specifiers.push(ExportSpecifierInfo::Namespace(ns.clone()));
        //       }
        //     }
        //   }

        //   original_export_info.specifiers = new_specifiers;
        // }

        used_idents
      }
    }
  }
}
