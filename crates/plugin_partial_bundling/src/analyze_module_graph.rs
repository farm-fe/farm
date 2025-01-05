use farmfe_core::module::module_group::{ModuleGroupId, ModuleGroupType};
use farmfe_core::HashMap;
use farmfe_core::{
  module::{
    module_graph::ModuleGraph,
    module_group::{ModuleGroup, ModuleGroupGraph},
    ModuleId,
  },
  HashSet,
};
use std::collections::VecDeque;

pub fn module_group_graph_from_module_graph(module_graph: &mut ModuleGraph) -> ModuleGroupGraph {
  let get_entries = |entries: &HashMap<ModuleId, String>| {
    entries
      .clone()
      .into_iter()
      .map(|(entry, _)| entry)
      .collect()
  };

  let mut module_group_graph =
    module_group_graph_from_entries(&get_entries(&module_graph.entries), module_graph);
  // handle dynamic entries
  let dynamic_module_groups =
    module_groups_from_dynamic_entries(get_entries(&module_graph.dynamic_entries), module_graph);
  for group in dynamic_module_groups {
    module_group_graph.add_module_group(group);
  }

  module_group_graph
}

pub fn module_group_graph_from_entries(
  entries: &Vec<ModuleId>,
  module_graph: &mut ModuleGraph,
) -> ModuleGroupGraph {
  let mut module_group_graph = ModuleGroupGraph::new();
  let mut edges = vec![];
  let mut visited = HashSet::default();

  for entry in entries.clone() {
    if visited.contains(&entry) {
      continue;
    }

    let (group, dynamic_dependencies) =
      module_group_from_entry(&entry, ModuleGroupType::Entry, module_graph);
    edges.extend(dynamic_dependencies.clone().into_iter().map(|dep| {
      (
        group.id.clone(),
        ModuleGroupId::new(&dep, &ModuleGroupType::DynamicImport),
      )
    }));

    module_group_graph.add_module_group(group);

    visited.insert(entry);
    let mut queue = VecDeque::from(dynamic_dependencies);

    while !queue.is_empty() {
      let head = queue.pop_front().unwrap();

      if visited.contains(&head) {
        continue;
      }

      visited.insert(head.clone());

      let (group, dynamic_dependencies) =
        module_group_from_entry(&head, ModuleGroupType::DynamicImport, module_graph);
      edges.extend(dynamic_dependencies.clone().into_iter().map(|dep| {
        (
          group.id.clone(),
          ModuleGroupId::new(&dep, &ModuleGroupType::DynamicImport),
        )
      }));

      module_group_graph.add_module_group(group);
      queue.extend(dynamic_dependencies);
    }
  }

  for (from, to) in &edges {
    module_group_graph.add_edge(from, to);
  }

  module_group_graph
}

/// get module group start from a entry. return (module group, dynamic dependencies)
/// traverse the module graph using bfs, stop when reach a dynamic dependency
fn module_group_from_entry(
  entry: &ModuleId,
  module_group_type: ModuleGroupType,
  graph: &mut ModuleGraph,
) -> (ModuleGroup, Vec<ModuleId>) {
  let mut visited = HashSet::default();
  // ModuleGroupType
  let mut module_group = ModuleGroup::new(entry.clone(), module_group_type);
  let mut dynamic_entries = vec![];

  graph
    .module_mut(entry)
    .unwrap()
    .module_groups
    .insert(module_group.id.clone());

  visited.insert(entry.clone());

  let deps = graph
    .dependencies(entry)
    .into_iter()
    .map(|(k, v)| (k, v.is_dynamic_import(), v.is_dynamic_entry()))
    .collect::<Vec<_>>();

  for (dep, is_dynamic_import, is_dynamic_entry) in deps {
    if is_dynamic_import {
      if !dynamic_entries.contains(&dep) {
        dynamic_entries.push(dep);
      }
    } else if !is_dynamic_entry {
      // visited all dep and its dependencies using BFS
      let mut queue = VecDeque::new();
      queue.push_back(dep.clone());

      while !queue.is_empty() {
        let head = queue.pop_front().unwrap();

        if visited.contains(&head) {
          continue;
        }

        visited.insert(head.clone());
        module_group.add_module(head.clone());
        graph
          .module_mut(&head)
          .unwrap()
          .module_groups
          .insert(module_group.id.clone());

        for (dep, edge) in graph.dependencies(&head) {
          if edge.is_dynamic_import() {
            if !dynamic_entries.contains(&dep) {
              dynamic_entries.push(dep);
            }
          } else if !edge.is_dynamic_entry() {
            queue.push_back(dep);
          }
        }
      }
    }
  }

  (module_group, dynamic_entries)
}

