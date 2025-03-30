use std::collections::VecDeque;

use farmfe_core::module::module_graph::ModuleGraphEdge;
use farmfe_core::module::module_group::{ModuleGroupId, ModuleGroupType};
use farmfe_core::module::{
  module_graph::ModuleGraph,
  module_group::{ModuleGroup, ModuleGroupGraph},
  Module, ModuleId,
};

use farmfe_core::{HashMap, HashSet};

use super::diff_and_patch_module_graph::DiffResult;

pub fn patch_module_group_graph(
  updated_module_ids: Vec<ModuleId>,
  diff_result: &DiffResult,
  removed_modules: &HashMap<ModuleId, Module>,
  module_graph: &mut ModuleGraph,
  module_group_graph: &mut ModuleGroupGraph,
) -> HashSet<ModuleGroupId> {
  let mut affected_module_groups = HashSet::default();

  for updated_module_id in &updated_module_ids {
    let module = module_graph.module(updated_module_id).unwrap();
    let module_group_ids = module.module_groups.clone();
    affected_module_groups.extend(module_group_ids);
  }

  let deps_changes = &diff_result.deps_changes;

  for (module_id, deps_diff_result) in deps_changes {
    // handle removed first so the new added module won't be removed by the removed module
    for (removed_module_id, edge_info) in &deps_diff_result.removed {
      patch_removed_dynamic_import_and_dynamic_entry(
        removed_module_id,
        module_id,
        edge_info,
        removed_modules,
        module_graph,
        module_group_graph,
        &mut affected_module_groups,
      );
    }

    for (added_module_id, edge_info) in &deps_diff_result.added {
      patch_added_dynamic_import_and_dynamic_entry(
        added_module_id,
        module_id,
        edge_info,
        diff_result,
        removed_modules,
        module_graph,
        module_group_graph,
        &mut affected_module_groups,
      );
    }
  }

  // Do not handle removed module group
  let affected_module_groups = affected_module_groups
    .into_iter()
    .filter(|g_id| module_group_graph.has(g_id))
    .collect::<Vec<_>>();

  let mut final_affected_module_groups = HashSet::default();
  let mut queue = VecDeque::from(affected_module_groups);
  // makes sure that all module groups that are affected are included
  while !queue.is_empty() {
    let module_group_id = queue.pop_front().unwrap();
    let module_group = module_group_graph.module_group(&module_group_id).unwrap();

    for module_id in module_group.modules() {
      let module = module_graph.module(module_id).unwrap();

      for module_group_id in &module.module_groups {
        if !final_affected_module_groups.contains(module_group_id) {
          final_affected_module_groups.insert(module_group_id.clone());
          queue.push_back(module_group_id.clone());
        }
      }
    }
  }

  final_affected_module_groups
}

fn get_previous_module_groups(
  module_id: &ModuleId,
  module_graph: &mut ModuleGraph,
  removed_modules: &HashMap<ModuleId, Module>,
) -> HashSet<ModuleGroupId> {
  if module_graph.has_module(module_id) {
    let module = module_graph.module(module_id).unwrap();
    module.module_groups.clone()
  } else {
    let removed_module = removed_modules.get(module_id).unwrap();
    removed_module.module_groups.clone()
  }
}

fn add_dynamic_module_group(
  added_module_id: &ModuleId,
  module_id: &ModuleId,
  module_group_type: ModuleGroupType,
  diff_result: &DiffResult,
  module_graph: &mut ModuleGraph,
  module_group_graph: &mut ModuleGroupGraph,
  affected_module_groups: &mut HashSet<ModuleGroupId>,
) {
  let added_module_group_id = ModuleGroupId::new(added_module_id, &module_group_type);

  let add_edges = |module_group_graph: &mut ModuleGroupGraph| {
    // only add edge for dynamic import
    if !matches!(module_group_type, ModuleGroupType::DynamicEntry) {
      let module_group_ids = {
        let module = module_graph
          .module(module_id)
          .unwrap_or_else(|| panic!("module {module_id:?} not found"));
        module.module_groups.clone()
      };

      for module_group_id in &module_group_ids {
        if !module_group_graph.has_edge(module_group_id, &added_module_group_id) {
          module_group_graph.add_edge(module_group_id, &added_module_group_id);
        }
      }
    }
  };

  // create new module group only when the module group does not exist
  if module_group_graph.has(&added_module_group_id) {
    add_edges(module_group_graph);
    return;
  }

  let module_group = ModuleGroup::new(added_module_id.clone(), module_group_type.clone());
  module_group_graph.add_module_group(module_group);

  add_edges(module_group_graph);

  affected_module_groups.insert(added_module_group_id.clone());

  // if the dynamic import is not a new module, we need to add the module to the module group of the parent module
  if diff_result.added_modules.contains(added_module_id) {
    let module = module_graph.module_mut(added_module_id).unwrap();
    module.module_groups.insert(added_module_group_id.clone());
  } else {
    patch_existing_added_non_dynamic_children(
      added_module_id,
      HashSet::from_iter([added_module_group_id]),
      module_graph,
      module_group_graph,
      affected_module_groups,
      &mut HashSet::default(),
    );
  }
}

