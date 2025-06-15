use std::collections::VecDeque;

use farmfe_core::{
  module::{module_graph::ModuleGraph, ModuleId},
  plugin::ResolveKind,
  HashMap, HashSet,
};

use crate::{
  module::{TreeShakeModule, UsedExports},
  statement_graph::traced_used_import::TracedUsedImportStatement,
};

pub mod remove_export_idents;
pub mod remove_useless_stmts;

pub fn tree_shake_modules(
  entry_module_ids: Vec<ModuleId>,
  module_graph: &mut ModuleGraph,
  tree_shake_modules_map: &mut HashMap<ModuleId, TreeShakeModule>,
) -> Vec<ModuleId> {
  let mut tree_shake_module_ids_queue = VecDeque::from(entry_module_ids);
  let mut visited_modules: HashSet<ModuleId> = HashSet::default();

  let set_dep_used_export_all =
    |dep_id: &ModuleId, tree_shake_modules_map: &mut HashMap<ModuleId, TreeShakeModule>| {
      let dep_tree_shake_module = tree_shake_modules_map.get_mut(dep_id);

      if let Some(dep_tree_shake_module) = dep_tree_shake_module {
        dep_tree_shake_module.pending_used_exports.set_export_all();
      }
    };

  // 1. traverse the tree_shake_modules in order, and mark all statement that should be preserved
  while let Some(tree_shake_module_id) = tree_shake_module_ids_queue.pop_front() {
    // handle non tree shakeable modules like css
    if !tree_shake_modules_map.contains_key(&tree_shake_module_id) {
      if !visited_modules.contains(&tree_shake_module_id) {
        // make sure all non tree shakeable modules are handled
        for (dep_id, _) in module_graph.dependencies(&tree_shake_module_id) {
          set_dep_used_export_all(&dep_id, tree_shake_modules_map);
          tree_shake_module_ids_queue.push_back(dep_id);
        }
        visited_modules.insert(tree_shake_module_id);
      }
      continue;
    }

    let tree_shake_module = tree_shake_modules_map
      .get_mut(&tree_shake_module_id)
      .unwrap();
    // Skip this module if all pending used exports are handled
    // This is used to handle cyclic dependencies
    if tree_shake_module.is_all_pending_used_exports_handled()
      && visited_modules.contains(&tree_shake_module_id)
    {
      continue;
    }
    visited_modules.insert(tree_shake_module_id.clone());

    // if module is not esm, mark all imported modules as [UsedExports::All]
    if !matches!(
      tree_shake_module.module_system,
      farmfe_core::module::ModuleSystem::EsModule
    ) {
      // all statements will be preserved when it not esm
      tree_shake_module.clear_pending_used_exports();

      for dep_id in module_graph.dependencies_ids(&tree_shake_module_id) {
        // avoid cyclic dependencies
        let dep_tree_shake_module = tree_shake_modules_map.get(&dep_id);
        if let Some(dep_tree_shake_module) = dep_tree_shake_module {
          if matches!(dep_tree_shake_module.handled_used_exports, UsedExports::All) {
            continue;
          }
        }

        set_dep_used_export_all(&dep_id, tree_shake_modules_map);
        tree_shake_module_ids_queue.push_back(dep_id);
      }
    } else {
      // the module is esm, trace all used statement in the statement graph, should analyze side effects of the statements too.
      let traced_import_stmts =
        trace_and_mark_used_statements(&tree_shake_module_id, module_graph, tree_shake_modules_map);
      // set dependencies' pending_used_exports
      for import_stmt in traced_import_stmts {
        let TracedUsedImportStatement {
          source,
          used_stmt_idents,
          kind,
          ..
        } = import_stmt;

        let dep_id = module_graph.get_dep_by_source(&tree_shake_module_id, &source, Some(kind));

        if let Some(dep_tree_shake_module) = tree_shake_modules_map.get_mut(&dep_id) {
          match used_stmt_idents {
            UsedExports::All => {
              dep_tree_shake_module.pending_used_exports.set_export_all();
            }
            UsedExports::Partial(used_stmt_idents) => {
              // add all unhandled used stmt idents to pending_used_exports
              for used_stmt_ident in used_stmt_idents {
                if !dep_tree_shake_module
                  .handled_used_exports
                  .contains(&used_stmt_ident)
                {
                  dep_tree_shake_module
                    .pending_used_exports
                    .add_used_export(used_stmt_ident);
                }
              }
            }
          }
        }

        tree_shake_module_ids_queue.push_back(dep_id);
      }
    }
  }

  let mut modules_to_remove = vec![];

  // 2. remove statements that is not used
  for tree_shake_module_id in &visited_modules {
    if tree_shake_modules_map.contains_key(tree_shake_module_id) {
      modules_to_remove.extend(remove_useless_stmts::remove_useless_stmts(
        tree_shake_module_id,
        module_graph,
        tree_shake_modules_map,
      ));
    }
  }

  modules_to_remove.extend(
    module_graph
      .modules()
      .into_iter()
      .map(|m| m.id.clone())
      .filter(|m_id| !visited_modules.contains(m_id)),
  );

  modules_to_remove
}

