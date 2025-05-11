use std::collections::{HashMap, HashSet};

use farmfe_core::{plugin::ResolveKind, swc_ecma_ast::Id};

use crate::module::{UsedExports, UsedExportsIdent};

use super::{
  analyze_used_import_all_fields::UsedImportAllFields, ExportInfo, ExportSpecifierInfo, ImportInfo,
  ImportSpecifierInfo, StatementId, UsedStatementIdent,
};

/// The result of tracing used import statements or export from statements
/// For example:
/// ```js
/// import * as foo from 'foo';
/// import { a, b, c as d } from 'foo';
/// import c from 'foo';
/// export * from 'foo';
/// export { a, b, c as d } from 'foo';
/// ```
/// The result should be:
/// ```ignore
/// [
///  TracedUsedImportStatement {
///   stmt_id: 0,
///   source: 'foo',
///   used_stmt_idents: [ExportAll],
///  },
///  TracedUsedImportStatement {
///   stmt_id: 1,
///   source: 'foo',
///   used_stmt_idents: [SwcIdent(a), SwcIdent(b), SwcIdent(c)],
///  },
///  TracedUsedImportStatement {
///   stmt_id: 2,
///   source: 'foo',
///   used_stmt_idents: [Default],
///  },
///  TracedUsedImportStatement {
///   stmt_id: 3,
///   source: 'foo',
///   used_stmt_idents: [ExportAll],
///  },
///  TracedUsedImportStatement {
///   stmt_id: 4,
///   source: 'foo',
///   used_stmt_idents: [SwcIdent(a), SwcIdent(b), SwcIdent(c)],
///  },
///
#[derive(Debug)]
pub struct TracedUsedImportStatement {
  pub stmt_id: StatementId,
  /// 'import * as foo from 'foo';' => foo
  pub source: String,
  pub used_stmt_idents: UsedExports,
  pub kind: ResolveKind,
}

impl TracedUsedImportStatement {
  pub fn from_import_info_and_used_idents(
    stmt_id: StatementId,
    import_info: &ImportInfo,
    used_defined_idents: &HashSet<UsedStatementIdent>,
    mut used_import_all_fields: HashMap<Id, HashSet<UsedImportAllFields>>,
  ) -> Self {
    let mut used_stmt_idents = HashSet::new();
    // for import, we only care about swc ident
    let used_defined_idents = used_defined_idents
      .iter()
      .filter_map(|i| match i {
        UsedStatementIdent::SwcIdent(i) => Some(i),
        _ => None,
      })
      .collect::<HashSet<_>>();

    for specifier in &import_info.specifiers {
      match specifier {
        ImportSpecifierInfo::Namespace(id) => {
          if used_defined_idents.contains(id) {
            // `import * as foo from './foo'` can be optimized when following conditions are met:
            // 1. xxx is used only by xxx.aa or xxx.['aa']
            if let Some(used_import_all_fields) = used_import_all_fields.remove(id) {
              // the conditions are met, so we can optimize it by setting used_stmt_idents to UsedExportsIdent::SwcIdent
              if !used_import_all_fields.contains(&UsedImportAllFields::All) {
                for used_import_all_field in used_import_all_fields {
                  match used_import_all_field {
                    UsedImportAllFields::All => unreachable!(),
                    UsedImportAllFields::Ident(field)
                    | UsedImportAllFields::LiteralComputed(field) => {
                      if &field == "default" {
                        used_stmt_idents.insert(UsedExportsIdent::Default);
                      } else {
                        used_stmt_idents.insert(UsedExportsIdent::SwcIdent(field));
                      }
                    }
                  }
                }
              } else {
                // the conditions are not met, so we should not optimize it
                used_stmt_idents.insert(UsedExportsIdent::ImportAll);
              }
            } else {
              used_stmt_idents.insert(UsedExportsIdent::ImportAll);
            }
          }
        }
        ImportSpecifierInfo::Named { local, imported } => {
          if used_defined_idents.contains(local) {
            if let Some(imported) = imported {
              if imported.0 == "default" {
                used_stmt_idents.insert(UsedExportsIdent::Default);
              } else {
                used_stmt_idents.insert(UsedExportsIdent::SwcIdent(imported.0.to_string()));
              }
            } else if used_defined_idents.contains(local) {
              used_stmt_idents.insert(UsedExportsIdent::SwcIdent(local.0.to_string()));
            }
          }
        }
        ImportSpecifierInfo::Default(id) => {
          if used_defined_idents.contains(id) {
            used_stmt_idents.insert(UsedExportsIdent::Default);
          }
        }
      }
    }

    Self {
      stmt_id,
      source: import_info.source.clone(),
      used_stmt_idents: UsedExports::Partial(used_stmt_idents),
      kind: ResolveKind::Import,
    }
  }

  pub fn from_export_info_and_used_idents(
    stmt_id: StatementId,
    export_info: &ExportInfo,
    used_defined_idents: &HashSet<UsedStatementIdent>,
  ) -> Option<Self> {
    if let Some(source) = &export_info.source {
      let mut used_stmt_idents = HashSet::new();

      for specifier in &export_info.specifiers {
        match specifier {
          ExportSpecifierInfo::Namespace(i) => {
            if used_defined_idents.contains(&UsedStatementIdent::SwcIdent(i.clone())) {
              used_stmt_idents.insert(UsedExportsIdent::ImportAll);
            }
          }
          ExportSpecifierInfo::Named { local, .. } => {
            if used_defined_idents.contains(&UsedStatementIdent::SwcIdent(local.clone())) {
              if local.0 == "default" {
                used_stmt_idents.insert(UsedExportsIdent::Default);
              } else {
                used_stmt_idents.insert(UsedExportsIdent::SwcIdent(local.0.to_string()));
              }
            }
          }
          ExportSpecifierInfo::Default => {
            unreachable!("export v from 'foo'; is not valid in ES module.");
          }
          ExportSpecifierInfo::All => {
            // for export all, if used statement idents contains ExportAll, we mark it as ExportAll
            if used_defined_idents.contains(&UsedStatementIdent::ExportAll) {
              used_stmt_idents.insert(UsedExportsIdent::ExportAll);
            } else {
              // else pass the may be used swc ident to the dependency module
              for used_ident in used_defined_idents {
                if let UsedStatementIdent::InExportAll(i) = used_ident {
                  used_stmt_idents.insert(UsedExportsIdent::SwcIdent(i.to_string()));
                }
              }
            }
          }
        }
      }

      return Some(Self {
        stmt_id,
        source: source.clone(),
        used_stmt_idents: UsedExports::Partial(used_stmt_idents),
        kind: ResolveKind::ExportFrom,
      });
    }

    None
  }
}
