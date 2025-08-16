use std::collections::VecDeque;

use farmfe_core::{
  module::{
    meta_data::script::statement::SwcId, module_graph::ModuleGraph, ModuleId, ModuleSystem,
  },
  plugin::ResolveKind,
  HashMap, HashSet,
};

use crate::{
  module::{TreeShakeModule, UsedExports, UsedExportsIdent},
  statement_graph::{
    analyze_used_import_all_fields::UsedImportAllFields,
    traced_used_import::TracedUsedImportStatement, StatementId, StatementSideEffects,
    UsedStatementIdent,
  },
};

pub mod remove_export_idents;
pub mod remove_useless_stmts;

pub fn tree_shake_modules(
  entry_module_ids: Vec<ModuleId>,
  module_graph: &mut ModuleGraph,
  tree_shake_modules_map: &mut HashMap<ModuleId, TreeShakeModule>,
) -> Vec<ModuleId> {
  let mut visited_modules: HashSet<ModuleId> = HashSet::default();

  // 1. traverse the tree_shake_modules in order, and trace the used import/used and mark all statement that should be preserved
  traverse_tree_shake_modules(
    entry_module_ids,
    module_graph,
    tree_shake_modules_map,
    &mut visited_modules,
  );

  // 2. traverse the tree_shake_modules in order, handle SideEffects::WriteTopLevelVar that writes imported ident,
  //  2.1 if the the imported idents are used, we should preserve it
  //  2.2 if the imported idents are read global variables, we should preserve it
  let extra_visited_modules =
    trace_and_mark_write_imported_idents_statements(tree_shake_modules_map, module_graph);

  visited_modules.extend(extra_visited_modules);

  let mut modules_to_remove = HashSet::default();

  modules_to_remove.extend(
    module_graph
      .modules()
      .into_iter()
      .map(|m| m.id.clone())
      .filter(|m_id| !visited_modules.contains(m_id)),
  );

  // sort visited_modules by execution order
  let mut visited_modules: Vec<ModuleId> = visited_modules.into_iter().collect();
  visited_modules.sort_by(|a, b| {
    let a_meta = module_graph.module(a).unwrap();
    let b_meta = module_graph.module(b).unwrap();

    a_meta.execution_order.cmp(&b_meta.execution_order)
  });

  // 3. remove statements that should not be used
  for tree_shake_module_id in &visited_modules {
    if tree_shake_modules_map.contains_key(tree_shake_module_id) {
      modules_to_remove.extend(remove_useless_stmts::remove_useless_stmts(
        tree_shake_module_id,
        module_graph,
        tree_shake_modules_map,
      ));
    }
  }

  modules_to_remove.into_iter().collect()
}

fn traverse_tree_shake_modules(
  entry_module_ids: Vec<ModuleId>,
  module_graph: &mut ModuleGraph,
  tree_shake_modules_map: &mut HashMap<ModuleId, TreeShakeModule>,
  visited_modules: &mut HashSet<ModuleId>,
) {
  let mut tree_shake_module_ids_queue = VecDeque::from(entry_module_ids);

  let set_dep_used_export_all =
    |dep_id: &ModuleId, tree_shake_modules_map: &mut HashMap<ModuleId, TreeShakeModule>| {
      let dep_tree_shake_module = tree_shake_modules_map.get_mut(dep_id);

      if let Some(dep_tree_shake_module) = dep_tree_shake_module {
        dep_tree_shake_module.pending_used_exports.set_export_all();
      }
    };

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
      let module_ids = set_pending_used_exports(
        &tree_shake_module_id,
        module_graph,
        tree_shake_modules_map,
        traced_import_stmts,
      );

      for module_id in module_ids {
        tree_shake_module_ids_queue.push_back(module_id);
      }
    }
  }
}

fn set_pending_used_exports(
  tree_shake_module_id: &ModuleId,
  module_graph: &ModuleGraph,
  tree_shake_modules_map: &mut HashMap<ModuleId, TreeShakeModule>,
  traced_import_stmts: Vec<TracedUsedImportStatement>,
) -> Vec<ModuleId> {
  let mut module_ids = vec![];

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

    module_ids.push(dep_id);
  }

  module_ids
}

