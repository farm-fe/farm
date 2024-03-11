use std::collections::HashMap;

use farmfe_core::{
  module::{module_graph::ModuleGraph, ModuleId},
  swc_common::{Globals, Mark},
  swc_ecma_ast::{
    self, Ident, ImportDecl, ImportSpecifier, ModuleDecl, ModuleExportName, ModuleItem,
  },
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

pub fn is_global_ident(unresolved_mark: Mark, ident: &Ident) -> bool {
  ident.span.ctxt.outer() == unresolved_mark
}

pub fn remove_useless_stmts(
  tree_shake_module_id: &ModuleId,
  module_graph: &mut ModuleGraph,
  tree_shake_module_map: &HashMap<ModuleId, TreeShakeModule>,
  globals: &Globals,
) -> (
  Vec<ImportInfo>,
  Vec<ExportInfo>,
  Vec<ImportInfo>,
  Vec<ExportInfo>,
  Vec<ImportInfo>,
) {
  farmfe_core::farm_profile_function!(format!(
    "remove_useless_stmts {:?}",
    tree_shake_module_id.to_string()
  ));

  let tree_shake_module = tree_shake_module_map
    .get(tree_shake_module_id)
    .expect("tree shake module should exist");
  let module = module_graph.module_mut(tree_shake_module_id).unwrap();
  // analyze the statement graph start from the used statements
  let mut used_stmts = tree_shake_module
    .used_statements(module, globals)
    .into_iter()
    .collect::<Vec<_>>();
  let swc_module = &mut module.meta.as_script_mut().ast;
  // sort used_stmts
  used_stmts.sort_by_key(|a| a.0);

  let mut used_import_infos = vec![];
  let mut used_export_from_infos = vec![];
  let mut removed_import_info = vec![];
  let mut removed_export_info = vec![];

  let mut stmts_to_remove = vec![];

  // remove unused specifiers in export statement and import statement
  for (stmt_id, used_defined_idents) in &used_stmts {
    let module_item = &mut swc_module.body[*stmt_id];

    let (import_info, export_info, ..) =
      analyze_imports_and_exports(stmt_id, module_item, Some(used_defined_idents.clone()));

    if let Some(import_info) = import_info {
      if import_info.is_import_executed {
        used_import_infos.push(import_info.clone());
      } else {
        if import_info.specifiers.is_empty() {
          stmts_to_remove.push(*stmt_id);
        } else {
          used_import_infos.push(import_info.clone());
        }

        let mut remover = UselessImportStmtsRemover {
          import_info,
          removed_specify: vec![],
        };

        module_item.visit_mut_with(&mut remover);

        if !remover.removed_specify.is_empty() {
          removed_import_info.push(ImportInfo {
            specifiers: remover.removed_specify,
            ..remover.import_info
          });
        }
      }
    }

    if let Some(mut export_info) = export_info {
      if export_info.specifiers.is_empty() {
        stmts_to_remove.push(*stmt_id);
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

        let mut remover = UselessExportStmtRemover {
          export_info,
          removed_export_info: vec![],
        };

        module_item.visit_mut_with(&mut remover);

        if !remover.removed_export_info.is_empty() {
          removed_export_info.push(ExportInfo {
            specifiers: remover.removed_export_info,
            ..remover.export_info
          });
        }
      }
    }
  }

  let used_stmts_indexes = used_stmts
    .iter()
    .map(|(index, _)| index)
    .collect::<Vec<_>>();

  // remove the unused statements from the module
  for (index, stmt) in swc_module.body.iter().enumerate() {
    if !used_stmts_indexes.contains(&&index) {
      match stmt {
        ModuleItem::ModuleDecl(module_decl) => match module_decl {
          ModuleDecl::ExportNamed(export_named) => {
            let mut specifiers = vec![];

            for specifier in &export_named.specifiers {
              match specifier {
                swc_ecma_ast::ExportSpecifier::Named(named) => {
                  let local = match &named.orig {
                    ModuleExportName::Ident(i) => i.clone(),
                    ModuleExportName::Str(_) => {
                      unimplemented!("exporting a string is not supported")
                    }
                  };

                  specifiers.push(ExportSpecifierInfo::Named {
                    local: local.to_string(),
                    exported: named.exported.as_ref().map(|i| match i {
                      ModuleExportName::Ident(i) => i.to_string(),
                      _ => panic!("non-ident exported is not supported when tree shaking"),
                    }),
                  });
                }
                swc_ecma_ast::ExportSpecifier::Default(_) => {
                  unreachable!("ExportSpecifier::Default is not valid esm syntax")
                }
                swc_ecma_ast::ExportSpecifier::Namespace(ns) => {
                  let ident = match &ns.name {
                    ModuleExportName::Ident(ident) => ident.to_string(),
                    ModuleExportName::Str(_) => unreachable!("exporting a string is not supported"),
                  };

                  specifiers.push(ExportSpecifierInfo::Namespace(ident));
                }
              }
            }

            removed_export_info.push(ExportInfo {
              source: export_named.src.as_ref().map(|s| s.value.to_string()),
              specifiers,
              stmt_id: 0,
            });
          }

          ModuleDecl::ExportAll(export_all) => removed_export_info.push(ExportInfo {
            source: Some(export_all.src.value.to_string()),
            specifiers: vec![ExportSpecifierInfo::All(None)],
            stmt_id: 0,
          }),
          ModuleDecl::Import(import_info) => {
            let info = ImportInfo {
              source: import_info.src.value.to_string(),
              specifiers: import_info.specifiers.iter().map(|s| s.into()).collect(),
              stmt_id: 0,
              is_import_executed: import_info.specifiers.is_empty(),
            };
            if info.is_import_executed {
              used_import_infos.push(info);
              continue;
            } else {
              removed_import_info.push(info);
            }
          }
          _ => {}
        },
        _ => {}
      };

      stmts_to_remove.push(index);
    }
  }

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

  let mut wait_check_need_remove_import = vec![];
  // transform the import statement to import 'xxx'
  for (stmt_index, ty) in pending_transform_stmts {
    if matches!(ty, PendingTransformType::Import) {
      let module_item = &mut swc_module.body[stmt_index];

      if let ModuleItem::ModuleDecl(module_decl) = module_item {
        if let ModuleDecl::Import(import_decl) = module_decl {
          import_decl.specifiers.clear();
          wait_check_need_remove_import.push(ImportInfo {
            source: import_decl.src.value.to_string(),
            specifiers: vec![],
            stmt_id: 0,
            is_import_executed: false,
          })
        }
      }
    }
    // leave the export statement as it is
  }

  for index in stmts_to_remove {
    swc_module.body.remove(index);
  }

  (
    used_import_infos,
    used_export_from_infos,
    removed_import_info,
    removed_export_info,
    wait_check_need_remove_import,
  )
}

#[derive(Debug)]
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
          pending_transforms.push((
            stmt_index,
            PendingTransformType::Import,
            import_decl.src.value.to_string(),
          ));
        }
        ModuleDecl::ExportNamed(_) => {
          filtered_stmts_to_remove.push(stmt_index);
        }
        ModuleDecl::ExportAll(_) => {
          filtered_stmts_to_remove.push(stmt_index);
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
        if tree_shake_module.side_effects
          || tree_shake_module.is_self_executed_import
          || tree_shake_module.contains_self_executed_stmt
        {
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
  removed_specify: Vec<ImportSpecifierInfo>,
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
                named_specifier.local.to_string() == *local
              }
              _ => false,
            })
          {
            specifiers_to_remove.push(index);

            self.removed_specify.push(specifier.into());
          }
        }
        ImportSpecifier::Default(default) => {
          if !self
            .import_info
            .specifiers
            .iter()
            .any(|specifier| match specifier {
              ImportSpecifierInfo::Default(d) => default.local.to_string() == *d,
              _ => false,
            })
          {
            specifiers_to_remove.push(index);

            self.removed_specify.push(specifier.into());
          }
        }
        ImportSpecifier::Namespace(ns) => {
          if !self
            .import_info
            .specifiers
            .iter()
            .any(|specifier| match specifier {
              ImportSpecifierInfo::Namespace(n) => ns.local.to_string() == *n,
              _ => false,
            })
          {
            specifiers_to_remove.push(index);

            self.removed_specify.push(specifier.into());
          }
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
  removed_export_info: Vec<ExportSpecifierInfo>,
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
        self.removed_export_info.push(match specifier {
          swc_ecma_ast::ExportSpecifier::Namespace(ns) => match &ns.name {
            ModuleExportName::Ident(ident) => ExportSpecifierInfo::Namespace(ident.to_string()),
            ModuleExportName::Str(_) => unreachable!("exporting a string is not supported"),
          },
          swc_ecma_ast::ExportSpecifier::Default(_) => {
            unreachable!("ExportSpecifier::Default is not valid esm syntax")
          }
          swc_ecma_ast::ExportSpecifier::Named(named) => {
            let local = match &named.orig {
              ModuleExportName::Ident(i) => i.clone(),
              ModuleExportName::Str(_) => unimplemented!("exporting a string is not supported"),
            };
            ExportSpecifierInfo::Named {
              local: local.to_string(),
              exported: named.exported.as_ref().map(|i| match i {
                ModuleExportName::Ident(i) => i.to_string(),
                _ => panic!("non-ident exported is not supported when tree shaking"),
              }),
            }
          }
        });
      }
    }

    specifiers_to_remove.reverse();

    for index in specifiers_to_remove {
      specifiers.remove(index);
    }
  }
}