fn trace_and_mark_used_statements(
  tree_shake_module_id: &ModuleId,
  module_graph: &mut ModuleGraph,
  tree_shake_modules_map: &mut HashMap<ModuleId, TreeShakeModule>,
) -> Vec<TracedUsedImportStatement> {
  let tree_shake_module = tree_shake_modules_map
    .get_mut(tree_shake_module_id)
    .unwrap();
  let mut traced_import_stmts = tree_shake_module.trace_and_mark_used_statements();
  tree_shake_module.clear_pending_used_exports();

  // for import and export from statements that imports a module includes side effects statements, it should preserved
  let mut extra_used_import_export_from_stmts = vec![];

  let tree_shake_module = tree_shake_modules_map.get(tree_shake_module_id).unwrap();

  for stmt_id in tree_shake_module.stmt_graph.stmt_ids() {
    if !tree_shake_module.stmt_graph.used_stmts().contains(&stmt_id) {
      let stmt = tree_shake_module.stmt_graph.stmt(&stmt_id);
      let source_kind = if let Some(import_info) = &stmt.import_info {
        Some((import_info.source.as_str(), ResolveKind::Import))
      } else if let Some(export_info) = &stmt.export_info {
        export_info
          .source
          .as_ref()
          .map(|source| (source.as_str(), ResolveKind::ExportFrom))
      } else {
        None
      };

      if let Some((source, kind)) = source_kind {
        let dep_module_id =
          module_graph.get_dep_by_source(tree_shake_module_id, source, Some(kind.clone()));
        let dep_module = module_graph.module(&dep_module_id).unwrap();
        let dep_tree_shake_module = tree_shake_modules_map.get(&dep_module_id);

        // for dep tree shake module that marked as side effects free, Farm won't check it
        if dep_tree_shake_module.is_some() && !dep_module.side_effects && !dep_module.external {
          continue;
        }

        // if dep tree shake module is not found, it means the dep module is not tree shakable, so we should keep the import / export from statement
        // and preserve import / export from statement if the source module contains side effects statement
        if dep_module.external
          || dep_tree_shake_module.is_none()
          || dep_tree_shake_module.unwrap().contains_self_executed_stmt
        {
          if matches!(kind, ResolveKind::Import) {
            traced_import_stmts.push(TracedUsedImportStatement::from_import_info_and_used_idents(
              stmt.id,
              stmt.import_info.as_ref().unwrap(),
              &HashSet::default(),
              HashMap::default(),
            ));
          } else if matches!(kind, ResolveKind::ExportFrom) {
            traced_import_stmts.push(
              TracedUsedImportStatement::from_export_info_and_used_idents(
                stmt.id,
                stmt.export_info.as_ref().unwrap(),
                &HashSet::default(),
              )
              .unwrap(),
            );
          }

          extra_used_import_export_from_stmts.push(stmt.id);
        }
      }
    }
  }

  let tree_shake_module = tree_shake_modules_map
    .get_mut(tree_shake_module_id)
    .unwrap();

  for stmt_id in extra_used_import_export_from_stmts {
    tree_shake_module.stmt_graph.mark_used_statements(stmt_id);
  }

  // for dependency kind other than import and export from, always trace the dependency
  for (_, edge) in module_graph.dependencies(tree_shake_module_id) {
    for edge_item in edge.items() {
      if edge_item.kind != ResolveKind::Import && edge_item.kind != ResolveKind::ExportFrom {
        traced_import_stmts.push(TracedUsedImportStatement {
          // stmt_id for dynamic import is not available, so we use a random number
          stmt_id: 19990112,
          source: edge_item.source.clone(),
          used_stmt_idents: UsedExports::All,
          kind: edge_item.kind.clone(),
        });
      }
    }
  }

  traced_import_stmts
}
