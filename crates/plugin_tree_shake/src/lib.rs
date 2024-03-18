#![feature(box_patterns)]
#![feature(exact_size_is_empty)]

use std::collections::{HashMap, HashSet, VecDeque};

use farmfe_core::{
  config::Config,
  module::{module_graph::ModuleGraph, ModuleId},
  plugin::Plugin,
  rayon::iter::{IntoParallelIterator, ParallelIterator},
};
use farmfe_toolkit::script::swc_try_with::resolve_module_mark;
use module::TreeShakeModule;
use statement_graph::{ExportInfo, ImportInfo};

use crate::remove_hot_update::remove_useless_hot_update_stmts;

pub mod module;
pub mod remove_hot_update;
pub mod remove_useless_stmts;
pub mod statement_graph;

pub struct FarmPluginTreeShake;

impl FarmPluginTreeShake {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

fn toposort(
  module_graph: &ModuleGraph,
  pre_shaking_module_map: &mut HashMap<ModuleId, TreeShakeModule>,
) -> (Vec<ModuleId>, Vec<ModuleId>) {
  fn dfs(
    entry: &ModuleId,
    graph: &ModuleGraph,
    stack: &mut Vec<ModuleId>,
    visited: &mut HashSet<ModuleId>,
    result: &mut Vec<ModuleId>,
    pre_shaking_module_map: &mut HashMap<ModuleId, TreeShakeModule>,
    cyclic_node: &mut Vec<ModuleId>,
  ) {
    // collect cycle
    if stack.iter().any(|m| m == entry) {
      // while see ahead of the stack, mark parse import specify
      let import_circle_module_id = stack.iter().last().unwrap();
      if let Some(import_circle_module) = pre_shaking_module_map.get_mut(&import_circle_module_id) {
        let imports = import_circle_module
          .imports()
          .into_iter()
          .filter(|import_info| {
            graph
              .get_dep_by_source_optional(import_circle_module_id, &import_info.source)
              .is_some()
          })
          .collect::<Vec<_>>();

        let shake_module = pre_shaking_module_map.get_mut(&entry).unwrap();

        for shake_module_import_info in imports {
          for specify in shake_module_import_info.specifiers {
            match specify {
              statement_graph::ImportSpecifierInfo::Namespace(_) => {
                shake_module
                  .used_exports
                  .add_used_export(import_circle_module_id, &module::UsedIdent::ExportAll);
              }

              statement_graph::ImportSpecifierInfo::Named { local, imported } => {
                if let Some(ident) = imported {
                  if ident.as_str() == "default" {
                    shake_module
                      .used_exports
                      .add_used_export(import_circle_module_id, &module::UsedIdent::Default);
                  } else {
                    shake_module.used_exports.add_used_export(
                      import_circle_module_id,
                      &module::UsedIdent::SwcIdent(strip_context(&ident)),
                    );
                  }
                } else {
                  shake_module.used_exports.add_used_export(
                    import_circle_module_id,
                    &module::UsedIdent::SwcIdent(strip_context(&local)),
                  );
                }
              }
              statement_graph::ImportSpecifierInfo::Default(_) => {
                shake_module
                  .used_exports
                  .add_used_export(import_circle_module_id, &module::UsedIdent::Default);
              }
            }
          }
        }
      };

      cyclic_node.push(entry.clone());
      return;
    } else if visited.contains(entry) {
      // skip visited module
      return;
    }

    visited.insert(entry.clone());
    stack.push(entry.clone());

    let deps = graph.dependencies(entry);

    for (dep, _) in &deps {
      dfs(
        dep,
        graph,
        stack,
        visited,
        result,
        pre_shaking_module_map,
        cyclic_node,
      )
    }

    // visit current entry
    result.push(stack.pop().unwrap());
  }

  let mut result = vec![];
  let mut stack = vec![];
  let mut cyclic_node = vec![];

  // sort entries to make sure it is stable
  let mut entries = module_graph.entries.iter().collect::<Vec<_>>();
  entries.sort();

  let mut visited = HashSet::new();

  for (entry, _) in entries {
    let mut res = vec![];
    dfs(
      entry,
      module_graph,
      &mut stack,
      &mut visited,
      &mut res,
      pre_shaking_module_map,
      &mut cyclic_node,
    );

    result.extend(res);
  }

  result.reverse();

  (result, cyclic_node)
}

#[derive(Debug)]
enum ShakeType {
  // stage1: normal traverse tree_shake_modules
  TopoShaking,
  // stage2: if tree_shake_module is circle_module, need to be processed multiple times
  CircleRemove(bool),
  // stage3: if tree_shake_module contain self executed module, it cannot be deleted immediately, need after it process, check it not contain side_effects
  CheckNeedRemoveImport(Vec<ImportInfo>),
}

impl Plugin for FarmPluginTreeShake {
  fn name(&self) -> &'static str {
    "FarmPluginTreeShake"
  }