fn remove_dynamic_module_group(
  removed_group_id: &ModuleGroupId,
  module_graph: &mut ModuleGraph,
  module_group_graph: &mut ModuleGroupGraph,
  affected_module_groups: &mut HashSet<ModuleGroupId>,
) {
  if !module_group_graph.has(removed_group_id) {
    return;
  }

  // means this module is no longer imported by any dynamic import, and its module group should be removed,
  // as well as all modules inside this module group
  let module_group = module_group_graph
    .module_group_mut(removed_group_id)
    .unwrap_or_else(|| panic!("module group {removed_group_id:?} not found"));

  module_group.modules().iter().for_each(|module_id| {
    let module = module_graph.module_mut(module_id).unwrap();
    module.module_groups.remove(removed_group_id);
    affected_module_groups.extend(module.module_groups.clone());
  });
  // do not need to remove the edge cause it will be removed automatically when the module is removed
  module_group_graph.remove_module_group(removed_group_id);
}

/// Patch the module group graph when a dynamic import is added
/// - If the added module is a dynamic import, create a new module group for this module
/// - If the added module is a normal import, extend the module group of the parent module
fn patch_added_dynamic_import_and_dynamic_entry(
  added_module_id: &ModuleId,
  module_id: &ModuleId,
  edge_info: &ModuleGraphEdge,
  diff_result: &DiffResult,
  removed_modules: &HashMap<ModuleId, Module>,
  module_graph: &mut ModuleGraph,
  module_group_graph: &mut ModuleGroupGraph,
  affected_module_groups: &mut HashSet<ModuleGroupId>,
) {
  if edge_info.contains_dynamic_entry() {
    add_dynamic_module_group(
      added_module_id,
      module_id,
      ModuleGroupType::DynamicEntry,
      diff_result,
      module_graph,
      module_group_graph,
      affected_module_groups,
    );
  }

  let previous_parent_groups = get_previous_module_groups(module_id, module_graph, removed_modules);

  if edge_info.contains_dynamic_import() {
    // if the edge is a dynamic import, we need to create a new module group for this module
    add_dynamic_module_group(
      added_module_id,
      module_id,
      ModuleGroupType::DynamicImport,
      diff_result,
      module_graph,
      module_group_graph,
      affected_module_groups,
    );
    // for dynamic entry groups, we have to patch the module group no matter it is dynamic import or not
    patch_dynamic_entry_group_for_added_dynamic_import(
      vec![added_module_id.clone()],
      previous_parent_groups.clone(),
      module_graph,
      module_group_graph,
      affected_module_groups,
      &mut HashSet::default(),
    );
  }

  if edge_info.contains_static() {
    // if the edge is a normal import, we need to add this module to the module group of the parent module
    // new module
    if diff_result.added_modules.contains(added_module_id) {
      for module_group_id in &previous_parent_groups {
        let module_group = module_group_graph
          .module_group_mut(module_group_id)
          .unwrap();
        module_group.add_module(added_module_id.clone());
        affected_module_groups.insert(module_group_id.clone());
      }
      let module = module_graph.module_mut(added_module_id).unwrap();
      module.module_groups.extend(previous_parent_groups);
    } else {
      // also need to handle all of its non-dynamic children
      patch_existing_added_non_dynamic_children(
        added_module_id,
        previous_parent_groups,
        module_graph,
        module_group_graph,
        affected_module_groups,
        &mut HashSet::default(),
      );
    }
  }
}

