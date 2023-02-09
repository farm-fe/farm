use std::collections::VecDeque;

use farmfe_core::{
  hashbrown::{HashMap, HashSet},
  module::{
    module_graph::ModuleGraph,
    module_group::{ModuleGroup, ModuleGroupMap},
    Module, ModuleId,
  },
  plugin::ResolveKind,
};

use super::diff_and_patch_module_graph::DiffResult;

pub fn patch_module_group_map(
  updated_module_ids: Vec<ModuleId>,
  diff_result: &DiffResult,
  removed_modules: &HashMap<ModuleId, Module>,
  module_graph: &mut ModuleGraph,
  module_group_map: &mut ModuleGroupMap,
) -> HashSet<ModuleId> {
  let mut affected_module_groups = HashSet::new();

  for updated_module_id in &updated_module_ids {
    let module = module_graph.module(updated_module_id).unwrap();
    let module_group_ids = module.module_groups.clone();
    affected_module_groups.extend(module_group_ids);
  }

  let deps_changes = &diff_result.deps_changes;

  let get_previous_module_groups = |module_id: &ModuleId, module_graph: &mut ModuleGraph| {
    if module_graph.has_module(module_id) {
      let module = module_graph.module(module_id).unwrap();
      module.module_groups.clone()
    } else {
      let removed_module = removed_modules.get(module_id).unwrap();
      removed_module.module_groups.clone()
    }
  };

  for (module_id, deps_diff_result) in deps_changes {
    for (removed_module_id, edge_info) in &deps_diff_result.removed {
      if module_graph.has_module(removed_module_id) {
        // a edge is removed, so we need to remove the module from the module group if necessary
        let current_parents = module_graph.dependents(removed_module_id);
        let previous_parent_groups = get_previous_module_groups(module_id, module_graph);

        if edge_info.kind == ResolveKind::DynamicImport {
          if current_parents
            .iter()
            .filter(|(_, kind, _)| kind == &ResolveKind::DynamicImport)
            .count()
            == 0
          {
            // means this module is no longer imported by any dynamic import, and its module group should be removed,
            // as well as all modules inside this module group
            let module_group = module_group_map
              .module_group_mut(removed_module_id)
              .unwrap();
            module_group.modules().iter().for_each(|module_id| {
              let module = module_graph.module_mut(module_id).unwrap();
              module.module_groups.remove(removed_module_id);
              affected_module_groups.extend(module.module_groups.clone());
            });

            module_group_map.remove_module_group(removed_module_id);
          }
        } else {
          let mut queue = VecDeque::from([removed_module_id.clone()]);

          while !queue.is_empty() {
            let current_module_id = queue.pop_front().unwrap();
            let current_parents = module_graph.dependents(&current_module_id);
            let mut current_module_group_change = false;

            for module_group_id in &previous_parent_groups {
              // if current parents don't contain previous parent's module group, remove the module from existing module groups
              // Note: current_parents don't contain module_id because the edge is removed
              if current_parents
                .iter()
                .filter(|(_, kind, _)| kind != &ResolveKind::DynamicImport)
                .all(|(id, _, _)| {
                  let parent = module_graph.module(id).unwrap();
                  !parent.module_groups.contains(module_group_id)
                })
              {
                current_module_group_change = true;
                let module_group = module_group_map.module_group_mut(module_group_id).unwrap();

                module_group.remove_module(&current_module_id);

                let modules_len = module_group.modules().len();

                if modules_len == 0 {
                  module_group_map.remove_module_group(module_group_id);
                }

                let current_module = module_graph.module_mut(&current_module_id).unwrap();
                affected_module_groups.extend(current_module.module_groups.clone());
                current_module.module_groups.remove(module_group_id);
              }
            }

            if current_module_group_change {
              for (child, kind, _) in module_graph.dependencies(&current_module_id) {
                if kind != ResolveKind::DynamicImport {
                  queue.push_back(child);
                }
              }
            }
          }
        }
      } else {
        // this module is removed, all module groups that contains this module should remove this module
        let removed_module = removed_modules.get(removed_module_id).unwrap();
        for removed_module_group_id in &removed_module.module_groups {
          let module_group = module_group_map.module_group_mut(removed_module_group_id);

          if let Some(module_group) = module_group {
            module_group.remove_module(removed_module_id);
            affected_module_groups.insert(removed_module_group_id.clone());
            let modules_len = module_group.modules().len();

            if modules_len == 0 {
              module_group_map.remove_module_group(removed_module_group_id);
            }
          }
        }
      }
    }

    for (added_module_id, edge_info) in &deps_diff_result.added {
      if edge_info.kind == ResolveKind::DynamicImport {
        // create new module group only when the module group does not exist
        if module_group_map.has(added_module_id) {
          continue;
        }
        // if the edge is a dynamic import, we need to create a new module group for this module
        let module_group_id = added_module_id.clone();
        let module_group = ModuleGroup::new(module_group_id.clone());
        module_group_map.add_module_group(module_group);
        let module = module_graph.module_mut(added_module_id).unwrap();
        module.module_groups.insert(module_group_id);
      } else {
        // if the edge is a normal import, we need to add this module to the module group of the parent module
        let previous_parent_groups = get_previous_module_groups(module_id, module_graph);
        // new module
        if diff_result.added_modules.contains(added_module_id) {
          for module_group_id in &previous_parent_groups {
            let module_group = module_group_map.module_group_mut(module_group_id).unwrap();
            module_group.add_module(added_module_id.clone());
            affected_module_groups.insert(module_group_id.clone());
          }
          let module = module_graph.module_mut(added_module_id).unwrap();
          module.module_groups.extend(previous_parent_groups);
        } else {
          // also need to handle all of its non-dynamic children
          let mut queue = VecDeque::from([added_module_id.clone()]);

          while !queue.is_empty() {
            let current_module_id = queue.pop_front().unwrap();

            for module_group_id in &previous_parent_groups {
              let module_group = module_group_map.module_group_mut(module_group_id).unwrap();

              module_group.add_module(current_module_id.clone());
              affected_module_groups.insert(module_group_id.clone());
            }

            let current_module = module_graph.module_mut(&current_module_id).unwrap();
            current_module
              .module_groups
              .extend(previous_parent_groups.clone());

            for (child_id, kind, _) in module_graph.dependencies(&current_module_id) {
              if kind != ResolveKind::DynamicImport {
                queue.push_back(child_id.clone());
              }
            }
          }
        }
      }
    }
  }

  affected_module_groups
    .into_iter()
    .filter(|g_id| module_group_map.has(g_id))
    .collect()
}

