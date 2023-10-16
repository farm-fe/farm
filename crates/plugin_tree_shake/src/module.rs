use farmfe_core::{
  hashbrown::{HashMap, HashSet},
  module::{Module, ModuleId, ModuleSystem},
};

use crate::statement_graph::{
  ExportInfo, ExportSpecifierInfo, ImportInfo, StatementGraph, StatementId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UsedIdent {
  /// Local ident
  SwcIdent(String),
  /// Default ident
  Default,
  /// This ident is used and may be exported from other module
  InExportAll(String),
  /// All idents is used and may be exported from other module
  ExportAll,
}

impl ToString for UsedIdent {
  fn to_string(&self) -> String {
    match self {
      UsedIdent::SwcIdent(ident) => ident.to_string(),
      UsedIdent::Default => "default".to_string(),
      UsedIdent::InExportAll(ident) => ident.to_string(),
      UsedIdent::ExportAll => "*".to_string(),
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
  pub contains_self_executed_stmt: bool,
  // used exports will be analyzed when tree shaking
  pub used_exports: UsedExports,
  pub module_system: ModuleSystem,
}

impl TreeShakeModule {
  pub fn new(module: &Module) -> Self {
    farmfe_core::farm_profile_function!(format!(
      "TreeShakeModule::new {:?}",
      module.id.to_string()
    ));
    let module_system = module.meta.as_script().module_system.clone();

    // 1. generate statement graph
    let ast = &module.meta.as_script().ast;
    let stmt_graph = if module_system == ModuleSystem::EsModule {
      StatementGraph::new(ast)
    } else {
      StatementGraph::empty()
    };

    // 2. set default used exports
    let used_exports = if module.side_effects {
      UsedExports::All
    } else {
      UsedExports::Partial(vec![])
    };

    Self {
      module_id: module.id.clone(),
      contains_self_executed_stmt: stmt_graph.contains_self_executed_stmt(),
      stmt_graph,
      used_exports,
      side_effects: module.side_effects,
      module_system,
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

    exports
  }

  pub fn used_statements(&self) -> HashMap<StatementId, HashSet<String>> {
    farmfe_core::farm_profile_function!(format!(
      "used_statements {:?}",
      self.module_id.to_string()
    ));
    // 1. get used exports
    let used_exports_idents = self.used_exports_idents();
    let mut stmt_used_idents_map = HashMap::new();

    for (used_ident, stmt_id) in used_exports_idents {
      let used_idents = stmt_used_idents_map
        .entry(stmt_id)
        .or_insert(HashSet::new());
      used_idents.insert(used_ident);
    }

    {
      farmfe_core::farm_profile_scope!(format!(
        "analyze self executed stmts {:?}",
        self.module_id.to_string()
      ));
      for stmt in self.stmt_graph.stmts() {
        if stmt.is_self_executed {
          stmt_used_idents_map
            .entry(stmt.id)
            .or_insert(HashSet::new());

          let dep_stmts = self.stmt_graph.dependencies(&stmt.id);

          for (dep_stmt, referred_idents) in dep_stmts {
            let used_idents = stmt_used_idents_map
              .entry(dep_stmt.id)
              .or_insert(HashSet::new());
            used_idents.extend(referred_idents.into_iter().map(UsedIdent::SwcIdent));
          }
        }
      }
    }

    // 2. analyze used statements starting from used exports

    self
      .stmt_graph
      .analyze_used_statements_and_idents(stmt_used_idents_map)
  }

  pub fn used_exports_idents(&self) -> Vec<(UsedIdent, StatementId)> {
    farmfe_core::farm_profile_function!(format!(
      "used_exports_idents {:?}",
      self.module_id.to_string()
    ));
    match &self.used_exports {
      UsedExports::All => {
        // all exported identifiers are used
        let mut used_idents = vec![];

        for export_info in self.exports() {
          for sp in export_info.specifiers {
            match sp {
              ExportSpecifierInfo::Default => {
                used_idents.push((UsedIdent::Default, export_info.stmt_id));
              }
              ExportSpecifierInfo::Named { local, .. } => {
                used_idents.push((UsedIdent::SwcIdent(local.clone()), export_info.stmt_id));
              }
              ExportSpecifierInfo::Namespace(ns) => {
                used_idents.push((UsedIdent::SwcIdent(ns.clone()), export_info.stmt_id));
              }
              ExportSpecifierInfo::All(_) => {
                used_idents.push((UsedIdent::ExportAll, export_info.stmt_id));
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
                let exported_ident = if let Some(exported) = exported {
                  exported
                } else {
                  local
                };

                is_ident_equal(ident, exported_ident)
              }
              ExportSpecifierInfo::Namespace(ns) => is_ident_equal(ident, ns),
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
                    if is_ident_equal(ident, &exported) {
                      used_idents.push((UsedIdent::SwcIdent(local.clone()), export_info.stmt_id));
                    }
                  } else if is_ident_equal(ident, &local) {
                    used_idents.push((UsedIdent::SwcIdent(local.clone()), export_info.stmt_id));
                  }
                }
                ExportSpecifierInfo::Namespace(ns) => {
                  if is_ident_equal(ident, &ns) {
                    used_idents.push((UsedIdent::SwcIdent(ns.clone()), export_info.stmt_id));
                  }
                }
                ExportSpecifierInfo::All(_) => unreachable!(),
              }
            }
          } else {
            // if export info is not found, and there are ExportSpecifierInfo::All, then the ident may be exported by `export * from 'xxx'`
            for export_info in self.exports() {
              if export_info
                .specifiers
                .iter()
                .any(|sp| matches!(sp, ExportSpecifierInfo::All(_)))
              {
                let stmt_id = export_info.stmt_id;
                used_idents.push((UsedIdent::InExportAll(ident.to_string()), stmt_id));
              }
            }
          }
        }

        used_idents
      }
    }
  }
}

fn is_ident_equal(ident1: &String, ident2: &String) -> bool {
  let split1 = ident1.split('#').collect::<Vec<_>>();
  let split2 = ident2.split('#').collect::<Vec<_>>();

  if split1.len() == 2 && split2.len() == 2 {
    split1[0] == split2[0] && split1[1] == split2[1]
  } else {
    split1[0] == split2[0]
  }
}