fn patch_removed_dynamic_import_and_dynamic_entry(
  removed_module_id: &ModuleId,
  module_id: &ModuleId,
  edge_info: &ModuleGraphEdge,
  removed_modules: &HashMap<ModuleId, Module>,
  module_graph: &mut ModuleGraph,
  module_group_graph: &mut ModuleGroupGraph,
  affected_module_groups: &mut HashSet<ModuleGroupId>,
) {
  if module_graph.has_module(removed_module_id) {
    let previous_parent_groups =
      get_previous_module_groups(module_id, module_graph, removed_modules);
    let mut module_groups_to_remove = HashSet::default();
    // a edge is removed, so we need to remove the module from the module group if necessary
    let current_parents = module_graph.dependents(removed_module_id);

    // if dynamic entry is removed and there is no other dynamic entry parent, remove the module group
    if edge_info.contains_dynamic_entry()
      && current_parents
        .iter()
        .all(|(_, edge_info)| !edge_info.contains_dynamic_entry())
    {
      let removed_group_id = ModuleGroupId::new(removed_module_id, &ModuleGroupType::DynamicEntry);
      module_groups_to_remove.insert(removed_group_id);
    }

    if edge_info.contains_dynamic_import() && !edge_info.contains_static() {
      let removed_group_id = ModuleGroupId::new(removed_module_id, &ModuleGroupType::DynamicImport);

      if current_parents
        .iter()
        .filter(|(_, edge_info)| edge_info.is_dynamic_import())
        .count()
        == 0
      {
        module_groups_to_remove.insert(removed_group_id);
      } else {
        let module_group_ids = {
          if removed_modules.contains_key(module_id) {
            let removed_module = removed_modules.get(module_id).unwrap();
            removed_module.module_groups.clone()
          } else {
            let module = module_graph
              .module(module_id)
              .unwrap_or_else(|| panic!("module {module_id:?} not found"));
            module.module_groups.clone()
          }
        };
        // remove the edge
        for module_group_id in &module_group_ids {
          if current_parents
            .iter()
            .filter(|(p, edge_info)| {
              if edge_info.is_dynamic_import() {
                let parent = module_graph.module(p).unwrap();
                return parent.module_groups.contains(module_group_id);
              }

              false
            })
            .count()
            == 0
            && module_group_graph.has_edge(module_group_id, &removed_group_id)
          {
            module_group_graph.remove_edge(module_group_id, &removed_group_id);
          }
        }
      }
    } else {
      patch_existing_removed_non_dynamic_children(
        removed_module_id,
        previous_parent_groups.clone(),
        module_graph,
        module_group_graph,
        affected_module_groups,
      );
    }

    patch_dynamic_entry_group_for_removed_dynamic_import(
      removed_module_id,
      previous_parent_groups,
      module_graph,
      module_group_graph,
      affected_module_groups,
    );

    for removed_group_id in module_groups_to_remove {
      remove_dynamic_module_group(
        &removed_group_id,
        module_graph,
        module_group_graph,
        affected_module_groups,
      );
    }
  } else {
    // this module is removed, all module groups that contains this module should remove this module
    let removed_module = removed_modules.get(removed_module_id).unwrap();

    for removed_module_group_id in &removed_module.module_groups {
      let module_group = module_group_graph.module_group_mut(removed_module_group_id);

      if let Some(module_group) = module_group {
        module_group.remove_module(removed_module_id);
        affected_module_groups.insert(removed_module_group_id.clone());
        let modules_len = module_group.modules().len();

        if modules_len == 0 {
          module_group_graph.remove_module_group(removed_module_group_id);
        }
      }
    }
  }
}