  /// tree shake useless modules and code, steps:
  /// 1. topo sort the module_graph, the cyclic modules will be marked as side_effects
  /// 2. generate tree_shake_modules based on the topo sorted modules
  /// 3. traverse the tree_shake_modules
  ///   3.1 mark entry modules as side_effects
  ///   3.2 if module is commonjs, mark all imported modules as [UsedExports::All]
  ///   3.3 else if module is esm and the module has side effects, add imported identifiers to [UsedExports::Partial] of the imported modules
  ///   3.4 else if module is esm and the module has no side effects, analyze the used statement based on the statement graph
  fn optimize_module_graph(
    &self,
    module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let mut tree_shake_modules_map: HashMap<ModuleId, TreeShakeModule> =
      std::collections::HashMap::new();

    module_graph
      .modules_mut()
      .into_par_iter()
      .filter(|m| m.module_type.is_script() && !m.external)
      .for_each(|module| {
        let meta = module.meta.as_script_mut();

        if meta.top_level_mark != 0 || meta.unresolved_mark != 0 {
          return;
        }

        let ast = &mut meta.ast;

        let (unresolved_mark, top_level_mark) =
          resolve_module_mark(ast, module.module_type.is_typescript(), context);

        module.meta.as_script_mut().unresolved_mark = unresolved_mark.as_u32();
        module.meta.as_script_mut().top_level_mark = top_level_mark.as_u32();
      });

    module_graph.modules().iter().for_each(|module| {
      if !module.module_type.is_script() || module.external {
        return;
      }

      let tree_shake_module = module::TreeShakeModule::new(module);
      tree_shake_modules_map.insert(module.id.clone(), tree_shake_module);
    });

    // topo sort the module_graph, the cyclic modules will be marked as side_effects
    let (topo_sorted_modules, cyclic_nodes) = {
      farmfe_core::farm_profile_scope!("tree shake toposort".to_string());
      toposort(&module_graph, &mut tree_shake_modules_map)
    };

    // mark entry modules as side_effects
    for (entry_module_id, _) in module_graph.entries.clone() {
      let module = module_graph.module_mut(&entry_module_id).unwrap();
      module.side_effects = true;
    }

    let mut tree_shake_modules_ids = vec![];

    for module_id in topo_sorted_modules {
      let module = module_graph.module(&module_id).unwrap();

      // skip non script modules and external modules
      if !module.module_type.is_script() || module.external {
        if !module.module_type.is_script() && !module.external {
          // mark all non script modules' script dependencies as side_effects
          for dep_id in module_graph.dependencies_ids(&module_id) {
            let dep_module = module_graph.module_mut(&dep_id).unwrap();

            if !dep_module.module_type.is_script() {
              continue;
            }

            dep_module.side_effects = true;
          }
        }

        continue;
      }

      // let tree_shake_module = module::TreeShakeModule::new(module);
      tree_shake_modules_ids.push((module.id.clone(), ShakeType::TopoShaking));
      if let Some(shake_module) = tree_shake_modules_map.get_mut(&module.id) {
        shake_module.side_effects = module.side_effects;
        if shake_module.side_effects {
          shake_module.used_exports.change_all();
        }
      }
      // tree_shake_modules_map.insert(tree_shake_module.module_id.clone(), tree_shake_module);
    }

    let mut modules_to_remove = vec![];

    let mut tree_shake_modules_ids = VecDeque::from(tree_shake_modules_ids);

    // After it is processed, if it needs to be processed again, it needs to be re-parsed
    fn reanalyze_module(
      module_id: &ModuleId,
      module_graph: &ModuleGraph,
      tree_shake_modules_map: &mut HashMap<ModuleId, TreeShakeModule>,
    ) {
      let module = module_graph.module(&module_id).unwrap();
      let old_tree_shake_module = tree_shake_modules_map
        .get_mut(&module_id)
        .unwrap_or_else(|| {
          panic!("imported module not found: {:?}", module_id);
        });
      let mut new_tree_shake_module = TreeShakeModule::new(module);
      new_tree_shake_module.used_exports = old_tree_shake_module.used_exports.clone();
      tree_shake_modules_map.insert(module_id.clone(), new_tree_shake_module);
    }

    let mut wait_check_remove_import: HashSet<ModuleId> = HashSet::new();
    let mut visited_modules: HashSet<ModuleId> = HashSet::new();

    // traverse the tree_shake_modules
    while let Some((tree_shake_module_id, shake_type)) = tree_shake_modules_ids.pop_front() {
      let tree_shake_module = tree_shake_modules_map
        .get_mut(&tree_shake_module_id)
        .unwrap();

      // if module is not esm, mark all imported modules as [UsedExports::All]
      if !matches!(
        tree_shake_module.module_system,
        farmfe_core::module::ModuleSystem::EsModule
      ) {
        for (dep_id, _) in module_graph.dependencies(&tree_shake_module_id) {
          let dep_tree_shake_module = tree_shake_modules_map.get_mut(&dep_id);

          if let Some(dep_tree_shake_module) = dep_tree_shake_module {
            dep_tree_shake_module.used_exports.change_all();
          }
        }
      } else {
        // if module is esm and the module has side effects, add imported identifiers to [UsedExports::Partial] of the imported modules
        if tree_shake_module.side_effects {
          let imports = tree_shake_module.imports();
          let exports = tree_shake_module.exports();

          for import_info in &imports {
            add_used_exports_by_import_info(
              &mut tree_shake_modules_map,
              &*module_graph,
              &tree_shake_module_id,
              import_info,
            );
          }

          for export_info in &exports {
            add_used_exports_by_export_info(
              &mut tree_shake_modules_map,
              &*module_graph,
              &tree_shake_module_id,
              export_info,
            );
          }
        } else {
          let tree_shake_module = tree_shake_modules_map
            .get_mut(&tree_shake_module_id)
            .unwrap();

          if let ShakeType::CheckNeedRemoveImport(imports) = &shake_type {
            let mut wait_remove_imports = vec![];
            for import in tree_shake_module.imports() {
              if imports.iter().any(|item| {
                if item.source != import.source {
                  return false;
                }

                let module_id =
                  module_graph.get_dep_by_source(&tree_shake_module_id, &import.source);

                let import_tree_shake_module = tree_shake_modules_map.get(&module_id).unwrap();

                return !import_tree_shake_module.contains_self_executed_stmt
                  && import_tree_shake_module.stmt_graph.stmts().is_empty()
                  && import_tree_shake_module.used_exports.is_empty();
              }) {
                wait_check_remove_import.remove(&tree_shake_module_id);
                wait_remove_imports.push(import.stmt_id);
              }
            }

            wait_remove_imports.sort();
            wait_remove_imports.reverse();

            if !wait_remove_imports.is_empty() {
              let module = module_graph.module_mut(&tree_shake_module_id).unwrap();
              let swc_ast = &mut module.meta.as_script_mut().ast;

              for stmt_id in wait_remove_imports {
                swc_ast.body.remove(stmt_id);
              }

              reanalyze_module(
                &tree_shake_module_id,
                module_graph,
                &mut tree_shake_modules_map,
              );
            };

            continue;
          }

          let tree_shake_module = tree_shake_modules_map
            .get_mut(&tree_shake_module_id)
            .unwrap();

          if tree_shake_module.used_exports.is_empty()
            && !tree_shake_module.is_self_executed_import
            && !wait_check_remove_import.contains(&tree_shake_module_id)
          {
            // if the module's used_exports is empty, and this module does not have self-executed statements, then this module is useless
            // which means this module is not used and should be removed
            modules_to_remove.push(tree_shake_module_id.clone());
            continue;
          }

          let mut removed_exports = vec![];

          // remove useless statements and useless imports/exports identifiers, then all preserved import info and export info will be added to the used_exports.
          let (
            used_imports,
            used_exports_from,
            removed_imports,
            removed_export,
            check_need_remove_import,
          ) = remove_useless_stmts::remove_useless_stmts(
            &tree_shake_module_id,
            module_graph,
            &tree_shake_modules_map,
            &context.meta.script.globals,
          );

          if !matches!(shake_type, ShakeType::CheckNeedRemoveImport(_))
            && !wait_check_remove_import.contains(&tree_shake_module_id)
            && !check_need_remove_import.is_empty()
          {
            for import_info in &check_need_remove_import {
              if let Some(import_module_id) =
                module_graph.get_dep_by_source_optional(&tree_shake_module_id, &import_info.source)
              {
                wait_check_remove_import.insert(import_module_id);
              }
            }

            tree_shake_modules_ids.push_back((
              tree_shake_module_id.clone(),
              ShakeType::CheckNeedRemoveImport(check_need_remove_import),
            ));

            reanalyze_module(
              &tree_shake_module_id,
              module_graph,
              &mut tree_shake_modules_map,
            );
          }

          removed_exports.extend(removed_export);

          // cyclic_nodes affect already proceed module
          for import_info in removed_imports {
            let Some((module_id, delay_import_info)) = remove_used_exports_by_import_info(
              &mut tree_shake_modules_map,
              &*module_graph,
              &tree_shake_module_id,
              &import_info,
              &cyclic_nodes,
            ) else {
              continue;
            };

            // If still in the queue
            if !visited_modules.contains(&module_id) {
              continue;
            }

            reanalyze_module(&module_id, module_graph, &mut tree_shake_modules_map);

            let is_removed_import_namespace = delay_import_info.is_some();
            if let Some((_, delay)) = delay_import_info {
              let tree_shake_module = tree_shake_modules_map.get_mut(&module_id).unwrap();
              for specify in &delay.specifiers {
                match specify {
                  statement_graph::ImportSpecifierInfo::Namespace(_) => {
                    tree_shake_module
                      .used_exports
                      .remove_used_export(&tree_shake_module_id, &module::UsedIdent::ExportAll);
                  }
                  _ => {}
                }
              }
            }

            tree_shake_modules_ids.push_back((
              module_id,
              ShakeType::CircleRemove(is_removed_import_namespace),
            ));
          }

          // in cyclic module, when `import * as Foo from "foo"` is dead code, and after it removed, "foo" module used_export will be changed, import need reanalyze
          if matches!(shake_type, ShakeType::CircleRemove(true)) {
            for export_info in &used_exports_from {
              if let Some(source) = export_info.source.clone() {
                replace_used_export_by_export_info(
                  &mut tree_shake_modules_map,
                  &*module_graph,
                  &tree_shake_module_id,
                  &export_info,
                );

                let import_module_id =
                  module_graph.get_dep_by_source(&tree_shake_module_id, &source);
                reanalyze_module(&import_module_id, module_graph, &mut tree_shake_modules_map);
                tree_shake_modules_ids
                  .push_back((import_module_id, ShakeType::CircleRemove(false)));
              }
            }
          }

          // in circle remove process, because the module is remove export, it already proceed
          if matches!(shake_type, ShakeType::CircleRemove(_)) {
            for export_info in removed_exports {
              if let Some(module_id) = removed_used_exports_by_export_info(
                &mut tree_shake_modules_map,
                &*module_graph,
                &tree_shake_module_id,
                &export_info,
              ) {
                reanalyze_module(&module_id, module_graph, &mut tree_shake_modules_map);
                tree_shake_modules_ids.push_back((module_id, ShakeType::CircleRemove(true)));
              };
            }
            continue;
          }

          for import_info in used_imports {
            add_used_exports_by_import_info(
              &mut tree_shake_modules_map,
              &*module_graph,
              &tree_shake_module_id,
              &import_info,
            );
          }

          for export_info in used_exports_from {
            add_used_exports_by_export_info(
              &mut tree_shake_modules_map,
              &*module_graph,
              &tree_shake_module_id,
              &export_info,
            );
          }
        }
      }

      // add all dynamic imported dependencies as [UsedExports::All]
      for (dep, edge) in module_graph.dependencies(&tree_shake_module_id) {
        if edge.is_dynamic() && tree_shake_modules_map.contains_key(&dep) {
          let tree_shake_module = tree_shake_modules_map.get_mut(&dep).unwrap_or_else(|| {
            panic!("dynamic imported module not found: {:?}", dep);
          });
          tree_shake_module.side_effects = true;
          tree_shake_module.used_exports.change_all();
        }
      }
      visited_modules.insert(tree_shake_module_id.clone());
    }

    // update used_exports in module_graph
    for module in module_graph.modules_mut() {
      if let Some(tree_shake_module) = tree_shake_modules_map.get(&module.id) {
        let mut used_exports = tree_shake_module.used_exports.to_string_vec();
        used_exports.sort();

        module.used_exports = used_exports;
      }
    }

    // remove the unused modules
    for module_id in modules_to_remove {
      module_graph.remove_module(&module_id);
    }
    // if production remove useless hot update statements
    remove_useless_hot_update_stmts(module_graph);

    Ok(Some(()))
  }
}