pub fn module_groups_from_dynamic_entries(
  dynamic_entries: Vec<ModuleId>,
  module_graph: &mut ModuleGraph,
) -> Vec<ModuleGroup> {
  let mut module_groups = vec![];

  for entry in dynamic_entries {
    let mut group = ModuleGroup::new(entry.clone(), ModuleGroupType::DynamicEntry);
    // find al deep dependencies of dynamic entries using BFS
    let mut queue = VecDeque::from([entry.clone()]);
    let mut visited = HashSet::default();

    while !queue.is_empty() {
      let head = queue.pop_front().unwrap();

      if visited.contains(&head) {
        continue;
      }

      visited.insert(head.clone());
      group.add_module(head.clone());
      module_graph
        .module_mut(&head)
        .unwrap_or_else(|| panic!("module {:?} not found", head))
        .module_groups
        .insert(group.id.clone());

      queue.extend(module_graph.dependencies_ids(&head));
    }

    module_groups.push(group.clone());
  }

  module_groups
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use farmfe_core::{
    context::CompilationContext,
    module::module_group::{ModuleGroupId, ModuleGroupType},
    parking_lot::RwLock,
    plugin::{Plugin, PluginHookContext, ResolveKind},
    HashMap, HashSet,
  };
  #[cfg(test)]
  use farmfe_testing_helpers::construct_test_module_graph;
  use farmfe_testing_helpers::{
    assert_debug_snapshot, construct_test_module_graph_complex, construct_test_module_group_graph,
  };

  use crate::FarmPluginPartialBundling;

  use super::module_group_from_entry as mgfe;

  #[test]
  fn analyze_module_graph() {
    let plugin = FarmPluginPartialBundling {
      partial_bundling_groups_enforce_map: Default::default(),
    };
    let mut context = CompilationContext::new(Default::default(), vec![]).unwrap();
    let graph = construct_test_module_graph();

    let _ = std::mem::replace(&mut context.module_graph, Box::new(RwLock::new(graph)));
    let context = Arc::new(context);
    let mut module_graph = context.module_graph.write();

    let module_group_graph = plugin
      .analyze_module_graph(
        &mut module_graph,
        &context,
        &PluginHookContext {
          caller: None,
          meta: HashMap::default(),
        },
      )
      .unwrap()
      .unwrap();

    assert_eq!(module_group_graph.len(), 5);
    let group_id_a = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
    let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);
    let group_id_d = ModuleGroupId::new(&"D".into(), &ModuleGroupType::DynamicImport);
    let group_id_f = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);
    let group_id_g = ModuleGroupId::new(&"G".into(), &ModuleGroupType::DynamicImport);
    assert!(module_group_graph.has(&group_id_a));
    assert!(module_group_graph.has(&group_id_b));
    assert!(module_group_graph.has(&group_id_d));
    assert!(module_group_graph.has(&group_id_f));
    assert!(module_group_graph.has(&group_id_g));

    let module_group_a = module_group_graph.module_group(&group_id_a).unwrap();
    assert_eq!(module_group_a.entry_module_id, "A".into());
    assert_eq!(
      module_group_a.modules(),
      &HashSet::from_iter(["A".into(), "C".into()])
    );

    let module_group_b = module_group_graph.module_group(&group_id_b).unwrap();
    assert_eq!(module_group_b.entry_module_id, "B".into());
    assert_eq!(
      module_group_b.modules(),
      &HashSet::from_iter(["B".into(), "D".into(), "E".into()])
    );

    let module_group_d = module_group_graph.module_group(&group_id_d).unwrap();
    assert_eq!(module_group_d.entry_module_id, "D".into());
    assert_eq!(module_group_d.modules(), &HashSet::from_iter(["D".into()]));

    let module_group_f = module_group_graph.module_group(&group_id_f).unwrap();
    assert_eq!(module_group_f.entry_module_id, "F".into());
    assert_eq!(
      module_group_f.modules(),
      &HashSet::from_iter(["F".into(), "A".into(), "C".into()])
    );

    let module_group_g = module_group_graph.module_group(&group_id_g).unwrap();
    assert_eq!(module_group_g.entry_module_id, "G".into());
    assert_eq!(module_group_g.modules(), &HashSet::from_iter(["G".into()]));

    let test_pairs = vec![
      ("A", vec![&group_id_a, &group_id_f]),
      ("B", vec![&group_id_b]),
      ("C", vec![&group_id_a, &group_id_f]),
      ("D", vec![&group_id_d, &group_id_b]),
      ("E", vec![&group_id_b]),
      ("F", vec![&group_id_f]),
      ("G", vec![&group_id_g]),
    ];

    for tp in test_pairs {
      let m_a = module_graph.module_mut(&tp.0.into()).unwrap();
      assert_eq!(m_a.module_groups.len(), tp.1.len());

      for g_id in tp.1 {
        assert!(m_a.module_groups.contains(g_id));
      }
    }
  }

  #[test]
  fn module_group_from_entry() {
    let mut graph = construct_test_module_graph();

    let (module_group, de) = mgfe(&"A".into(), ModuleGroupType::Entry, &mut graph);
    assert_eq!(de, vec!["F".into(), "D".into()]);
    assert_eq!(module_group.entry_module_id, "A".into());
    assert_eq!(
      module_group.modules(),
      &HashSet::from_iter(["A".into(), "C".into()])
    );
    assert!(graph
      .module(&"A".into())
      .unwrap()
      .module_groups
      .contains(&module_group.id));
    assert!(graph
      .module(&"C".into())
      .unwrap()
      .module_groups
      .contains(&module_group.id));
  }

  #[test]
  fn module_group_graph_from_entries() {
    let mut graph = construct_test_module_graph();

    let entries = vec!["A".into(), "B".into()];
    let module_group_graph = crate::module_group_graph_from_entries(&entries, &mut graph);
    let final_module_group_graph = construct_test_module_group_graph();

    assert_eq!(module_group_graph, final_module_group_graph);
  }

  #[test]
  fn module_groups_from_dynamic_entries() {
    let mut graph = construct_test_module_graph_complex();

    let dynamic_entries = vec!["A".into(), "E".into()];
    let module_groups = super::module_groups_from_dynamic_entries(dynamic_entries, &mut graph);

    let mut module_group_entry_module_ids = module_groups
      .iter()
      .map(|mg| mg.entry_module_id.clone())
      .collect::<Vec<_>>();
    module_group_entry_module_ids.sort();
    assert_eq!(module_group_entry_module_ids, vec!["A".into(), "E".into()]);

    let test_pairs = vec![
      ("A", vec!["A", "C", "D", "F", "H"]),
      ("E", vec!["E", "G", "H"]),
    ];

    assert_eq!(module_groups.len(), test_pairs.len());

    for (mg_id, modules) in test_pairs {
      let mg = module_groups
        .iter()
        .find(|mg| mg.entry_module_id == mg_id.into())
        .unwrap();
      let mut mg_modules = mg
        .modules()
        .iter()
        .map(|m| m.to_string())
        .collect::<Vec<_>>();
      mg_modules.sort();
      assert_eq!(mg_modules, modules);

      for m in modules {
        assert!(graph
          .module(&m.into())
          .unwrap()
          .module_groups
          .contains(&mg.id));
      }
    }
  }

  #[test]
  fn module_group_graph_from_module_graph() {
    let mut graph = construct_test_module_graph_complex();
    // update A -> D to dynamic entry
    graph
      .update_edge(
        &"A".into(),
        &"D".into(),
        farmfe_core::module::module_graph::ModuleGraphEdge::new(vec![
          farmfe_core::module::module_graph::ModuleGraphEdgeDataItem {
            kind: ResolveKind::DynamicEntry {
              name: "AD".to_string(),
              output_filename: None,
            },
            ..Default::default()
          },
        ]),
      )
      .unwrap();
    graph.dynamic_entries = HashMap::from_iter([("D".into(), "D".to_string())]);

    let module_group_graph = crate::module_group_graph_from_module_graph(&mut graph);
    assert_debug_snapshot!(module_group_graph);
  }
}