fn trace_and_mark_used_statements(
  tree_shake_module_id: &ModuleId,
  module_graph: &ModuleGraph,
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

/// For case like:
/// import { a } from './src/foo';
/// a.field = 1;
/// export { a };
///
/// a is not used in the module, but it is used in the module's dependency, this has side effects for ident a, so we should preserve the import statement
fn trace_and_mark_write_imported_idents_statements(
  tree_shake_modules_map: &mut HashMap<ModuleId, TreeShakeModule>,
  module_graph: &mut ModuleGraph,
) -> HashSet<ModuleId> {
  let mut tree_shake_modules_to_trace = vec![];
  let mut write_side_effects_to_trace = vec![];

  for tree_shake_module in tree_shake_modules_map.values() {
    if !tree_shake_module
      .stmt_graph
      .written_imported_idents
      .is_empty()
    {
      let mut used_stmt_exports = HashMap::default();
      let mut used_import_all_fields = HashMap::default();

      for written_imported_ident in &tree_shake_module.stmt_graph.written_imported_idents {
        if let Some(stmt_id) = tree_shake_module
          .stmt_graph
          .reverse_defined_idents_map
          .get(&written_imported_ident.ident)
        {
          // if the imported ident is marked as used in the dep module, we should preserve the import statement
          let stmt = tree_shake_module.stmt_graph.stmt(stmt_id);
          let import_info = stmt.import_info.as_ref().unwrap();
          let source = import_info.source.as_str();
          let dep_module_id = module_graph.get_dep_by_source(
            &tree_shake_module.module_id,
            source,
            Some(ResolveKind::Import),
          );

          if let Some(dep_tree_shake_module) = tree_shake_modules_map.get(&dep_module_id) {
            // if the dep module is not esm, we should always preserve the import statement
            if !matches!(dep_tree_shake_module.module_system, ModuleSystem::EsModule) {
              used_stmt_exports
                .entry(*stmt_id)
                .or_insert_with(HashSet::default)
                .insert(UsedStatementIdent::SwcIdent(
                  written_imported_ident.ident.clone(),
                ));
              continue;
            }

            for specifier in &import_info.specifiers {
              match specifier {
                crate::statement_graph::ImportSpecifierInfo::Namespace(ns) => {
                  if ns == &written_imported_ident.ident {
                    if let Some(fields) = &written_imported_ident.fields {
                      let used_fields = used_import_all_fields
                        .entry(written_imported_ident.ident.clone())
                        .or_insert_with(HashSet::default);

                      for field in fields {
                        match field {
                          UsedImportAllFields::All => {
                            // This case is handled in previous round of tree shaking, all exports are preserved in this case
                          }
                          UsedImportAllFields::Ident(field_str)
                          | UsedImportAllFields::LiteralComputed(field_str) => {
                            // 1. whether the export is being used in the dep module
                            if dep_tree_shake_module
                              .handled_used_exports
                              .contains(&UsedExportsIdent::SwcIdent(field_str.to_string()))
                            {
                              used_stmt_exports
                                .entry(*stmt_id)
                                .or_insert_with(HashSet::default)
                                .insert(UsedStatementIdent::SwcIdent(
                                  written_imported_ident.ident.clone(),
                                ));

                              used_fields.insert(field.clone());
                            } else {
                              // 2. if the export is not being used in the dep module, we should trace the dep module to find if the export idents use global variables
                              write_side_effects_to_trace.push(
                                TraceDepModuleWriteSideEffectsItem {
                                  module_id: tree_shake_module.module_id.clone(),
                                  dep_module_id: dep_module_id.clone(),
                                  ident: written_imported_ident.ident.clone(),
                                  is_namespace: true,
                                  export: field_str.to_string().into(),
                                },
                              );
                            }
                          }
                        }
                      }
                    }
                  }
                }
                crate::statement_graph::ImportSpecifierInfo::Named { local, imported } => {
                  if local == &written_imported_ident.ident {
                    let export_str = imported
                      .as_ref()
                      .map(|i| i.sym.to_string())
                      .unwrap_or(local.sym.to_string());

                    if dep_tree_shake_module
                      .handled_used_exports
                      .contains(&UsedExportsIdent::SwcIdent(export_str.clone()))
                    {
                      used_stmt_exports
                        .entry(*stmt_id)
                        .or_insert_with(HashSet::default)
                        .insert(UsedStatementIdent::SwcIdent(
                          written_imported_ident.ident.clone(),
                        ));
                    } else {
                      // we need to trace the dep module to find if the export idents use global variables
                      write_side_effects_to_trace.push(TraceDepModuleWriteSideEffectsItem {
                        module_id: tree_shake_module.module_id.clone(),
                        dep_module_id: dep_module_id.clone(),
                        ident: written_imported_ident.ident.clone(),
                        is_namespace: false,
                        export: export_str.into(),
                      });
                    }
                  }
                }
                crate::statement_graph::ImportSpecifierInfo::Default(ident) => {
                  if ident == &written_imported_ident.ident {
                    if dep_tree_shake_module
                      .handled_used_exports
                      .contains(&UsedExportsIdent::Default)
                    {
                      used_stmt_exports
                        .entry(*stmt_id)
                        .or_insert_with(HashSet::default)
                        .insert(UsedStatementIdent::SwcIdent(
                          written_imported_ident.ident.clone(),
                        ));

                      write_side_effects_to_trace.push(TraceDepModuleWriteSideEffectsItem {
                        module_id: tree_shake_module.module_id.clone(),
                        dep_module_id: dep_module_id.clone(),
                        ident: written_imported_ident.ident.clone(),
                        is_namespace: false,
                        export: "default".to_string().into(),
                      });
                    }
                  }
                }
              }
            }
          }
        }
      }

      tree_shake_modules_to_trace.push((
        tree_shake_module.module_id.clone(),
        used_stmt_exports,
        used_import_all_fields,
      ));
    }
  }

  let mut dep_module_ids = vec![];

  for (module_id, used_stmt_exports, used_import_all_fields) in tree_shake_modules_to_trace {
    if let Some(tree_shake_module) = tree_shake_modules_map.get_mut(&module_id) {
      // trace and mark used import statements
      let traced_import_stmts = tree_shake_module
        .stmt_graph
        .trace_and_mark_used_statements(used_stmt_exports, Some(used_import_all_fields));

      let module_ids = set_pending_used_exports(
        &module_id,
        module_graph,
        tree_shake_modules_map,
        traced_import_stmts,
      );

      for module_id in module_ids {
        if !dep_module_ids.contains(&module_id) {
          dep_module_ids.push(module_id);
        }
      }
    }
  }

  // handle write side effects when dep module read global variables
  let write_side_effects_to_trace_map =
    write_side_effects_to_trace
      .into_iter()
      .fold(HashMap::default(), |mut acc, item| {
        acc
          .entry(item.dep_module_id.clone())
          .or_insert_with(Vec::new)
          .push(item);
        acc
      });

  let mut current_module_to_trace = HashMap::default();

  for (dep_module_id, items) in write_side_effects_to_trace_map {
    for item in items {
      let is_read_global_var = find_dep_module_read_global_variables_dfs(
        &dep_module_id,
        item.export.clone(),
        tree_shake_modules_map,
        module_graph,
        &mut HashSet::default(),
      );

      if is_read_global_var {
        if let Some(tree_shake_module) = tree_shake_modules_map.get_mut(&dep_module_id) {
          tree_shake_module
            .pending_used_exports
            .add_used_export(item.export.clone());
        }

        // trace statements of current current module
        if let Some(tree_shake_module) = tree_shake_modules_map.get(&item.module_id) {
          if let Some(stmt_id) = tree_shake_module
            .stmt_graph
            .reverse_defined_idents_map
            .get(&item.ident)
          {
            let res = current_module_to_trace
              .entry(item.module_id)
              .or_insert((HashMap::default(), HashMap::default()));
            let used_stmt_exports: &mut HashMap<usize, HashSet<UsedStatementIdent>> = &mut res.0;
            let used_import_all_fields: &mut HashMap<SwcId, HashSet<UsedImportAllFields>> =
              &mut res.1;

            used_stmt_exports
              .entry(*stmt_id)
              .or_insert_with(HashSet::default)
              .insert(UsedStatementIdent::SwcIdent(item.ident.clone()));

            if item.is_namespace {
              used_import_all_fields
                .entry(item.ident.clone())
                .or_insert_with(HashSet::default)
                .insert(UsedImportAllFields::Ident(item.export.to_string()));
            }
          }
        }

        if !dep_module_ids.contains(&dep_module_id) {
          dep_module_ids.push(dep_module_id.clone());
        }
      }
    }
  }

  // trace statements of current current module
  for (module_id, (used_stmt_exports, used_import_all_fields)) in current_module_to_trace {
    if let Some(tree_shake_module) = tree_shake_modules_map.get_mut(&module_id) {
      let traced_import_stmts = tree_shake_module
        .stmt_graph
        .trace_and_mark_used_statements(used_stmt_exports, Some(used_import_all_fields));

      let module_ids = set_pending_used_exports(
        &module_id,
        module_graph,
        tree_shake_modules_map,
        traced_import_stmts,
      );

      for module_id in module_ids {
        if !dep_module_ids.contains(&module_id) {
          dep_module_ids.push(module_id);
        }
      }
    }
  }

  let mut visited_module_ids = HashSet::default();
  // trace dep modules
  traverse_tree_shake_modules(
    dep_module_ids,
    module_graph,
    tree_shake_modules_map,
    &mut visited_module_ids,
  );

  visited_module_ids
}

#[derive(Debug)]
struct TraceDepModuleWriteSideEffectsItem {
  pub module_id: ModuleId,
  pub dep_module_id: ModuleId,
  pub ident: SwcId,
  pub is_namespace: bool,
  pub export: UsedExportsIdent,
}

/// Preserve the statements of a dep module that read global variables
///   For case like:
/// ```js
/// import { a } from './src/foo';
/// a.field = 1;
/// // in src/foo
/// export const a = window;
/// export const b = 1;
/// ```
/// We should preserve the statement `a.field = 1;` and remove `b`
fn find_dep_module_read_global_variables_dfs(
  dep_module_id: &ModuleId,
  ident: UsedExportsIdent,
  tree_shake_modules_map: &HashMap<ModuleId, TreeShakeModule>,
  module_graph: &ModuleGraph,
  module_visited: &mut HashSet<ModuleId>,
) -> bool {
  if module_visited.contains(dep_module_id) {
    return false;
  }

  module_visited.insert(dep_module_id.clone());

  if let Some(tree_shake_module) = tree_shake_modules_map.get(dep_module_id) {
    // 1. find export statement
    let stmt_idents =
      tree_shake_module.used_exports_idents_to_statement_idents(&HashSet::from_iter([ident]));
    let stmt_used_idents_map = tree_shake_module.get_stmt_used_idents_map(stmt_idents);

    // 2. trace from export statement and find unused statements that read global variables
    let mut queue = VecDeque::from(
      stmt_used_idents_map
        .into_iter()
        .map(|i| (i.0, i.1, HashMap::default()))
        .collect::<Vec<_>>(),
    );

    let mut all_traced_import_stmts = vec![];
    let mut stmt_visited = HashSet::<StatementId>::default();

    while let Some((stmt_id, used_defined_idents, used_import_all_fields)) = queue.pop_front() {
      if tree_shake_module.stmt_graph.used_stmts().contains(&stmt_id)
        || stmt_visited.contains(&stmt_id)
      {
        continue;
      }

      stmt_visited.insert(stmt_id);

      let stmt = tree_shake_module.stmt_graph.stmt(&stmt_id);

      if let StatementSideEffects::ReadTopLevelVar(top_level_vars) = &stmt.side_effects {
        for top_level_var in top_level_vars {
          if top_level_var.is_global_var {
            return true;
          }
        }
      }

      if let Some(import_info) = &stmt.import_info {
        let traced_import_stmts = TracedUsedImportStatement::from_import_info_and_used_idents(
          stmt.id,
          import_info,
          &used_defined_idents,
          used_import_all_fields,
        );
        all_traced_import_stmts.push(traced_import_stmts);
      }

      for (dep_stmt, edge) in tree_shake_module.stmt_graph.dependencies(&stmt_id) {
        let all_used_dep_defined_idents = tree_shake_module
          .stmt_graph
          .find_all_used_defined_idents(&stmt_id, dep_stmt, &used_defined_idents, edge);

        queue.push_back((
          dep_stmt.id,
          all_used_dep_defined_idents
            .into_iter()
            .map(|i| UsedStatementIdent::SwcIdent(i))
            .collect(),
          edge.used_import_all_fields.clone(),
        ));
      }
    }

    // 3. trace dep module if another import statement is found
    for import_stmt in all_traced_import_stmts {
      let TracedUsedImportStatement {
        source,
        used_stmt_idents,
        kind,
        ..
      } = import_stmt;
      let dep_module_id =
        module_graph.get_dep_by_source(dep_module_id, source.as_str(), Some(kind));

      if let UsedExports::Partial(used_stmt_idents) = used_stmt_idents {
        for used_stmt_ident in used_stmt_idents {
          if find_dep_module_read_global_variables_dfs(
            &dep_module_id,
            used_stmt_ident,
            tree_shake_modules_map,
            module_graph,
            module_visited,
          ) {
            return true;
          }
        }
      }
    }
  }

  false
}
