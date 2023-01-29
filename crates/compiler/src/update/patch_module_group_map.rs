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
            });
            drop(module_group);
            module_group_map.remove_module_group(removed_module_id);
          }
        } else {
          for module_group_id in &previous_parent_groups {
            if current_parents
              .iter()
              .filter(|(_, kind, _)| kind != &ResolveKind::DynamicImport)
              .all(|(id, _, _)| {
                let parent = module_graph.module(id).unwrap();
                !parent.module_groups.contains(module_group_id)
              })
            {
              let module_group = module_group_map.module_group_mut(module_group_id).unwrap();

              module_group.remove_module(removed_module_id);
              affected_module_groups.insert(module_group_id.clone());

              let removed_module = module_graph.module_mut(removed_module_id).unwrap();
              removed_module.module_groups.remove(module_group_id);
            }
          }
        }
      } else {
        // this module is removed, all module groups that contains this module should remove this module
        let removed_module = removed_modules.get(removed_module_id).unwrap();
        for removed_module_group_id in &removed_module.module_groups {
          let module_group = module_group_map
            .module_group_mut(removed_module_group_id)
            .unwrap();

          module_group.remove_module(removed_module_id);
          affected_module_groups.insert(removed_module_group_id.clone());
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
        for module_group_id in &previous_parent_groups {
          let module_group = module_group_map.module_group_mut(module_group_id).unwrap();
          module_group.add_module(added_module_id.clone());
          affected_module_groups.insert(module_group_id.clone());
        }
        let module = module_graph.module_mut(added_module_id).unwrap();
        module.module_groups.extend(previous_parent_groups);
      }
    }
  }

  affected_module_groups
}

#[cfg(test)]
mod tests {
  use farmfe_core::module::Module;
  use farmfe_plugin_partial_bundling::module_group_map_from_entries;
  use farmfe_testing_helpers::construct_test_module_graph;

  use crate::update::diff_and_patch_module_graph::{diff_module_graph, patch_module_graph};

  use super::patch_module_group_map;

  #[test]
  fn test_patch_module_group_map_1() {
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

    patch_module_group_map(
      start_points.clone(),
      &diff_result,
      &removed_modules,
      &mut module_graph,
      &mut module_group_map,
    );

    let update_module_group_map = module_group_map_from_entries(&start_points, &mut module_graph);

    assert_eq!(module_group_map, update_module_group_map);
  }
}
