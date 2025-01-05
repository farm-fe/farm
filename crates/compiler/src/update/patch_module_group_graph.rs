use std::collections::VecDeque;

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
    for (added_module_id, edge_items) in &deps_diff_result.added {
      if edge_items.iter().any(|item| item.kind.is_dynamic_import()) {
        let added_module_group_id =
          ModuleGroupId::new(added_module_id, &ModuleGroupType::DynamicImport);
        // create new module group only when the module group does not exist
        if module_group_graph.has(&added_module_group_id) {
          continue;
        }
        // if the edge is a dynamic import, we need to create a new module group for this module
        add_dynamic_module_group(
          added_module_id,
          module_id,
          ModuleGroupType::DynamicImport,
          module_graph,
          module_group_graph,
        );
        affected_module_groups.insert(added_module_group_id);
      } else {
        // if the edge is a normal import, we need to add this module to the module group of the parent module
        let previous_parent_groups =
          get_previous_module_groups(module_id, module_graph, removed_modules);
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
          let affected_group_ids = patch_existing_added_non_dynamic_children(
            added_module_id,
            previous_parent_groups,
            module_graph,
            module_group_graph,
          );
          affected_module_groups.extend(affected_group_ids);
        }
      }
    }

    for (removed_module_id, edge_info) in &deps_diff_result.removed {
      if module_graph.has_module(removed_module_id) {
        let previous_parent_groups =
          get_previous_module_groups(module_id, module_graph, removed_modules);
        // a edge is removed, so we need to remove the module from the module group if necessary
        let current_parents = module_graph.dependents(removed_module_id);

        if edge_info.is_dynamic_import() {
          let removed_group_id =
            ModuleGroupId::new(&removed_module_id, &ModuleGroupType::DynamicImport);

          if current_parents
            .iter()
            .filter(|(_, edge_info)| edge_info.is_dynamic_import())
            .count()
            == 0
          {
            // means this module is no longer imported by any dynamic import, and its module group should be removed,
            // as well as all modules inside this module group
            let module_group = module_group_graph
              .module_group_mut(&removed_group_id)
              .unwrap();
            module_group.modules().iter().for_each(|module_id| {
              let module = module_graph.module_mut(module_id).unwrap();
              module.module_groups.remove(&removed_group_id);
              affected_module_groups.extend(module.module_groups.clone());
            });
            // do not need to remove the edge cause it will be removed automatically when the module is removed
            module_group_graph.remove_module_group(&removed_group_id);
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
          let affected_group_ids = patch_existing_removed_non_dynamic_children(
            removed_module_id,
            previous_parent_groups,
            module_graph,
            module_group_graph,
          );
          affected_module_groups.extend(affected_group_ids);
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
  module_graph: &mut ModuleGraph,
  module_group_graph: &mut ModuleGroupGraph,
) {
  let module_group = ModuleGroup::new(added_module_id.clone(), module_group_type);
  let added_module_group_id = module_group.id.clone();
  module_group_graph.add_module_group(module_group);

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

  let module = module_graph.module_mut(added_module_id).unwrap();
  module.module_groups.insert(added_module_group_id);
}

fn patch_existing_added_non_dynamic_children(
  added_module_id: &ModuleId,
  previous_parent_groups: HashSet<ModuleGroupId>,
  module_graph: &mut ModuleGraph,
  module_group_graph: &mut ModuleGroupGraph,
) -> HashSet<ModuleGroupId> {
  let mut queue = VecDeque::from([added_module_id.clone()]);
  let mut visited = HashSet::default();
  let mut affected_module_groups = HashSet::default();

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

      for (child, edge_info) in module_graph.dependencies(&current_module_id) {
        let child_group_id = ModuleGroupId::new(&child, &ModuleGroupType::DynamicImport);

        if edge_info.is_dynamic_import()
          && !module_group_graph.has_edge(module_group_id, &child_group_id)
        {
          module_group_graph.add_edge(module_group_id, &child_group_id);
        }
      }
    }

    if current_module_group_change {
      for (child, edge_info) in module_graph.dependencies(&current_module_id) {
        if !edge_info.is_dynamic_import() {
          queue.push_back(child);
        }
      }
    }
  }

  affected_module_groups
}

fn patch_existing_removed_non_dynamic_children(
  removed_module_id: &ModuleId,
  previous_parent_groups: HashSet<ModuleGroupId>,
  module_graph: &mut ModuleGraph,
  module_group_graph: &mut ModuleGroupGraph,
) -> HashSet<ModuleGroupId> {
  let mut queue = VecDeque::from([removed_module_id.clone()]);
  let mut visited = HashSet::default();
  let mut affected_module_groups = HashSet::default();

  while !queue.is_empty() {
    let current_module_id = queue.pop_front().unwrap();

    if visited.contains(&current_module_id) {
      continue;
    }
    visited.insert(current_module_id.clone());

    let current_parents = module_graph
      .dependents(&current_module_id)
      .into_iter()
      .map(|(id, edge_info)| (id, edge_info.is_dynamic_import()))
      .collect::<Vec<_>>();
    let mut current_module_group_change = false;

    for module_group_id in &previous_parent_groups {
      // if current parents don't contain previous parent's module group, remove the module from existing module groups
      // Note: current_parents don't contain module_id because the edge is removed
      if current_parents
        .iter()
        .filter(|(_, is_dynamic)| !is_dynamic)
        .all(|(id, _)| {
          let parent = module_graph.module(id).unwrap();
          !parent.module_groups.contains(module_group_id)
        })
      {
        current_module_group_change = true;
        let module_group = module_group_graph
          .module_group_mut(module_group_id)
          .unwrap();

        module_group.remove_module(&current_module_id);
        let current_module = module_graph.module_mut(&current_module_id).unwrap();
        affected_module_groups.extend(current_module.module_groups.clone());
        current_module.module_groups.remove(module_group_id);

        let modules_len = module_group.modules().len();

        if modules_len == 0 {
          module_group_graph.remove_module_group(module_group_id);
        } else {
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

    if current_module_group_change {
      for (child, edge_info) in module_graph.dependencies(&current_module_id) {
        if !edge_info.is_dynamic_import() {
          queue.push_back(child);
        }
      }
    }
  }

  affected_module_groups
}

#[cfg(test)]
mod tests;