// Add all imported to used_exports
fn add_used_exports_by_import_info(
  tree_shake_modules_map: &mut std::collections::HashMap<ModuleId, TreeShakeModule>,
  module_graph: &ModuleGraph,
  tree_shake_module_id: &ModuleId,
  import_info: &ImportInfo,
) {
  let imported_module_id =
    module_graph.get_dep_by_source(tree_shake_module_id, &import_info.source);
  let imported_module = module_graph.module(&imported_module_id).unwrap();

  if imported_module.external || !imported_module.module_type.is_script() {
    return;
  }

  let imported_tree_shake_module = tree_shake_modules_map
    .get_mut(&imported_module_id)
    .unwrap_or_else(|| {
      panic!("imported module not found: {:?}", imported_module_id);
    });

  if import_info.is_import_executed {
    imported_tree_shake_module.is_self_executed_import = true;
    return;
  }

  if import_info.specifiers.is_empty() {
    return;
  }

  for sp in &import_info.specifiers {
    match sp {
      statement_graph::ImportSpecifierInfo::Namespace(_) => {
        imported_tree_shake_module
          .used_exports
          .add_used_export(tree_shake_module_id, &module::UsedIdent::ExportAll);
      }
      statement_graph::ImportSpecifierInfo::Named { local, imported } => {
        if let Some(ident) = imported {
          if *ident == "default" {
            imported_tree_shake_module
              .used_exports
              .add_used_export(tree_shake_module_id, &module::UsedIdent::Default);
          } else {
            imported_tree_shake_module.used_exports.add_used_export(
              tree_shake_module_id,
              &module::UsedIdent::SwcIdent(strip_context(ident)),
            );
          }
        } else {
          imported_tree_shake_module.used_exports.add_used_export(
            tree_shake_module_id,
            &module::UsedIdent::SwcIdent(strip_context(local)),
          );
        }
      }
      statement_graph::ImportSpecifierInfo::Default(_) => {
        imported_tree_shake_module
          .used_exports
          .add_used_export(tree_shake_module_id, &module::UsedIdent::Default);
      }
    }
  }
}

