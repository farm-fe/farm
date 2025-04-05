use farmfe_core::{
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdge, ModuleGraphEdgeDataItem},
    module_group::{ModuleGroupId, ModuleGroupType},
    Module, ModuleType,
  },
  plugin::ResolveKind,
  HashMap, HashSet,
};
use farmfe_plugin_partial_bundling::module_group_graph_from_module_graph;
use farmfe_testing_helpers::construct_test_module_graph_complex;

use crate::update::{
  diff_and_patch_module_graph::{diff_module_graph, patch_module_graph},
  patch_module_group_graph,
};

fn construct_remove_then_add_test_module_graph() -> ModuleGraph {
  let mut module_graph = construct_test_module_graph_complex();

  // add a new node I
  let mut module_i = Module::new("I".into());
  module_i.module_type = ModuleType::Js;
  module_graph.add_module(module_i);
  module_graph
    .add_edge(
      &"G".into(),
      &"I".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::DynamicImport,
        ..Default::default()
      }]),
    )
    .unwrap();
  module_graph
    .add_edge(
      &"I".into(),
      &"H".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::Import,
        ..Default::default()
      }]),
    )
    .unwrap();

  // add a new node J
  let mut module_j = Module::new("J".into());
  module_j.module_type = ModuleType::Js;
  module_graph.add_module(module_j);

  // H -> J
  module_graph
    .add_edge(
      &"H".into(),
      &"J".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::Import,
        ..Default::default()
      }]),
    )
    .unwrap();

  // add a new node K
  let mut module_k = Module::new("K".into());
  module_k.module_type = ModuleType::Js;

  // J -> K
  module_graph.add_module(module_k);
  module_graph
    .add_edge(
      &"J".into(),
      &"K".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::Import,
        ..Default::default()
      }]),
    )
    .unwrap();

  // H -> K
  module_graph
    .add_edge(
      &"H".into(),
      &"K".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::Import,
        ..Default::default()
      }]),
    )
    .unwrap();

  // add cyclic dependency K -> J
  module_graph
    .add_edge(
      &"K".into(),
      &"J".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::Import,
        ..Default::default()
      }]),
    )
    .unwrap();

  module_graph
}

#[test]
fn test_patch_module_group_graph_remove_then_add() {
  let mut module_graph = construct_remove_then_add_test_module_graph();

  let mut backup_graph = ModuleGraph::new();
  module_graph.copy_to(&mut backup_graph, true).unwrap();
  let mut update_module_graph = ModuleGraph::new();
  module_graph
    .copy_to(&mut update_module_graph, true)
    .unwrap();

  update_module_graph
    .remove_edge(&"B".into(), &"E".into())
    .unwrap();
  update_module_graph.remove_module(&"F".into());
  update_module_graph.remove_module(&"A".into());
  update_module_graph.remove_module(&"C".into());
  update_module_graph.remove_module(&"H".into());
  update_module_graph.remove_module(&"E".into());
  update_module_graph.remove_module(&"G".into());
  update_module_graph.remove_module(&"I".into());
  update_module_graph.entries = HashMap::from_iter([("B".into(), "B".to_string())]);
  update_module_graph.update_execution_order_for_modules();

  let mut module_group_graph = module_group_graph_from_module_graph(&mut module_graph);
  let start_points = vec!["B".into()];
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
      group_id_d.clone()
    ])
  );

  let update_module_group_graph = module_group_graph_from_module_graph(&mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);

  // add the dynamic modules back
  let mut update_module_graph = ModuleGraph::new();
  backup_graph
    .copy_to(&mut update_module_graph, true)
    .unwrap();

  update_module_graph
    .add_edge(
      &"B".into(),
      &"E".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::Import,
        ..Default::default()
      }]),
    )
    .unwrap();
  update_module_graph.remove_module(&"F".into());
  update_module_graph.remove_module(&"A".into());
  update_module_graph.remove_module(&"C".into());

  let start_points = vec!["B".into()];
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

  let group_id_i = ModuleGroupId::new(&"I".into(), &ModuleGroupType::DynamicImport);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([
      group_id_a.clone(),
      group_id_b.clone(),
      group_id_f.clone(),
      group_id_g.clone(),
      group_id_d.clone(),
      group_id_i.clone()
    ])
  );

  let update_module_group_graph = module_group_graph_from_module_graph(&mut module_graph);
  let backup_module_group_graph = module_group_graph_from_module_graph(&mut backup_graph);

  assert_eq!(backup_module_group_graph, update_module_group_graph);
}
