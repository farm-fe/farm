use farmfe_core::{
  HashMap, HashSet,
  module::{
    Module, ModuleId,
    module_graph::{ModuleGraph, ModuleGraphEdge, ModuleGraphEdgeDataItem},
    module_group::{ModuleGroupId, ModuleGroupType},
  },
  plugin::ResolveKind,
};
use farmfe_plugin_partial_bundling::{
  module_group_graph_from_entries, module_group_graph_from_module_graph,
};
use farmfe_testing_helpers::{construct_test_module_graph, construct_test_module_graph_complex};

use crate::update::diff_and_patch_module_graph::{diff_module_graph, patch_module_graph};

use super::patch_module_group_graph;

#[test]
fn deleted_glob_start_point_is_removed_from_its_module_group() {
  let importer: ModuleId = "glob-importer.js".into();
  let deleted: ModuleId = "files/deleted.js".into();
  let mut graph = ModuleGraph::new();
  graph.add_module(Module::new(importer.clone()));
  graph.add_module(Module::new(deleted.clone()));
  graph.entries.insert(importer.clone(), "index".into());
  graph
    .add_edge(&importer, &deleted, Default::default())
    .unwrap();
  let mut groups = module_group_graph_from_entries(std::slice::from_ref(&importer), &mut graph);
  let entry_group = ModuleGroupId::new(&importer, &ModuleGroupType::Entry);
  assert!(
    groups
      .module_group(&entry_group)
      .unwrap()
      .modules()
      .contains(&deleted)
  );

  let mut update = ModuleGraph::new();
  update.add_module(Module::new(importer.clone()));
  update.add_module(Module::new(deleted.clone()));
  let start_points = vec![deleted.clone(), importer.clone()];
  let diff = diff_module_graph(start_points.clone(), &graph, &update);
  let removed = patch_module_graph(start_points.clone(), &diff, &mut graph, &mut update);
  let affected = patch_module_group_graph(start_points, &diff, &removed, &mut graph, &mut groups);

  assert!(removed.contains_key(&deleted));
  assert!(!graph.has_module(&deleted));
  assert!(affected.contains(&entry_group));
  let group = groups.module_group(&entry_group).unwrap();
  assert!(group.modules().contains(&importer));
  assert!(!group.modules().contains(&deleted));
  assert_eq!(group.modules().len(), 1);
}

#[test]
fn shared_module_group_closure_keeps_transitive_workers_and_excludes_disconnected_groups() {
  use farmfe_core::module::module_group::{ModuleGroup, ModuleGroupGraph};
  for count in [2, 8, 32] {
    let mut graph = ModuleGraph::new();
    for name in ["seed", "shared", "bridge", "leaf", "isolated"] {
      graph.add_module(Module::new(name.into()));
    }
    let mut groups = ModuleGroupGraph::new();
    let mut connected = HashSet::default();
    for index in 0..count {
      let kind = if index % 2 == 0 {
        ModuleGroupType::DynamicImport
      } else {
        ModuleGroupType::DynamicEntry
      };
      let mut group = ModuleGroup::new(format!("page-{index}").as_str().into(), kind);
      let mut entry = Module::new(group.entry_module_id.clone());
      entry.module_groups.insert(group.id.clone());
      graph.add_module(entry);
      let mut members = vec!["shared"];
      if index < 2 {
        members.push("seed");
      }
      if index == count - 1 {
        members.push("bridge");
      }
      for member in members {
        group.add_module(member.into());
        graph
          .module_mut(&member.into())
          .unwrap()
          .module_groups
          .insert(group.id.clone());
      }
      connected.insert(group.id.clone());
      groups.add_module_group(group);
    }
    let mut leaf = ModuleGroup::new("leaf".into(), ModuleGroupType::DynamicImport);
    for member in ["bridge", "leaf"] {
      leaf.add_module(member.into());
      graph
        .module_mut(&member.into())
        .unwrap()
        .module_groups
        .insert(leaf.id.clone());
    }
    connected.insert(leaf.id.clone());
    groups.add_module_group(leaf);
    let mut isolated = ModuleGroup::new("isolated".into(), ModuleGroupType::Entry);
    isolated.add_module("isolated".into());
    graph
      .module_mut(&"isolated".into())
      .unwrap()
      .module_groups
      .insert(isolated.id.clone());
    groups.add_module_group(isolated);
    let before: HashMap<_, _> = graph
      .modules()
      .into_iter()
      .map(|module| (module.id.clone(), module.module_groups.clone()))
      .collect();
    let affected = patch_module_group_graph(
      vec!["seed".into()],
      &Default::default(),
      &HashMap::default(),
      &mut graph,
      &mut groups,
    );
    assert_eq!(affected, connected);
    for module in graph.modules() {
      assert_eq!(module.module_groups, before[&module.id]);
    }
  }
}