fn remove_used_exports_by_import_info(
  tree_shake_modules_map: &mut std::collections::HashMap<ModuleId, TreeShakeModule>,
  module_graph: &ModuleGraph,
  tree_shake_module_id: &ModuleId,
  import_info: &ImportInfo,
  cyclic_nodes: &Vec<ModuleId>,
) -> Option<(ModuleId, Option<(ModuleId, ImportInfo)>)> {
  let imported_module_id =
    module_graph.get_dep_by_source(tree_shake_module_id, &import_info.source);
  let mut delay_process_specifies = vec![];
  if !cyclic_nodes.contains(&imported_module_id) {
    return None;
  }

  let imported_module = module_graph.module(&imported_module_id).unwrap();

  if imported_module.external || !imported_module.module_type.is_script() {
    return None;
  }

  let imported_tree_shake_module = tree_shake_modules_map
    .get_mut(&imported_module_id)
    .unwrap_or_else(|| {
      panic!("imported module not found: {:?}", imported_module_id);
    });

  if import_info.specifiers.is_empty() {
    return None;
  }

  let mut updated = false;

  for sp in &import_info.specifiers {
    match sp {
      statement_graph::ImportSpecifierInfo::Namespace(_) => {
        delay_process_specifies.push(sp.clone());
        updated = true;
      }
      statement_graph::ImportSpecifierInfo::Named { local, imported } => {
        if let Some(ident) = imported {
          if *ident == "default" {
            imported_tree_shake_module
              .used_exports
              .remove_used_export(tree_shake_module_id, &module::UsedIdent::Default);

            updated = true;
          } else {
            imported_tree_shake_module.used_exports.remove_used_export(
              tree_shake_module_id,
              &module::UsedIdent::SwcIdent(strip_context(ident)),
            );
            updated = true;
          }
        } else {
          imported_tree_shake_module.used_exports.remove_used_export(
            tree_shake_module_id,
            &module::UsedIdent::SwcIdent(strip_context(local)),
          );
          updated = true;
        }
      }
      statement_graph::ImportSpecifierInfo::Default(_) => {
        imported_tree_shake_module
          .used_exports
          .remove_used_export(tree_shake_module_id, &module::UsedIdent::Default);
        updated = true;
      }
    }
  }

  if updated {
    return Some((
      imported_module_id,
      if delay_process_specifies.is_empty() {
        None
      } else {
        Some((
          tree_shake_module_id.clone(),
          ImportInfo {
            specifiers: delay_process_specifies,
            source: import_info.source.clone(),
            stmt_id: import_info.stmt_id.clone(),
            is_import_executed: import_info.is_import_executed,
          },
        ))
      },
    ));
  }

  None
}

