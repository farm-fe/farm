use std::collections::{HashMap, HashSet, VecDeque};

use farmfe_core::module::{module_graph::ModuleGraph, ModuleId};

use crate::{
  module::TreeShakeModule, statement_graph::traced_used_import::TracedUsedImportStatement,
};

pub(crate) mod handle_side_effects_module;
pub mod remove_useless_stmts;
pub(crate) mod utils;

pub fn tree_shake_modules(
  mut tree_shake_modules_ids: Vec<ModuleId>,
  module_graph: &mut ModuleGraph,
  tree_shake_modules_map: &mut HashMap<ModuleId, TreeShakeModule>,
) -> Vec<ModuleId> {
  let mut tree_shake_module_ids_queue = VecDeque::from(tree_shake_modules_ids.clone());

  let mut visited_modules: HashSet<ModuleId> = HashSet::new();
  let mut modules_to_remove: Vec<ModuleId> = vec![];

  // 1. traverse the tree_shake_modules in order, and mark all statement that should be preserved
  while let Some(tree_shake_module_id) = tree_shake_module_ids_queue.pop_front() {
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
      // mark the non-esm module as side_effects
      // Farm won't tree shake the side effects module
      tree_shake_module.side_effects = true;

      for (dep_id, _) in module_graph.dependencies(&tree_shake_module_id) {
        let dep_tree_shake_module = tree_shake_modules_map.get_mut(&dep_id);

        if let Some(dep_tree_shake_module) = dep_tree_shake_module {
          dep_tree_shake_module.pending_used_exports.set_export_all();
        }
      }
    } else {
      // the module is esm
      if tree_shake_module.side_effects {
        // the module has side effects, add all imported identifiers to [UsedExports::Partial] of the imported modules
        handle_side_effects_module::handle_side_effects_module(
          &tree_shake_module_id,
          tree_shake_modules_map,
          module_graph,
        );
      } else {
        // the module doesn't have side effects, trace all used statement in the statement graph, should analyze side effects of the statements too.
        let traced_import_stmts = tree_shake_module.trace_and_mark_used_statements();
        // clear pending_used_exports
        tree_shake_module.clear_pending_used_exports();
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
            // if all pending used exports are handled, add the module to the queue
            // the module will be processed multiple times when there are cyclic dependencies
            if !dep_tree_shake_module.is_all_pending_used_exports_handled() {
              tree_shake_module_ids_queue.push_back(dep_id);
            }
          }
        }
      }
    }
  }

  // traverse the tree_shake_modules from bottom to top
  // so the import statement would be preserved when the dependency module is not empty
  tree_shake_modules_ids.reverse();
  // 2. remove statements that should is not used
  for tree_shake_module_id in tree_shake_modules_ids {
    remove_useless_stmts::remove_useless_stmts(
      &tree_shake_module_id,
      module_graph,
      tree_shake_modules_map,
    );

    let module = module_graph.module(&tree_shake_module_id).unwrap();
    let tree_shake_module = tree_shake_modules_map.get(&tree_shake_module_id).unwrap();
    // do not remove the module if it is external or has side effects or does not have statements
    if !module.external
      && !module.side_effects
      && !tree_shake_module.side_effects
      && module.meta.as_script().ast.body.is_empty()
    {
      modules_to_remove.push(tree_shake_module_id);
    }
  }

  modules_to_remove
}