#[test]
fn removed_dynamic_group_does_not_panic_when_later_edge_cleanup_revisits_it() {
  let mut graph = create_basic_graph();
  graph
    .update_edge(
      &"a".into(),
      &"b".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::DynamicImport,
        ..Default::default()
      }]),
    )
    .unwrap();
  let mut groups = module_group_graph_from_entries(&vec!["a".into()], &mut graph);
  let removed = ModuleGroupId::new(&"b".into(), &ModuleGroupType::DynamicImport);
  assert!(groups.has(&removed));
  let mut affected = HashSet::default();
  super::remove_dynamic_module_group(&removed, &mut graph, &mut groups, &mut affected);
  graph.remove_edge(&"a".into(), &"b".into()).unwrap();
  super::patch_any_module_group_for_toposort_modules(
    vec!["b".into(), "c".into()],
    vec![],
    HashSet::from_iter([removed.clone()]),
    &mut graph,
    &mut groups,
    &mut affected,
  );
  assert!(!groups.has(&removed));
  for id in ["b", "c"] {
    assert!(
      !graph
        .module(&id.into())
        .unwrap()
        .module_groups
        .contains(&removed)
    );
  }
}

#[test]
fn deleted_dynamic_page_keeps_other_entry_shared_dependency_and_memberships() {
  let mut graph = create_basic_graph();
  graph
    .update_edge(
      &"a".into(),
      &"b".into(),
      get_edge_info(ResolveKind::DynamicImport),
    )
    .unwrap();
  graph.add_module(Module::new("other".into()));
  graph
    .add_edge(&"other".into(), &"c".into(), ModuleGraphEdge::default())
    .unwrap();
  let entries = vec!["a".into(), "other".into()];
  let mut groups = module_group_graph_from_entries(&entries, &mut graph);
  let mut updated = create_basic_graph();
  updated.remove_module(&"b".into());
  updated.add_module(Module::new("other".into()));
  updated
    .add_edge(&"other".into(), &"c".into(), ModuleGraphEdge::default())
    .unwrap();
  let start = vec!["a".into()];
  let diff = diff_module_graph(start.clone(), &graph, &updated);
  let removed = patch_module_graph(start.clone(), &diff, &mut graph, &mut updated);
  patch_module_group_graph(start, &diff, &removed, &mut graph, &mut groups);
  let old_group = ModuleGroupId::new(&"b".into(), &ModuleGroupType::DynamicImport);
  assert!(!groups.has(&old_group));
  let before: HashMap<_, _> = ["a", "c", "d", "other"]
    .into_iter()
    .map(|id| {
      let module = graph.module(&id.into()).unwrap();
      assert!(!module.module_groups.contains(&old_group));
      (id, module.module_groups.clone())
    })
    .collect();
  assert!(before["c"].contains(&ModuleGroupId::new(
    &"other".into(),
    &ModuleGroupType::Entry
  )));
  let rebuilt = module_group_graph_from_entries(&entries, &mut graph);
  assert_eq!(groups, rebuilt);
  for (id, membership) in before {
    assert_eq!(membership, graph.module(&id.into()).unwrap().module_groups);
  }
}