fn removed_used_exports_by_export_info(
  tree_shake_modules_map: &mut std::collections::HashMap<ModuleId, TreeShakeModule>,
  module_graph: &ModuleGraph,
  tree_shake_module_id: &ModuleId,
  export_info: &ExportInfo,
) -> Option<ModuleId> {
  if let Some(source) = &export_info.source {
    let exported_module_id = module_graph.get_dep_by_source(tree_shake_module_id, source);
    let exported_module = module_graph.module(&exported_module_id).unwrap();

    if !exported_module.module_type.is_script() || exported_module.external {
      return None;
    }

    let mut updated = false;

    let exported_tree_shake_module = tree_shake_modules_map.get_mut(&exported_module_id).unwrap();

    for sp in &export_info.specifiers {
      match sp {
        statement_graph::ExportSpecifierInfo::Namespace(_) => {}
        statement_graph::ExportSpecifierInfo::Named { local, .. } => {
          if local == &"default".to_string() {
            exported_tree_shake_module
              .used_exports
              .remove_used_export(tree_shake_module_id, &module::UsedIdent::Default);
          } else {
            exported_tree_shake_module.used_exports.remove_used_export(
              tree_shake_module_id,
              &module::UsedIdent::SwcIdent(strip_context(local)),
            );
          }
          updated = true;
        }
        statement_graph::ExportSpecifierInfo::Default => {
          exported_tree_shake_module
            .used_exports
            .remove_used_export(tree_shake_module_id, &module::UsedIdent::Default);
          updated = true;
        }

        statement_graph::ExportSpecifierInfo::All(used_idents) => {
          if let Some(used_idents) = used_idents {
            for ident in used_idents {
              if ident == "*" {
                exported_tree_shake_module
                  .used_exports
                  .remove_used_export(tree_shake_module_id, &module::UsedIdent::ExportAll);
                updated = true;
              } else {
                exported_tree_shake_module
                  .used_exports
                  .remove_used_export(tree_shake_module_id, &strip_context(ident));
                updated = true;
              }
            }
          } else {
            exported_tree_shake_module
              .used_exports
              .remove_used_export(tree_shake_module_id, &module::UsedIdent::ExportAll);
            updated = true;
          }
        }
      }
    }
    if updated {
      return Some(exported_module_id.clone());
    }
  }

  None
}

