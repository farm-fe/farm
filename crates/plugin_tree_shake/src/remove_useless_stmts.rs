use std::collections::HashMap;

use farmfe_core::{
  module::{module_graph::ModuleGraph, ModuleId},
  swc_ecma_ast::{ImportDecl, ImportSpecifier, ModuleDecl, ModuleExportName, ModuleItem},
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith, VisitWith};

use crate::{
  module::TreeShakeModule,
  statement_graph::{
    analyze_imports_and_exports::analyze_imports_and_exports,
    defined_idents_collector::DefinedIdentsCollector, ExportInfo, ExportSpecifierInfo, ImportInfo,
    ImportSpecifierInfo,
  },
};

pub fn remove_useless_stmts(
  tree_shake_module_id: &ModuleId,
  module_graph: &mut ModuleGraph,
  tree_shake_module_map: &HashMap<ModuleId, TreeShakeModule>,
) -> (Vec<ImportInfo>, Vec<ExportInfo>) {
  farmfe_core::farm_profile_function!(format!(
    "remove_useless_stmts {:?}",
    tree_shake_module.module_id.to_string()
  ));
  let tree_shake_module = tree_shake_module_map
    .get(tree_shake_module_id)
    .expect("tree shake module should exist");
  let module = module_graph.module_mut(tree_shake_module_id).unwrap();
  let swc_module = &mut module.meta.as_script_mut().ast;
  // analyze the statement graph start from the used statements
  let mut used_stmts = tree_shake_module
    .used_statements()
    .into_iter()
    .collect::<Vec<_>>();
  // sort used_stmts
  used_stmts.sort_by_key(|a| a.0);

  let mut used_import_infos = vec![];
  let mut used_export_from_infos = vec![];

  // remove unused specifiers in export statement and import statement
  for (stmt_id, used_defined_idents) in &used_stmts {
    let module_item = &mut swc_module.body[*stmt_id];

    let (import_info, export_info, ..) =
      analyze_imports_and_exports(stmt_id, module_item, Some(used_defined_idents.clone()));

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
        export_info.specifiers[0] =
          ExportSpecifierInfo::All(Some(used_defined_idents.clone().into_iter().collect()));
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

  drop(module);
  // remove from the end to the start
  stmts_to_remove.reverse();
  let (stmts_to_remove, pending_transform_stmts) = filter_stmts_to_remove(
    stmts_to_remove,
    tree_shake_module_id,
    module_graph,
    tree_shake_module_map,
  );

  let module = module_graph.module_mut(tree_shake_module_id).unwrap();
  let swc_module = &mut module.meta.as_script_mut().ast;

  // transform the import statement to import 'xxx'
  for (stmt_index, ty) in pending_transform_stmts {
    if matches!(ty, PendingTransformType::Import) {
      let module_item = &mut swc_module.body[stmt_index];

      if let ModuleItem::ModuleDecl(module_decl) = module_item {
        if let ModuleDecl::Import(import_decl) = module_decl {
          import_decl.specifiers.clear();
        }
      }
    }
    // leave the export statement as it is
  }

  for index in stmts_to_remove {
    swc_module.body.remove(index);
  }

  (used_import_infos, used_export_from_infos)
}

enum PendingTransformType {
  Import,
  ExportFrom,
  ExportAll,
}

fn filter_stmts_to_remove(
  stmts_to_remove: Vec<usize>,
  tree_shake_module_id: &ModuleId,
  module_graph: &ModuleGraph,
  tree_shake_module_map: &HashMap<ModuleId, TreeShakeModule>,
) -> (Vec<usize>, Vec<(usize, PendingTransformType)>) {
  let mut filtered_stmts_to_remove = vec![];
  let mut pending_transforms = vec![];

  let module = module_graph.module(tree_shake_module_id).unwrap();
  let swc_module = &module.meta.as_script().ast;

  for stmt_index in stmts_to_remove {
    // fix https://github.com/farm-fe/farm/issues/625
    // 1. if statement is `import xxx from 'dep' or `import { xxx } from 'dep'`, and dep has side effects or contains self-executed statements. it should not be removed, and it should be transformed to `import 'dep'`
    // 2. if statement is `export xxx from 'dep'`, and dep has side effects or contains self-executed statements. it should not be removed.

    match &swc_module.body[stmt_index] {
      ModuleItem::ModuleDecl(module_decl) => match module_decl {
        ModuleDecl::Import(import_decl) => {
          let source = import_decl.src.value.to_string();
          pending_transforms.push((stmt_index, PendingTransformType::Import, source));
        }
        ModuleDecl::ExportNamed(named) => {
          if let Some(source) = &named.src {
            let source = source.value.to_string();
            pending_transforms.push((stmt_index, PendingTransformType::ExportFrom, source));
          } else {
            filtered_stmts_to_remove.push(stmt_index);
          }
        }
        ModuleDecl::ExportAll(export_all) => {
          let source = export_all.src.value.to_string();
          pending_transforms.push((stmt_index, PendingTransformType::ExportAll, source));
        }
        _ => {
          filtered_stmts_to_remove.push(stmt_index);
        }
      },
      ModuleItem::Stmt(_) => {
        filtered_stmts_to_remove.push(stmt_index);
      }
    }
  }

  let mut pending_transform_stmts = vec![];
  // determine whether to remove the statement
  // if the dep has side effects or contains self-executed statements, it should not be removed
  for (stmt_index, ty, source) in pending_transforms {
    let dep_module_id = module_graph.get_dep_by_source_optional(tree_shake_module_id, &source);

    if let Some(dep_module_id) = dep_module_id {
      if let Some(tree_shake_module) = tree_shake_module_map.get(&dep_module_id) {
        if tree_shake_module.side_effects || tree_shake_module.contains_self_executed_stmt {
          match ty {
            PendingTransformType::Import => {
              pending_transform_stmts.push((stmt_index, PendingTransformType::Import));
            }
            PendingTransformType::ExportFrom | PendingTransformType::ExportAll => { /* do nothing, just preserve this statement */
            }
          }
        } else {
          filtered_stmts_to_remove.push(stmt_index);
        }
      } else {
        filtered_stmts_to_remove.push(stmt_index);
      }
    } else {
      filtered_stmts_to_remove.push(stmt_index);
    }
  }
  // make sure the stmts_to_remove is sorted from the end to the start
  filtered_stmts_to_remove.sort();
  filtered_stmts_to_remove.reverse();

  (filtered_stmts_to_remove, pending_transform_stmts)
}

pub struct UselessImportStmtsRemover {
  import_info: ImportInfo,
}

impl VisitMut for UselessImportStmtsRemover {
  fn visit_mut_import_decl(&mut self, import_decl: &mut ImportDecl) {
    let mut specifiers_to_remove = vec![];

    for (index, specifier) in import_decl.specifiers.iter().enumerate() {
      if let ImportSpecifier::Named(named_specifier) = specifier {
        if !self
          .import_info
          .specifiers
          .iter()
          .any(|specifier| match specifier {
            ImportSpecifierInfo::Named { local, .. } => named_specifier.local.to_string() == *local,
            _ => false,
          })
        {
          specifiers_to_remove.push(index);
        }
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

        for (index, decl) in var_decl.decls.iter_mut().enumerate() {
          if !self
            .export_info
            .specifiers
            .iter()
            .any(|export_specifier| match export_specifier {
              ExportSpecifierInfo::Named { local, .. } => {
                let mut defined_idents_collector = DefinedIdentsCollector::new();
                decl.name.visit_with(&mut defined_idents_collector);

                defined_idents_collector.defined_idents.contains(local)
              }
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
                ModuleExportName::Ident(ident) => ident.to_string() == *local,
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
