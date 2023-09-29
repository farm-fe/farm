use std::collections::VecDeque;

use farmfe_core::{
  hashbrown::{HashMap, HashSet},
  module::{
    module_graph::ModuleGraph,
    module_group::{ModuleGroup, ModuleGroupGraph},
    Module, ModuleId,
  },
};

use super::diff_and_patch_module_graph::DiffResult;

pub fn patch_module_group_graph(
  updated_module_ids: Vec<ModuleId>,
  diff_result: &DiffResult,
  removed_modules: &HashMap<ModuleId, Module>,
  module_graph: &mut ModuleGraph,
  module_group_graph: &mut ModuleGroupGraph,
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
        let previous_parent_groups = get_previous_module_groups(module_id, module_graph);
        // a edge is removed, so we need to remove the module from the module group if necessary
        let current_parents = module_graph.dependents(removed_module_id);

        if edge_info.is_dynamic() {
          if current_parents
            .iter()
            .filter(|(_, edge_info)| edge_info.is_dynamic())
            .count()
            == 0
          {
            // means this module is no longer imported by any dynamic import, and its module group should be removed,
            // as well as all modules inside this module group
            let module_group = module_group_graph
              .module_group_mut(removed_module_id)
              .unwrap();
            module_group.modules().iter().for_each(|module_id| {
              let module = module_graph.module_mut(module_id).unwrap();
              module.module_groups.remove(removed_module_id);
              affected_module_groups.extend(module.module_groups.clone());
            });
            // do not need to remove the edge cause it will be removed automatically when the module is removed
            module_group_graph.remove_module_group(removed_module_id);
          } else {
            let module_group_ids = {
              if removed_modules.contains_key(module_id) {
                let removed_module = removed_modules.get(module_id).unwrap();
                removed_module.module_groups.clone()
              } else {
                let module = module_graph
                  .module(module_id)
                  .unwrap_or_else(|| panic!("module {:?} not found", module_id));
                module.module_groups.clone()
              }
            };
            // remove the edge
            for module_group_id in &module_group_ids {
              if current_parents
                .iter()
                .filter(|(p, edge_info)| {
                  if edge_info.is_dynamic() {
                    let parent = module_graph.module(p).unwrap();
                    return parent.module_groups.contains(module_group_id);
                  }

                  false
                })
                .count()
                == 0
                && module_group_graph.has_edge(module_group_id, removed_module_id)
              {
                module_group_graph.remove_edge(module_group_id, removed_module_id);
              }
            }
          }
        } else {
          let mut queue = VecDeque::from([removed_module_id.clone()]);

          while !queue.is_empty() {
            let current_module_id = queue.pop_front().unwrap();
            let current_parents = module_graph
              .dependents(&current_module_id)
              .into_iter()
              .map(|(id, edge_info)| (id, edge_info.is_dynamic()))
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
                    if edge_info.is_dynamic()
                      && module_group_graph
                        .dependencies_ids(module_group_id)
                        .contains(&child)
                    {
                      let parents = module_graph
                        .dependents(&child)
                        .into_iter()
                        .filter(|(_, edge_info)| edge_info.is_dynamic())
                        .collect::<Vec<_>>();
                      let parents_in_module_group = parents.iter().any(|(id, _)| {
                        let parent = module_graph.module(id).unwrap();
                        parent.module_groups.contains(module_group_id)
                      });

                      if !parents_in_module_group {
                        module_group_graph.remove_edge(module_group_id, &child);
                      }
                    }
                  }
                }
              }
            }

            if current_module_group_change {
              for (child, edge_info) in module_graph.dependencies(&current_module_id) {
                if !edge_info.is_dynamic() {
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

    for (added_module_id, edge_items) in &deps_diff_result.added {
      if edge_items.iter().any(|item| item.kind.is_dynamic()) {
        // create new module group only when the module group does not exist
        if module_group_graph.has(added_module_id) {
          continue;
        }
        // if the edge is a dynamic import, we need to create a new module group for this module
        let module_group_id = added_module_id.clone();
        let module_group = ModuleGroup::new(module_group_id.clone());
        module_group_graph.add_module_group(module_group);
        affected_module_groups.insert(module_group_id.clone());

        let module_group_ids = {
          let module = module_graph
            .module(module_id)
            .unwrap_or_else(|| panic!("module {:?} not found", module_id));
          module.module_groups.clone()
        };

        for module_group_id in &module_group_ids {
          if !module_group_graph.has_edge(module_group_id, added_module_id) {
            module_group_graph.add_edge(module_group_id, added_module_id);
          }
        }

        let module = module_graph.module_mut(added_module_id).unwrap();
        module.module_groups.insert(module_group_id);
      } else {
        // if the edge is a normal import, we need to add this module to the module group of the parent module
        let previous_parent_groups = get_previous_module_groups(module_id, module_graph);
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
          let mut queue = VecDeque::from([added_module_id.clone()]);

          while !queue.is_empty() {
            let current_module_id = queue.pop_front().unwrap();
            let mut current_module_group_change = false;

            for module_group_id in &previous_parent_groups {
              let current_module = module_graph.module(&current_module_id).unwrap();

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
                if edge_info.is_dynamic() && !module_group_graph.has_edge(module_group_id, &child) {
                  module_group_graph.add_edge(module_group_id, &child);
                }
              }
            }

            if current_module_group_change {
              for (child, edge_info) in module_graph.dependencies(&current_module_id) {
                if !edge_info.is_dynamic() {
                  queue.push_back(child);
                }
              }
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

  let mut final_affected_module_groups = HashSet::new();
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

#[cfg(test)]
mod tests {
  use farmfe_core::{
    hashbrown::HashSet,
    module::{
      module_graph::{ModuleGraphEdge, ModuleGraphEdgeDataItem},
      Module,
    },
    plugin::ResolveKind,
  };
  use farmfe_plugin_partial_bundling::module_group_graph_from_entries;
  use farmfe_testing_helpers::construct_test_module_graph;

  use crate::update::diff_and_patch_module_graph::{diff_module_graph, patch_module_graph};

  use super::patch_module_group_graph;

  #[test]
  fn test_patch_module_group_graph_1() {
    let mut module_graph = construct_test_module_graph();
    let mut update_module_graph = construct_test_module_graph();
    update_module_graph.remove_module(&"F".into());
    update_module_graph.remove_module(&"G".into());
    update_module_graph
      .remove_edge(&"A".into(), &"D".into())
      .unwrap();
    let entries = vec!["A".into(), "B".into()];
    let start_points = vec!["A".into(), "C".into(), "D".into(), "E".into()];
    let mut module_group_graph = module_group_graph_from_entries(&entries, &mut module_graph);

    let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);
    let removed_modules = patch_module_graph(
      start_points.clone(),
      &diff_result,
      &mut module_graph,
      &mut update_module_graph,
    );

    let affected_groups = patch_module_group_graph(
      start_points.clone(),
      &diff_result,
      &removed_modules,
      &mut module_graph,
      &mut module_group_graph,
    );
    assert_eq!(affected_groups, HashSet::from(["A".into(), "B".into()]));

    let update_module_group_graph = module_group_graph_from_entries(&entries, &mut module_graph);

    assert_eq!(module_group_graph, update_module_group_graph);

    // makes sure that module_groups field of each module is correct
    let module_a = module_graph.module(&"A".into()).unwrap();
    assert_eq!(module_a.module_groups, HashSet::from(["A".into()]));
    let module_b = module_graph.module(&"B".into()).unwrap();
    assert_eq!(module_b.module_groups, HashSet::from(["B".into()]));
    let module_c = module_graph.module(&"C".into()).unwrap();
    assert_eq!(module_c.module_groups, HashSet::from(["A".into()]));
    let module_d = module_graph.module(&"D".into()).unwrap();
    assert_eq!(module_d.module_groups, HashSet::from(["B".into()]));
    let module_e = module_graph.module(&"E".into()).unwrap();
    assert_eq!(module_e.module_groups, HashSet::from(["B".into()]));
  }

  #[test]
  fn test_patch_module_group_graph_2() {
    let mut module_graph = construct_test_module_graph();
    let mut update_module_graph = construct_test_module_graph();

    update_module_graph.remove_module(&"D".into());
    update_module_graph.remove_module(&"E".into());
    update_module_graph.remove_module(&"G".into());
    update_module_graph
      .remove_edge(&"C".into(), &"F".into())
      .unwrap();
    update_module_graph.add_module(Module::new("H".into()));
    update_module_graph
      .add_edge(&"B".into(), &"H".into(), Default::default())
      .unwrap();
    update_module_graph
      .add_edge(&"H".into(), &"F".into(), Default::default())
      .unwrap();

    let start_points = vec!["B".into(), "A".into()];
    let updated_modules = vec!["B".into(), "A".into()];

    let mut module_group_graph = module_group_graph_from_entries(&start_points, &mut module_graph);

    let diff_result =
      diff_module_graph(updated_modules.clone(), &module_graph, &update_module_graph);
    let removed_modules = patch_module_graph(
      updated_modules.clone(),
      &diff_result,
      &mut module_graph,
      &mut update_module_graph,
    );

    let affected_groups = patch_module_group_graph(
      updated_modules.clone(),
      &diff_result,
      &removed_modules,
      &mut module_graph,
      &mut module_group_graph,
    );
    assert_eq!(
      affected_groups,
      HashSet::from(["A".into(), "B".into(), "F".into()])
    );
    let module_group_b = module_group_graph.module_group(&"B".into()).unwrap();
    assert_eq!(
      module_group_b.modules(),
      &HashSet::from(["B".into(), "H".into(), "F".into(), "C".into(), "A".into()])
    );

    let update_module_group_graph =
      module_group_graph_from_entries(&start_points, &mut module_graph);

    assert_eq!(module_group_graph, update_module_group_graph);

    // makes sure that module_groups field of each module is correct
    let module_a = module_graph.module(&"A".into()).unwrap();
    assert_eq!(
      module_a.module_groups,
      HashSet::from(["A".into(), "F".into(), "B".into()])
    );
    let module_b = module_graph.module(&"B".into()).unwrap();
    assert_eq!(module_b.module_groups, HashSet::from(["B".into()]));
    let module_c = module_graph.module(&"C".into()).unwrap();
    assert_eq!(
      module_c.module_groups,
      HashSet::from(["A".into(), "F".into(), "B".into()])
    );
    let module_f = module_graph.module(&"F".into()).unwrap();
    assert_eq!(
      module_f.module_groups,
      HashSet::from(["B".into(), "F".into()])
    );
    let module_h = module_graph.module(&"H".into()).unwrap();
    assert_eq!(module_h.module_groups, HashSet::from(["B".into()]));
  }

  #[test]
  fn test_patch_module_group_graph_3() {
    let mut module_graph = construct_test_module_graph();
    let mut update_module_graph = construct_test_module_graph();
    update_module_graph.remove_module(&"G".into());
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

    let updated_modules = vec!["F".into(), "E".into(), "B".into()];
    let mut module_group_graph = module_group_graph_from_entries(
      &module_graph
        .entries
        .clone()
        .into_iter()
        .map(|(entry, _)| entry)
        .collect(),
      &mut module_graph,
    );
    let diff_result =
      diff_module_graph(updated_modules.clone(), &module_graph, &update_module_graph);

    let removed_modules = patch_module_graph(
      updated_modules.clone(),
      &diff_result,
      &mut module_graph,
      &mut update_module_graph,
    );

    let affected_groups = patch_module_group_graph(
      updated_modules,
      &diff_result,
      &removed_modules,
      &mut module_graph,
      &mut module_group_graph,
    );
    assert_eq!(
      affected_groups,
      HashSet::from(["A".into(), "B".into(), "F".into(), "D".into()])
    );

    let update_module_group_graph = module_group_graph_from_entries(
      &module_graph
        .entries
        .clone()
        .into_iter()
        .map(|(entry, _)| entry)
        .collect(),
      &mut module_graph,
    );

    assert_eq!(module_group_graph, update_module_group_graph);

    // makes sure that module_groups field of each module is correct
    let module_a = module_graph.module(&"A".into()).unwrap();
    assert_eq!(module_a.module_groups, HashSet::from(["A".into()]));
    let module_b = module_graph.module(&"B".into()).unwrap();
    assert_eq!(module_b.module_groups, HashSet::from(["B".into()]));
    let module_c = module_graph.module(&"C".into()).unwrap();
    assert_eq!(module_c.module_groups, HashSet::from(["A".into()]));
    let module_d = module_graph.module(&"D".into()).unwrap();
    assert_eq!(
      module_d.module_groups,
      HashSet::from(["B".into(), "D".into()])
    );
    let module_e = module_graph.module(&"E".into()).unwrap();
    assert_eq!(module_e.module_groups, HashSet::from(["B".into()]));
    let module_f = module_graph.module(&"F".into()).unwrap();
    assert_eq!(
      module_f.module_groups,
      HashSet::from(["F".into(), "B".into()])
    );
    let module_h = module_graph.module(&"H".into()).unwrap();
    assert_eq!(module_h.module_groups, HashSet::from(["B".into()]));
  }

  fn get_edge_info(kind: ResolveKind) -> ModuleGraphEdge {
    ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
      kind,
      ..Default::default()
    }])
  }

  #[test]
  fn test_patch_module_group_graph_css_modules() {
    let mut module_graph = construct_test_module_graph();
    module_graph.add_module(Module::new("I.module.css".into()));
    module_graph.add_module(Module::new("I.module.css.FARM_CSS_MODULES?1".into()));
    module_graph
      .add_edge(
        &"D".into(),
        &"I.module.css".into(),
        get_edge_info(ResolveKind::Import),
      )
      .unwrap();
    module_graph
      .add_edge(
        &"I.module.css".into(),
        &"I.module.css.FARM_CSS_MODULES?1".into(),
        get_edge_info(ResolveKind::Import),
      )
      .unwrap();

    let mut update_module_graph = construct_test_module_graph();
    update_module_graph.remove_module(&"A".into());
    update_module_graph.remove_module(&"C".into());
    update_module_graph.remove_module(&"B".into());
    update_module_graph.remove_module(&"E".into());
    update_module_graph.remove_module(&"G".into());

    update_module_graph.add_module(Module::new("I.module.css".into()));
    update_module_graph.add_module(Module::new("H".into()));
    update_module_graph
      .add_edge(&"D".into(), &"H".into(), get_edge_info(ResolveKind::Import))
      .unwrap();
    update_module_graph
      .add_edge(
        &"H".into(),
        &"I.module.css".into(),
        get_edge_info(ResolveKind::Import),
      )
      .unwrap();

    let start_points = vec!["D".into()];
    let mut module_group_graph = module_group_graph_from_entries(
      &module_graph
        .entries
        .clone()
        .into_iter()
        .map(|(entry, _)| entry)
        .collect(),
      &mut module_graph,
    );
    let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);
    let removed_modules = patch_module_graph(
      start_points.clone(),
      &diff_result,
      &mut module_graph,
      &mut update_module_graph,
    );

    let affected_groups = patch_module_group_graph(
      start_points,
      &diff_result,
      &removed_modules,
      &mut module_graph,
      &mut module_group_graph,
    );
    assert_eq!(affected_groups, HashSet::from(["D".into(), "B".into()]));

    let update_module_group_graph = module_group_graph_from_entries(
      &module_graph
        .entries
        .clone()
        .into_iter()
        .map(|(entry, _)| entry)
        .collect(),
      &mut module_graph,
    );

    assert_eq!(module_group_graph, update_module_group_graph);
  }
}
