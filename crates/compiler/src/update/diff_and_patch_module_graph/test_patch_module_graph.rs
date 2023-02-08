use farmfe_core::{
  module::{module_graph::ModuleGraphEdge, Module, ModuleId, ModuleType},
  plugin::ResolveKind,
};
use farmfe_testing_helpers::construct_test_module_graph;

use crate::update::diff_and_patch_module_graph::{
  test_diff_module_deps::create_basic_graph, ModuleDepsDiffResult,
};

use super::{diff_module_graph, patch_module_graph};

#[test]
fn test_patch_module_graph_1() {
  let (mut module_graph, mut update_module_graph) = create_basic_graph();
  let changed_module_id: ModuleId = "a".into();

  update_module_graph
    .module_mut(&changed_module_id)
    .unwrap()
    .module_type = ModuleType::Custom(changed_module_id.to_string());

  let start_points = vec![changed_module_id.clone()];
  let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);

  patch_module_graph(
    start_points,
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  assert_eq!(
    module_graph.module(&changed_module_id).unwrap().module_type,
    ModuleType::Custom(changed_module_id.to_string())
  );
}

/// Static import changed to dynamic import
#[test]
fn test_patch_module_graph_2() {
  let (mut module_graph, mut update_module_graph) = create_basic_graph();
  let changed_module_id: ModuleId = "a".into();

  update_module_graph
    .module_mut(&changed_module_id)
    .unwrap()
    .module_type = ModuleType::Custom(changed_module_id.to_string());

  update_module_graph
    .remove_edge(&"a".into(), &"b".into())
    .unwrap();
  update_module_graph
    .add_edge(
      &"a".into(),
      &"b".into(),
      ModuleGraphEdge {
        kind: ResolveKind::DynamicImport,
        ..Default::default()
      },
    )
    .unwrap();

  let start_points = vec![changed_module_id.clone()];
  let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);

  patch_module_graph(
    start_points,
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  assert_eq!(
    module_graph.module(&changed_module_id).unwrap().module_type,
    ModuleType::Custom(changed_module_id.to_string())
  );
  assert!(diff_result.added_modules.is_empty());
  assert!(diff_result.removed_modules.is_empty());
  assert_eq!(diff_result.deps_changes.len(), 1);

  let deps_change = diff_result
    .deps_changes
    .iter()
    .find(|(id, _)| id == &changed_module_id)
    .unwrap();
  assert_eq!(
    deps_change.1,
    ModuleDepsDiffResult {
      added: vec![(
        "b".into(),
        ModuleGraphEdge {
          kind: ResolveKind::DynamicImport,
          ..Default::default()
        }
      )],
      removed: vec![("b".into(), ModuleGraphEdge::default())],
    }
  );

  let edge_info = module_graph
    .edge_info(&changed_module_id, &"b".into())
    .unwrap();
  assert_eq!(edge_info.kind, ResolveKind::DynamicImport);
}

#[test]
fn test_patch_module_graph_complex_1() {
  let mut module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();

  update_module_graph
    .remove_edge(&"A".into(), &"D".into())
    .unwrap();

  let diff_result = super::diff_module_graph(
    vec!["A".into(), "B".into()],
    &module_graph,
    &update_module_graph,
  );
  patch_module_graph(
    vec!["A".into(), "B".into()],
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  assert_eq!(module_graph.modules().len(), 7);
  assert_eq!(module_graph.edge_count(), 7);
  assert!(!module_graph.has_edge(&"A".into(), &"D".into()));
}

#[test]
fn test_patch_module_graph_complex_2() {
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

  let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);

  patch_module_graph(
    start_points,
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  assert_eq!(module_graph.modules().len(), 5);
  assert_eq!(module_graph.edge_count(), 5);

  assert!(!module_graph.has_edge(&"A".into(), &"D".into()));
  assert!(!module_graph.has_module(&"D".into()));
  assert!(!module_graph.has_module(&"E".into()));
  assert!(!module_graph.has_module(&"G".into()));
  assert!(module_graph.has_module(&"H".into()));
  assert!(module_graph.has_edge(&"B".into(), &"H".into()));
  assert!(module_graph.has_edge(&"H".into(), &"F".into()));
  assert!(module_graph.has_edge(&"F".into(), &"A".into()));
  assert!(module_graph.has_edge(&"A".into(), &"C".into()));
}

#[test]
fn test_patch_module_graph_complex_3() {
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

  let diff_result = super::diff_module_graph(
    vec!["F".into(), "B".into()],
    &module_graph,
    &update_module_graph,
  );

  assert!(module_graph.has_edge(&"F".into(), &"A".into()));
  assert_eq!(module_graph.modules().len(), 7);
  assert_eq!(module_graph.edge_count(), 8);

  patch_module_graph(
    vec!["F".into(), "B".into()],
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  assert!(!module_graph.has_edge(&"F".into(), &"A".into()));
  assert!(module_graph.has_module(&"H".into()));
  assert!(module_graph.has_edge(&"H".into(), &"F".into()));
  assert!(module_graph.has_edge(&"B".into(), &"H".into()));

  assert_eq!(module_graph.modules().len(), 8);
  assert_eq!(module_graph.edge_count(), 9);
}
