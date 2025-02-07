use farmfe_core::{
  module::{
    module_graph::{ModuleGraphEdge, ModuleGraphEdgeDataItem},
    module_group::{ModuleGroupId, ModuleGroupType},
    Module,
  },
  plugin::ResolveKind,
  HashMap, HashSet,
};
use farmfe_plugin_partial_bundling::module_group_graph_from_module_graph;
use farmfe_testing_helpers::construct_test_module_graph_complex;

use crate::update::{
  diff_and_patch_module_graph::{diff_module_graph, patch_module_graph},
  patch_module_group_graph::patch_module_group_graph,
};

#[test]
fn test_patch_module_group_graph_dynamic_entry_complex() {
  let mut module_graph = construct_test_module_graph_complex();
  module_graph.dynamic_entries = HashMap::from_iter([("D".into(), "D".to_string())]);
  module_graph
    .update_edge(
      &"A".into(),
      &"D".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::DynamicEntry {
          name: "AD".to_string(),
          output_filename: None,
        },
        ..Default::default()
      }]),
    )
    .unwrap();

  let mut update_module_graph = construct_test_module_graph_complex();
  update_module_graph.add_module(Module::new("I".into()));
  update_module_graph
    .add_edge(
      &"E".into(),
      &"I".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::DynamicEntry {
          name: "EI".to_string(),
          output_filename: None,
        },
        ..Default::default()
      }]),
    )
    .unwrap();
  update_module_graph
    .add_edge(
      &"I".into(),
      &"H".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::Import,
        ..Default::default()
      }]),
    )
    .unwrap();
  update_module_graph.remove_module(&"F".into());
  update_module_graph.remove_module(&"B".into());
  update_module_graph.remove_module(&"D".into());
  update_module_graph.entries =
    HashMap::from_iter([("A".into(), "A".to_string()), ("E".into(), "E".to_string())]);
  update_module_graph.update_execution_order_for_modules();

  let mut module_group_graph = module_group_graph_from_module_graph(&mut module_graph);

  let start_points = vec!["A".into(), "E".into()];
  let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);
  let removed_modules = patch_module_graph(
    start_points.clone(),
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  assert_eq!(
    module_graph.entries,
    HashMap::from_iter([("A".into(), "A".to_string()), ("B".into(), "B".to_string()),])
  );

  assert_eq!(
    module_graph.dynamic_entries,
    HashMap::from_iter([("I".into(), "EI".to_string())])
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
  let group_id_f = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);
  let group_id_g = ModuleGroupId::new(&"G".into(), &ModuleGroupType::DynamicImport);
  let group_id_i = ModuleGroupId::new(&"I".into(), &ModuleGroupType::DynamicEntry);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([
      group_id_a.clone(),
      group_id_b.clone(),
      group_id_f.clone(),
      group_id_g.clone(),
      group_id_i.clone()
    ])
  );

  let update_module_group_graph = module_group_graph_from_module_graph(&mut module_graph);

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
    HashSet::from_iter([group_id_a.clone(), group_id_f.clone()])
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
  let module_i = module_graph.module(&"I".into()).unwrap();
  assert_eq!(
    module_i.module_groups,
    HashSet::from_iter([group_id_i.clone()])
  );
  let module_h = module_graph.module(&"H".into()).unwrap();
  assert_eq!(
    module_h.module_groups,
    HashSet::from_iter([
      group_id_i.clone(),
      group_id_f.clone(),
      group_id_g.clone(),
      group_id_b.clone()
    ])
  );
}

#[test]
fn test_patch_module_group_graph_dynamic_entry_update_dynamic_entry() {
  let mut module_graph = construct_test_module_graph_complex();
  let mut update_module_graph = construct_test_module_graph_complex();
  update_module_graph
    .update_edge(
      &"A".into(),
      &"D".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::DynamicEntry {
          name: "AD".to_string(),
          output_filename: None,
        },
        ..Default::default()
      }]),
    )
    .unwrap();
  update_module_graph.remove_module(&"F".into());
  update_module_graph.remove_module(&"B".into());
  update_module_graph.remove_module(&"E".into());
  update_module_graph.remove_module(&"G".into());
  update_module_graph.remove_module(&"H".into());
  update_module_graph.entries = HashMap::from_iter([("A".into(), "A".to_string())]);
  update_module_graph.update_execution_order_for_modules();

  let mut module_group_graph = module_group_graph_from_module_graph(&mut module_graph);

  let start_points = vec!["A".into()];
  let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);
  let removed_modules = patch_module_graph(
    start_points.clone(),
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  assert_eq!(
    module_graph.entries,
    HashMap::from_iter([("A".into(), "A".to_string()), ("B".into(), "B".to_string()),])
  );

  assert_eq!(
    module_graph.dynamic_entries,
    HashMap::from_iter([("D".into(), "AD".to_string())])
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
  let group_id_f = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);
  let group_id_g = ModuleGroupId::new(&"G".into(), &ModuleGroupType::DynamicImport);
  let group_id_d = ModuleGroupId::new(&"D".into(), &ModuleGroupType::DynamicEntry);

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

  // makes sure that module_groups field of each module is correct
  let module_a = module_graph.module(&"A".into()).unwrap();
  assert_eq!(
    module_a.module_groups,
    HashSet::from_iter([group_id_a.clone(), group_id_f.clone(), group_id_d.clone()])
  );
  let module_b = module_graph.module(&"B".into()).unwrap();
  assert_eq!(
    module_b.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_c = module_graph.module(&"C".into()).unwrap();
  assert_eq!(
    module_c.module_groups,
    HashSet::from_iter([group_id_a.clone(), group_id_f.clone(), group_id_d.clone()])
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
  let module_h = module_graph.module(&"H".into()).unwrap();
  assert_eq!(
    module_h.module_groups,
    HashSet::from_iter([
      group_id_d.clone(),
      group_id_f.clone(),
      group_id_g.clone(),
      group_id_b.clone()
    ])
  );
}