fn patch_existing_added_non_dynamic_children(
  added_module_id: &ModuleId,
  previous_parent_groups: HashSet<ModuleGroupId>,
  module_graph: &mut ModuleGraph,
  module_group_graph: &mut ModuleGroupGraph,
  affected_module_groups: &mut HashSet<ModuleGroupId>,
  visited: &mut HashSet<ModuleId>,
) {
  let mut queue = VecDeque::from([added_module_id.clone()]);

  while !queue.is_empty() {
    let current_module_id = queue.pop_front().unwrap();

    if visited.contains(&current_module_id) {
      continue;
    }
    visited.insert(current_module_id.clone());

    let mut current_module_group_change = false;

    for module_group_id in &previous_parent_groups {
      let current_module = module_graph
        .module(&current_module_id)
        .unwrap_or_else(|| panic!("module {current_module_id:?} not found"));

      if current_module.module_groups.contains(module_group_id) {
        continue;
      }

      current_module_group_change = true;

      let module_group = module_group_graph
        .module_group_mut(module_group_id)
        .unwrap();

      module_group.add_module(current_module_id.clone());
      let current_module = module_graph.module_mut(&current_module_id).unwrap();
      current_module.module_groups.insert(module_group_id.clone());
      affected_module_groups.insert(module_group_id.clone());

      // only add edge for dynamic import or static import
      if !matches!(
        module_group.module_group_type,
        ModuleGroupType::DynamicEntry
      ) {
        for (child, edge_info) in module_graph.dependencies(&current_module_id) {
          let child_group_id = ModuleGroupId::new(&child, &ModuleGroupType::DynamicImport);

          if edge_info.is_dynamic_import()
            && !module_group_graph.has_edge(module_group_id, &child_group_id)
          {
            module_group_graph.add_edge(module_group_id, &child_group_id);
          }
        }
      }
    }

    if current_module_group_change {
      let mut dynamic_imported_children = vec![];

      for (child, edge_info) in module_graph.dependencies(&current_module_id) {
        if !edge_info.is_dynamic_import() {
          queue.push_back(child);
        } else {
          dynamic_imported_children.push(child);
        }
      }

      if dynamic_imported_children.is_empty() {
        continue;
      }

      // for dynamic entry groups, we have to patch the module group no matter it is dynamic import or not
      patch_dynamic_entry_group_for_added_dynamic_import(
        dynamic_imported_children,
        previous_parent_groups.clone(),
        module_graph,
        module_group_graph,
        affected_module_groups,
        visited,
      );
    }
  }
}

fn patch_existing_removed_non_dynamic_children(
  removed_module_id: &ModuleId,
  previous_parent_groups: HashSet<ModuleGroupId>,
  module_graph: &mut ModuleGraph,
  module_group_graph: &mut ModuleGroupGraph,
  affected_module_groups: &mut HashSet<ModuleGroupId>,
) {
  let (result, cyclic) =
    topo_sort_removed_module(removed_module_id, module_graph, &|_, edge_info| {
      !edge_info.is_dynamic_import()
    });

  patch_any_module_group_for_toposort_modules(
    result,
    cyclic,
    previous_parent_groups,
    module_graph,
    module_group_graph,
    affected_module_groups,
  );
}

fn get_dynamic_entry_group_ids(
  previous_parent_groups: &HashSet<ModuleGroupId>,
  module_group_graph: &ModuleGroupGraph,
) -> HashSet<ModuleGroupId> {
  previous_parent_groups
    .iter()
    .filter(|group_id| {
      if !module_group_graph.has(group_id) {
        return false;
      }

      let group = module_group_graph.module_group(group_id).unwrap();
      matches!(group.module_group_type, ModuleGroupType::DynamicEntry)
    })
    .cloned()
    .collect::<HashSet<_>>()
}

fn patch_dynamic_entry_group_for_added_dynamic_import(
  dynamic_imported_children: Vec<ModuleId>,
  previous_parent_groups: HashSet<ModuleGroupId>,
  module_graph: &mut ModuleGraph,
  module_group_graph: &mut ModuleGroupGraph,
  affected_module_groups: &mut HashSet<ModuleGroupId>,
  visited: &mut HashSet<ModuleId>,
) {
  let dynamic_entry_group_ids =
    get_dynamic_entry_group_ids(&previous_parent_groups, module_group_graph);

  if !dynamic_entry_group_ids.is_empty() {
    for child in dynamic_imported_children {
      // patch the module group for dynamic entry recursively for dynamic import
      patch_existing_added_non_dynamic_children(
        &child,
        dynamic_entry_group_ids.clone(),
        module_graph,
        module_group_graph,
        affected_module_groups,
        visited,
      );
    }
  }
}

fn patch_dynamic_entry_group_for_removed_dynamic_import(
  removed_module_id: &ModuleId,
  previous_parent_groups: HashSet<ModuleGroupId>,
  module_graph: &mut ModuleGraph,
  module_group_graph: &mut ModuleGroupGraph,
  affected_module_groups: &mut HashSet<ModuleGroupId>,
) {
  let dynamic_entry_group_ids =
    get_dynamic_entry_group_ids(&previous_parent_groups, module_group_graph);

  if !dynamic_entry_group_ids.is_empty() {
    let (result, cyclic) = topo_sort_removed_module(removed_module_id, module_graph, &|_, _| true);

    patch_any_module_group_for_toposort_modules(
      result,
      cyclic,
      dynamic_entry_group_ids,
      module_graph,
      module_group_graph,
      affected_module_groups,
    );
  }
}

