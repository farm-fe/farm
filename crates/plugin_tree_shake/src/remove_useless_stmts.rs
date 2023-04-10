use farmfe_core::swc_ecma_ast::{
  ImportDecl, ImportSpecifier, Module as SwcModule, ModuleExportName,
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::{
  module::TreeShakeModule,
  statement_graph::{
    analyze_imports_and_exports::analyze_imports_and_exports, ExportInfo, ExportSpecifierInfo,
    ImportInfo, ImportSpecifierInfo,
  },
};

pub fn remove_useless_stmts(
  tree_shake_module: &mut TreeShakeModule,
  swc_module: &mut SwcModule,
) -> (Vec<ImportInfo>, Vec<ExportInfo>) {
  // analyze the statement graph start from the used statements
  let mut used_stmts = tree_shake_module.used_statements();
  // sort used_stmts
  used_stmts.sort_by_key(|a| a.0);

  let mut used_import_infos = vec![];
  let mut used_export_from_infos = vec![];

  // remove unused specifiers in export statement and import statement
  for (stmt_id, used_defined_idents) in &used_stmts {
    let module_item = &mut swc_module.body[*stmt_id];

    let (import_info, export_info, ..) =
      analyze_imports_and_exports(&stmt_id, module_item, Some(used_defined_idents.clone()));

    if let Some(import_info) = import_info {
      used_import_infos.push(import_info.clone());

      let mut remover = UselessImportStmtsRemover { import_info };

      module_item.visit_mut_with(&mut remover);
    }

    if let Some(mut export_info) = export_info {
      if export_info.specifiers.is_empty() {
        continue;
      }

      // if this export statement is export * from 'xxx'
      if export_info.source.is_some()
        && matches!(export_info.specifiers[0], ExportSpecifierInfo::All(_))
      {
        export_info.specifiers[0] = ExportSpecifierInfo::All(Some(used_defined_idents.clone()));
        used_export_from_infos.push(export_info.clone());
      } else {
        if export_info.source.is_some() {
          used_export_from_infos.push(export_info.clone());
        }

        let mut remover = UselessExportStmtRemover { export_info };

        module_item.visit_mut_with(&mut remover);
      }
    }
  }

  let mut stmts_to_remove = vec![];
  // TODO recognize the self-executed statements and preserve all the related statements

  let used_stmts_indexes = used_stmts
    .iter()
    .map(|(index, _)| index)
    .collect::<Vec<_>>();

  // remove the unused statements from the module
  for (index, _) in swc_module.body.iter().enumerate() {
    if !used_stmts_indexes.contains(&&index) {
      stmts_to_remove.push(index);
    }
  }

  // remove from the end to the start
  stmts_to_remove.reverse();

  for stmt in stmts_to_remove {
    swc_module.body.remove(stmt);
  }

  (used_import_infos, used_export_from_infos)
}

pub struct UselessImportStmtsRemover {
  import_info: ImportInfo,
}

impl VisitMut for UselessImportStmtsRemover {
  fn visit_mut_import_decl(&mut self, import_decl: &mut ImportDecl) {
    let mut specifiers_to_remove = vec![];

    for (index, specifier) in import_decl.specifiers.iter().enumerate() {
      match specifier {
        ImportSpecifier::Named(named_specifier) => {
          if !self
            .import_info
            .specifiers
            .iter()
            .any(|specifier| match specifier {
              ImportSpecifierInfo::Named { local, .. } => {
                named_specifier.local.to_string() == local.to_string()
              }
              _ => false,
            })
          {
            specifiers_to_remove.push(index);
          }
        }
        _ => {}
      }
    }

    specifiers_to_remove.reverse();

    for index in specifiers_to_remove {
      import_decl.specifiers.remove(index);
    }
  }
}

pub struct UselessExportStmtRemover {
  export_info: ExportInfo,
}

impl VisitMut for UselessExportStmtRemover {
  fn visit_mut_export_decl(&mut self, export_decl: &mut farmfe_core::swc_ecma_ast::ExportDecl) {
    match &mut export_decl.decl {
      farmfe_core::swc_ecma_ast::Decl::Var(var_decl) => {
        let mut decls_to_remove = vec![];

        for (index, decl) in var_decl.decls.iter().enumerate() {
          if !self
            .export_info
            .specifiers
            .iter()
            .any(|export_specifier| match export_specifier {
              ExportSpecifierInfo::Named { local, .. } => match &decl.name {
                farmfe_core::swc_ecma_ast::Pat::Ident(ident) => {
                  ident.to_string() == local.to_string()
                }
                // TODO support other patterns
                _ => false,
              },
              _ => false,
            })
          {
            decls_to_remove.push(index);
          }
        }

        decls_to_remove.reverse();

        for index in decls_to_remove {
          var_decl.decls.remove(index);
        }
      }
      _ => {}
    }
  }

  fn visit_mut_export_specifiers(
    &mut self,
    specifiers: &mut Vec<farmfe_core::swc_ecma_ast::ExportSpecifier>,
  ) {
    let mut specifiers_to_remove = vec![];

    for (index, specifier) in specifiers.iter().enumerate() {
      if !self
        .export_info
        .specifiers
        .iter()
        .any(|export_specifier| match export_specifier {
          ExportSpecifierInfo::Named { local, .. } => match specifier {
            farmfe_core::swc_ecma_ast::ExportSpecifier::Named(named_specifier) => {
              match &named_specifier.orig {
                ModuleExportName::Ident(ident) => ident.to_string() == local.to_string(),
                _ => false,
              }
            }
            _ => false,
          },
          _ => false,
        })
      {
        specifiers_to_remove.push(index);
      }
    }

    specifiers_to_remove.reverse();

    for index in specifiers_to_remove {
      specifiers.remove(index);
    }
  }
}