/// All all exported to used_exports
fn add_used_exports_by_export_info(
  tree_shake_modules_map: &mut std::collections::HashMap<ModuleId, TreeShakeModule>,
  module_graph: &ModuleGraph,
  tree_shake_module_id: &ModuleId,
  export_info: &ExportInfo,
) {
  if let Some(source) = &export_info.source {
    let exported_module_id = module_graph.get_dep_by_source(tree_shake_module_id, source);
    let exported_module = module_graph.module(&exported_module_id).unwrap();

    if exported_module.external {
      return;
    }

    let exported_tree_shake_module = tree_shake_modules_map.get_mut(&exported_module_id).unwrap();

    for sp in &export_info.specifiers {
      match sp {
        statement_graph::ExportSpecifierInfo::Namespace(_) => {
          // exported_tree_shake_module.used_exports.change_all();
          exported_tree_shake_module
            .used_exports
            .add_used_export(tree_shake_module_id, &module::UsedIdent::ExportAll);
        }
        statement_graph::ExportSpecifierInfo::Named { local, .. } => {
          if local == &"default".to_string() {
            exported_tree_shake_module
              .used_exports
              .add_used_export(tree_shake_module_id, &module::UsedIdent::Default);
          } else {
            exported_tree_shake_module.used_exports.add_used_export(
              tree_shake_module_id,
              &module::UsedIdent::SwcIdent(strip_context(local)),
            );
          }
        }
        statement_graph::ExportSpecifierInfo::Default => {
          exported_tree_shake_module
            .used_exports
            .add_used_export(tree_shake_module_id, &module::UsedIdent::Default);
        }
        statement_graph::ExportSpecifierInfo::All(used_idents) => {
          if let Some(used_idents) = used_idents {
            for ident in used_idents {
              if ident == "*" {
                exported_tree_shake_module
                  .used_exports
                  .add_used_export(tree_shake_module_id, &module::UsedIdent::ExportAll);
              } else {
                exported_tree_shake_module
                  .used_exports
                  .add_used_export(tree_shake_module_id, &strip_context(ident));
              }
            }
          } else {
            exported_tree_shake_module
              .used_exports
              .add_used_export(tree_shake_module_id, &module::UsedIdent::ExportAll);
          }
        }
      }
    }
  }
}

// reset used export
fn replace_used_export_by_export_info(
  tree_shake_modules_map: &mut std::collections::HashMap<ModuleId, TreeShakeModule>,
  module_graph: &ModuleGraph,
  tree_shake_module_id: &ModuleId,
  export_info: &ExportInfo,
) {
  if let Some(source) = &export_info.source {
    let exported_module_id = module_graph.get_dep_by_source(tree_shake_module_id, source);
    let exported_module = module_graph.module(&exported_module_id).unwrap();

    if exported_module.external {
      return;
    }

    let exported_tree_shake_module = tree_shake_modules_map.get_mut(&exported_module_id).unwrap();

    exported_tree_shake_module
      .used_exports
      .remove_all_used_export(tree_shake_module_id);

    add_used_exports_by_export_info(
      tree_shake_modules_map,
      module_graph,
      tree_shake_module_id,
      export_info,
    )
  }
}

fn strip_context(ident: &String) -> String {
  let ident_split = ident.split('#').collect::<Vec<_>>();
  ident_split[0].to_string()
}