fn topo_sort_removed_module<F: Fn(&ModuleId, &&ModuleGraphEdge) -> bool>(
  removed_module_id: &ModuleId,
  module_graph: &mut ModuleGraph,
  filter_fn: &F,
) -> (Vec<ModuleId>, Vec<Vec<ModuleId>>) {
  // we have to ensure that a module is handled only when all of its non dynamic parents are handled
  let mut result = vec![];
  let mut cyclic = vec![];
  module_graph.toposort_dfs(
    removed_module_id,
    &mut vec![],
    &mut HashSet::default(),
    &mut result,
    &mut cyclic,
    filter_fn,
  );
  result.reverse();

  (result, cyclic)
}

fn patch_any_module_group_for_toposort_modules(
  result: Vec<ModuleId>,
  cyclic: Vec<Vec<ModuleId>>,
  previous_parent_groups: HashSet<ModuleGroupId>,
  module_graph: &mut ModuleGraph,
  module_group_graph: &mut ModuleGroupGraph,
  affected_module_groups: &mut HashSet<ModuleGroupId>,
) {
  for current_module_id in result {
    let current_parents = module_graph
      .dependents(&current_module_id)
      .into_iter()
      .map(|(id, edge_info)| (id, edge_info.is_dynamic_import()))
      .collect::<Vec<_>>();

    for module_group_id in &previous_parent_groups {
      // if current parents don't contain previous parent's module group, remove the module from existing module groups
      // Note: current_parents don't contain module_id because the edge is removed
      if current_parents
        .iter()
        .filter(|(_, is_dynamic)| !is_dynamic)
        .all(|(id, _)| {
          let parent = module_graph.module(id).unwrap();
          let parent_contains_group = parent.module_groups.contains(module_group_id);
          !parent_contains_group
            || (parent_contains_group
              && should_ignore_cyclic_dependencies(&cyclic, &id, &current_module_id))
        })
      {
        let module_group = module_group_graph
          .module_group_mut(module_group_id)
          .unwrap_or_else(|| panic!("module group {module_group_id:?} not found"));

        module_group.remove_module(&current_module_id);
        let current_module = module_graph.module_mut(&current_module_id).unwrap();
        affected_module_groups.extend(current_module.module_groups.clone());
        current_module.module_groups.remove(module_group_id);

        let modules_len = module_group.modules().len();

        if modules_len == 0 {
          module_group_graph.remove_module_group(module_group_id);
        } else if !matches!(
          module_group.module_group_type,
          ModuleGroupType::DynamicEntry
        ) {
          // determine if there are edges that should be removed
          let children = module_graph.dependencies(&current_module_id);

          for (child, edge_info) in children {
            let child_group_id = ModuleGroupId::new(&child, &ModuleGroupType::DynamicImport);

            if edge_info.is_dynamic_import()
              && module_group_graph
                .dependencies_ids(module_group_id)
                .contains(&child_group_id)
            {
              let parents = module_graph
                .dependents(&child)
                .into_iter()
                .filter(|(_, edge_info)| edge_info.is_dynamic_import())
                .collect::<Vec<_>>();
              let parents_in_module_group = parents.iter().any(|(id, _)| {
                let parent = module_graph.module(id).unwrap();
                parent.module_groups.contains(module_group_id)
              });

              if !parents_in_module_group {
                module_group_graph.remove_edge(module_group_id, &child_group_id);
              }
            }
          }
        }
      }
    }
  }
}

fn should_ignore_cyclic_dependencies(
  cyclic: &Vec<Vec<ModuleId>>,
  module_id: &ModuleId,
  dep_id: &ModuleId,
) -> bool {
  let stack = cyclic
    .iter()
    .find(|c| c.contains(module_id) && c.contains(dep_id));

  if stack.is_none() {
    return false;
  }

  let stack = stack.unwrap();
  let module_index = stack.iter().position(|c| c == module_id);
  let dep_index = stack.iter().position(|c| c == dep_id);

  if module_index.is_none() || dep_index.is_none() {
    return false;
  }

  // for A <-> B, ignore B -> A
  module_index.unwrap() > dep_index.unwrap()
}

#[cfg(test)]
mod test_dynamic_entries;
#[cfg(test)]
mod test_remove_and_add;
#[cfg(test)]
mod tests;