#[cfg(test)]
mod tests {
  use farmfe_core::{hashbrown::HashSet, module::Module};
  use farmfe_plugin_partial_bundling::module_group_map_from_entries;
  use farmfe_testing_helpers::construct_test_module_graph;

  use crate::update::diff_and_patch_module_graph::{diff_module_graph, patch_module_graph};

  use super::patch_module_group_map;

  #[test]
  fn test_patch_module_group_map_1() {
    let mut module_graph = construct_test_module_graph();
    let mut update_module_graph = construct_test_module_graph();

    update_module_graph
      .remove_edge(&"A".into(), &"D".into())
      .unwrap();

    let start_points = vec!["A".into(), "B".into()];
    let mut module_group_map = module_group_map_from_entries(&start_points, &mut module_graph);

    let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);
    let removed_modules = patch_module_graph(
      start_points.clone(),
      &diff_result,
      &mut module_graph,
      &mut update_module_graph,
    );

    let affected_groups = patch_module_group_map(
      start_points.clone(),
      &diff_result,
      &removed_modules,
      &mut module_graph,
      &mut module_group_map,
    );
    assert_eq!(
      affected_groups,
      HashSet::from(["A".into(), "B".into(), "F".into()])
    );

    let update_module_group_map = module_group_map_from_entries(&start_points, &mut module_graph);

    assert_eq!(module_group_map, update_module_group_map);
  }

  #[test]
  fn test_patch_module_group_map_2() {
    let mut module_graph = construct_test_module_graph();
    let mut update_module_graph = construct_test_module_graph();

    update_module_graph.remove_module(&"D".into());
    update_module_graph.remove_module(&"E".into());
    update_module_graph.remove_module(&"G".into());
    update_module_graph.add_module(Module::new("H".into()));
    update_module_graph
      .add_edge(&"B".into(), &"H".into(), Default::default())
      .unwrap();
    update_module_graph
      .add_edge(&"H".into(), &"F".into(), Default::default())
      .unwrap();

    let start_points = vec!["B".into(), "A".into()];

    let mut module_group_map = module_group_map_from_entries(&start_points, &mut module_graph);

    let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);
    let removed_modules = patch_module_graph(
      start_points.clone(),
      &diff_result,
      &mut module_graph,
      &mut update_module_graph,
    );

    let affected_groups = patch_module_group_map(
      start_points.clone(),
      &diff_result,
      &removed_modules,
      &mut module_graph,
      &mut module_group_map,
    );
    assert_eq!(
      affected_groups,
      HashSet::from(["A".into(), "B".into(), "F".into()])
    );

    let update_module_group_map = module_group_map_from_entries(&start_points, &mut module_graph);

    assert_eq!(module_group_map, update_module_group_map);
  }

  #[test]
  fn test_patch_module_group_map_3() {
    let mut module_graph = construct_test_module_graph();
    let mut update_module_graph = construct_test_module_graph();

    update_module_graph
      .remove_edge(&"F".into(), &"A".into())
      .unwrap();
    update_module_graph.add_module(Module::new("H".into()));
    update_module_graph
      .add_edge(&"B".into(), &"H".into(), Default::default())
      .unwrap();
    update_module_graph
      .add_edge(&"H".into(), &"F".into(), Default::default())
      .unwrap();

    let start_points = vec!["F".into(), "B".into()];
    let mut module_group_map = module_group_map_from_entries(
      &module_graph.entries.clone().into_iter().collect(),
      &mut module_graph,
    );
    let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);

    let removed_modules = patch_module_graph(
      start_points.clone(),
      &diff_result,
      &mut module_graph,
      &mut update_module_graph,
    );

    let affected_groups = patch_module_group_map(
      start_points.clone(),
      &diff_result,
      &removed_modules,
      &mut module_graph,
      &mut module_group_map,
    );
    assert_eq!(
      affected_groups,
      HashSet::from(["A".into(), "B".into(), "F".into()])
    );

    let update_module_group_map = module_group_map_from_entries(
      &module_graph.entries.clone().into_iter().collect(),
      &mut module_graph,
    );

    assert_eq!(module_group_map, update_module_group_map);
  }
}