///
/// ```ignore
///      a
///     / \
///   b    d
///  /
/// c
/// ```
pub fn create_basic_graph() -> ModuleGraph {
  let mut module_graph = ModuleGraph::new();

  module_graph.add_module(Module::new("a".into()));
  module_graph.add_module(Module::new("b".into()));
  module_graph.add_module(Module::new("c".into()));
  module_graph.add_module(Module::new("d".into()));
  module_graph
    .add_edge(&"a".into(), &"b".into(), ModuleGraphEdge::default())
    .unwrap();
  module_graph
    .add_edge(&"a".into(), &"d".into(), ModuleGraphEdge::default())
    .unwrap();
  module_graph
    .add_edge(&"b".into(), &"c".into(), ModuleGraphEdge::default())
    .unwrap();

  module_graph
}

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

  let group_id_a = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([group_id_a.clone(), group_id_b.clone()])
  );

  let update_module_group_graph = module_group_graph_from_entries(&entries, &mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  let module_a = module_graph.module(&"A".into()).unwrap();
  assert_eq!(
    module_a.module_groups,
    HashSet::from_iter([group_id_a.clone()])
  );
  let module_b = module_graph.module(&"B".into()).unwrap();
  assert_eq!(
    module_b.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_c = module_graph.module(&"C".into()).unwrap();
  assert_eq!(module_c.module_groups, HashSet::from_iter([group_id_a]));
  let module_d = module_graph.module(&"D".into()).unwrap();
  assert_eq!(
    module_d.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_e = module_graph.module(&"E".into()).unwrap();
  assert_eq!(
    module_e.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
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

  let diff_result = diff_module_graph(updated_modules.clone(), &module_graph, &update_module_graph);
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

  let group_id_a = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);
  let group_id_f = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([group_id_a.clone(), group_id_b.clone(), group_id_f.clone()])
  );
  let module_group_b = module_group_graph.module_group(&group_id_b).unwrap();
  assert_eq!(
    module_group_b.modules(),
    &HashSet::from_iter(["B".into(), "H".into(), "F".into(), "C".into(), "A".into()])
  );

  let update_module_group_graph = module_group_graph_from_entries(&start_points, &mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  let module_a = module_graph.module(&"A".into()).unwrap();
  assert_eq!(
    module_a.module_groups,
    HashSet::from_iter([group_id_a.clone(), group_id_b.clone(), group_id_f.clone()])
  );
  let module_b = module_graph.module(&"B".into()).unwrap();
  assert_eq!(
    module_b.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_c = module_graph.module(&"C".into()).unwrap();
  assert_eq!(
    module_c.module_groups,
    HashSet::from_iter([group_id_a.clone(), group_id_b.clone(), group_id_f.clone()])
  );
  let module_f = module_graph.module(&"F".into()).unwrap();
  assert_eq!(
    module_f.module_groups,
    HashSet::from_iter([group_id_b.clone(), group_id_f.clone()])
  );
  let module_h = module_graph.module(&"H".into()).unwrap();
  assert_eq!(
    module_h.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
}

#[test]
fn test_patch_module_group_graph_3() {
  let mut module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();
  update_module_graph.remove_module(&"C".into());
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
    &module_graph.entries.clone().into_keys().collect::<Vec<_>>(),
    &mut module_graph,
  );
  let diff_result = diff_module_graph(updated_modules.clone(), &module_graph, &update_module_graph);

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

  let group_id_a = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);
  let group_id_d = ModuleGroupId::new(&"D".into(), &ModuleGroupType::DynamicImport);
  let group_id_f = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([
      group_id_a.clone(),
      group_id_b.clone(),
      group_id_d.clone(),
      group_id_f.clone()
    ])
  );

  let update_module_group_graph = module_group_graph_from_entries(
    &module_graph.entries.clone().into_keys().collect::<Vec<_>>(),
    &mut module_graph,
  );

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  let module_a = module_graph.module(&"A".into()).unwrap();
  assert_eq!(
    module_a.module_groups,
    HashSet::from_iter([group_id_a.clone()])
  );
  let module_b = module_graph.module(&"B".into()).unwrap();
  assert_eq!(
    module_b.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_c = module_graph.module(&"C".into()).unwrap();
  assert_eq!(
    module_c.module_groups,
    HashSet::from_iter([group_id_a.clone()])
  );
  let module_d = module_graph.module(&"D".into()).unwrap();
  assert_eq!(
    module_d.module_groups,
    HashSet::from_iter([group_id_b.clone(), group_id_d.clone()])
  );
  let module_e = module_graph.module(&"E".into()).unwrap();
  assert_eq!(
    module_e.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_f = module_graph.module(&"F".into()).unwrap();
  assert_eq!(
    module_f.module_groups,
    HashSet::from_iter([group_id_f.clone(), group_id_b.clone()])
  );
  let module_h = module_graph.module(&"H".into()).unwrap();
  assert_eq!(
    module_h.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
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
    &module_graph.entries.clone().into_keys().collect::<Vec<_>>(),
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

  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);
  let group_id_d = ModuleGroupId::new(&"D".into(), &ModuleGroupType::DynamicImport);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([group_id_b.clone(), group_id_d.clone()])
  );

  let update_module_group_graph = module_group_graph_from_entries(
    &module_graph.entries.clone().into_keys().collect::<Vec<_>>(),
    &mut module_graph,
  );

  assert_eq!(module_group_graph, update_module_group_graph);
}

#[test]
fn test_patch_module_group_graph_add_and_remove() {
  let mut module_graph = create_basic_graph();
  let mut update_module_graph = create_basic_graph();
  let module_e = Module::new("e".into());
  update_module_graph.add_module(module_e);
  update_module_graph
    .add_edge(&"a".into(), &"e".into(), Default::default())
    .unwrap();
  // remove a -> b and add e -> b
  update_module_graph
    .remove_edge(&"a".into(), &"b".into())
    .unwrap();
  update_module_graph
    .add_edge(&"e".into(), &"b".into(), Default::default())
    .unwrap();

  let start_points = vec!["a".into()];
  let mut module_group_graph = module_group_graph_from_entries(&start_points, &mut module_graph);
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

  let group_id_a = ModuleGroupId::new(&"a".into(), &ModuleGroupType::Entry);

  assert_eq!(affected_groups, HashSet::from_iter([group_id_a.clone()]));

  let update_module_group_graph = module_group_graph_from_entries(&start_points, &mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  for module in module_graph.modules() {
    assert_eq!(
      module.module_groups,
      HashSet::from_iter([group_id_a.clone()])
    );
  }
}

#[test]
fn test_patch_module_group_graph_remove_and_add() {
  let mut update_module_graph = create_basic_graph();
  let mut module_graph = create_basic_graph();
  let module_e = Module::new("e".into());
  module_graph.add_module(module_e);
  module_graph
    .add_edge(&"a".into(), &"e".into(), Default::default())
    .unwrap();
  // remove a -> b and add e -> b
  module_graph.remove_edge(&"a".into(), &"b".into()).unwrap();
  module_graph
    .add_edge(&"e".into(), &"b".into(), Default::default())
    .unwrap();
  let mut module_group_graph = module_group_graph_from_entries(&["a".into()], &mut module_graph);

  let diff_result = diff_module_graph(vec!["a".into()], &module_graph, &update_module_graph);

  let removed_modules = patch_module_graph(
    vec!["a".into()],
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  let affected_groups = patch_module_group_graph(
    vec!["a".into()],
    &diff_result,
    &removed_modules,
    &mut module_graph,
    &mut module_group_graph,
  );

  let group_id_a = ModuleGroupId::new(&"a".into(), &ModuleGroupType::Entry);

  assert_eq!(affected_groups, HashSet::from_iter([group_id_a.clone()]));

  let update_module_group_graph = module_group_graph_from_entries(&["a".into()], &mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  for module in module_graph.modules() {
    assert_eq!(
      module.module_groups,
      HashSet::from_iter([group_id_a.clone()])
    );
  }
}

#[test]
fn test_diff_module_deps_remove_and_add_complex() {
  let create_update_module_graph = || {
    let mut update_module_graph = create_basic_graph();
    let module_e = Module::new("e".into());
    update_module_graph.add_module(module_e);
    update_module_graph
      .remove_edge(&"a".into(), &"d".into())
      .unwrap();
    update_module_graph
      .add_edge(&"c".into(), &"e".into(), Default::default())
      .unwrap();
    // add edge e -> d
    update_module_graph
      .add_edge(&"e".into(), &"d".into(), Default::default())
      .unwrap();
    update_module_graph
  };
  let mut update_module_graph = create_update_module_graph();

  let mut module_graph = create_basic_graph();
  module_graph.remove_module(&"c".into());
  let mut module_group_graph = module_group_graph_from_entries(&["a".into()], &mut module_graph);

  let diff_result = diff_module_graph(
    vec!["a".into(), "b".into()],
    &module_graph,
    &update_module_graph,
  );

  let removed_modules = patch_module_graph(
    vec!["a".into(), "b".into()],
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  let affected_groups = patch_module_group_graph(
    vec!["a".into(), "b".into()],
    &diff_result,
    &removed_modules,
    &mut module_graph,
    &mut module_group_graph,
  );

  let group_id_a = ModuleGroupId::new(&"a".into(), &ModuleGroupType::Entry);

  assert_eq!(affected_groups, HashSet::from_iter([group_id_a.clone()]));

  let update_module_group_graph = module_group_graph_from_entries(&["a".into()], &mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  for module in module_graph.modules() {
    assert_eq!(
      module.module_groups,
      HashSet::from_iter([group_id_a.clone()])
    );
  }
}

#[test]
fn test_patch_module_group_graph_remove_normal_edge() {
  let mut module_graph = construct_test_module_graph_complex();
  let module_i = Module::new("I".into());
  module_graph.add_module(module_i);
  module_graph
    .add_edge(&"D".into(), &"I".into(), Default::default())
    .unwrap();
  let mut update_module_graph = construct_test_module_graph_complex();
  update_module_graph.remove_module(&"F".into());
  update_module_graph.remove_module(&"G".into());
  update_module_graph.remove_module(&"A".into());
  update_module_graph.remove_module(&"C".into());
  update_module_graph.remove_module(&"D".into());
  update_module_graph.remove_module(&"H".into());

  let entries = vec!["A".into(), "B".into()];
  let start_points = vec!["B".into()];
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

  let group_id_a = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  let group_id_d = ModuleGroupId::new(&"D".into(), &ModuleGroupType::DynamicImport);
  let group_id_g = ModuleGroupId::new(&"G".into(), &ModuleGroupType::DynamicImport);
  let group_id_f = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);
  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([
      group_id_d.clone(),
      group_id_b.clone(),
      group_id_g.clone(),
      group_id_a.clone(),
      group_id_f.clone()
    ])
  );

  let update_module_group_graph = module_group_graph_from_entries(&entries, &mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  let module_a = module_graph.module(&"A".into()).unwrap();
  assert_eq!(
    module_a.module_groups,
    HashSet::from_iter([group_id_a.clone(), group_id_f.clone()])
  );
  let module_b = module_graph.module(&"B".into()).unwrap();
  assert_eq!(
    module_b.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_c = module_graph.module(&"C".into()).unwrap();
  assert_eq!(
    module_c.module_groups,
    HashSet::from_iter([group_id_a, group_id_f.clone()])
  );
  let module_d = module_graph.module(&"D".into()).unwrap();
  assert_eq!(
    module_d.module_groups,
    HashSet::from_iter([group_id_d.clone()])
  );
  let module_e = module_graph.module(&"E".into()).unwrap();
  assert_eq!(
    module_e.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_h = module_graph.module(&"H".into()).unwrap();
  assert_eq!(
    module_h.module_groups,
    HashSet::from_iter([group_id_d.clone(), group_id_f.clone(), group_id_g.clone()])
  );
  let module_i = module_graph.module(&"I".into()).unwrap();
  assert_eq!(
    module_i.module_groups,
    HashSet::from_iter([group_id_d.clone()])
  );
}

#[test]
fn test_patch_module_group_graph_remove_dynamic_import_edge() {
  let mut module_graph = construct_test_module_graph_complex();
  let module_i = Module::new("I".into());
  module_graph.add_module(module_i);
  module_graph
    .add_edge(&"D".into(), &"I".into(), Default::default())
    .unwrap();
  let mut update_module_graph = construct_test_module_graph_complex();
  update_module_graph.remove_module(&"F".into());
  update_module_graph.remove_module(&"G".into());
  update_module_graph.remove_module(&"B".into());
  update_module_graph.remove_module(&"E".into());
  update_module_graph.remove_module(&"D".into());
  update_module_graph.remove_module(&"H".into());

  let entries = vec!["A".into(), "B".into()];
  let start_points = vec!["A".into()];
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

  let group_id_a = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  // let group_id_d = ModuleGroupId::new(&"D".into(), &ModuleGroupType::DynamicImport);
  let group_id_g = ModuleGroupId::new(&"G".into(), &ModuleGroupType::DynamicImport);
  let group_id_f = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);
  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([
      // group_id_d.clone(),
      group_id_b.clone(),
      group_id_g.clone(),
      group_id_a.clone(),
      group_id_f.clone()
    ])
  );

  let update_module_group_graph = module_group_graph_from_entries(&entries, &mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  let module_a = module_graph.module(&"A".into()).unwrap();
  assert_eq!(
    module_a.module_groups,
    HashSet::from_iter([group_id_a.clone(), group_id_f.clone()])
  );
  let module_b = module_graph.module(&"B".into()).unwrap();
  assert_eq!(
    module_b.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_c = module_graph.module(&"C".into()).unwrap();
  assert_eq!(
    module_c.module_groups,
    HashSet::from_iter([group_id_a, group_id_f.clone()])
  );
  let module_d = module_graph.module(&"D".into()).unwrap();
  assert_eq!(
    module_d.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_e = module_graph.module(&"E".into()).unwrap();
  assert_eq!(
    module_e.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_h = module_graph.module(&"H".into()).unwrap();
  assert_eq!(
    module_h.module_groups,
    HashSet::from_iter([group_id_b.clone(), group_id_f.clone(), group_id_g.clone()])
  );
  let module_i = module_graph.module(&"I".into()).unwrap();
  assert_eq!(
    module_i.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
}

#[test]
fn test_patch_module_group_graph_update_dynamic_import_edge() {
  let mut module_graph = construct_test_module_graph_complex();
  // make E -> G non-dynamic import, then in update_module_graph make it dynamic import
  module_graph
    .update_edge(
      &"E".into(),
      &"G".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::Import,
        ..Default::default()
      }]),
    )
    .unwrap();
  let mut update_module_graph = construct_test_module_graph_complex();
  update_module_graph
    .update_edge(
      &"B".into(),
      &"D".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::DynamicImport,
        ..Default::default()
      }]),
    )
    .unwrap();
  update_module_graph.remove_module(&"F".into());
  update_module_graph.remove_module(&"A".into());
  update_module_graph.remove_module(&"C".into());
  update_module_graph.remove_module(&"H".into());
  update_module_graph.entries =
    HashMap::from_iter([("B".into(), "B".to_string()), ("E".into(), "E".to_string())]);
  update_module_graph.update_execution_order_for_modules();

  let mut module_group_graph = module_group_graph_from_module_graph(&mut module_graph);
  let start_points = vec!["B".into(), "E".into()];
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

  let group_id_a = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);
  let group_id_f = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);
  let group_id_g = ModuleGroupId::new(&"G".into(), &ModuleGroupType::DynamicImport);
  let group_id_d = ModuleGroupId::new(&"D".into(), &ModuleGroupType::DynamicImport);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([
      group_id_a.clone(),
      group_id_b.clone(),
      group_id_f.clone(),
      group_id_g.clone(),
      group_id_d.clone()
    ])
  );

  let update_module_group_graph = module_group_graph_from_module_graph(&mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);
}
