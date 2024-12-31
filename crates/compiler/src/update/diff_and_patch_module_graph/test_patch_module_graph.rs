use farmfe_core::{
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdge, ModuleGraphEdgeDataItem},
    Module, ModuleId, ModuleType,
  },
  plugin::ResolveKind,
};
use farmfe_testing_helpers::construct_test_module_graph;

use crate::update::diff_and_patch_module_graph::{
  test_diff_module_deps::create_basic_graph, ModuleDepsDiffResult,
};

use super::{diff_module_graph, patch_module_graph};

fn is_module_graph_equals(g1: &ModuleGraph, g2: &ModuleGraph) -> bool {
  if g1.modules().len() != g2.modules().len() {
    return false;
  }

  if g1.edge_count() != g2.edge_count() {
    return false;
  }

  for module in g1.modules() {
    if !g2.has_module(&module.id) {
      return false;
    }
  }

  for module in g2.modules() {
    if !g1.has_module(&module.id) {
      return false;
    }
  }

  for a in g1.modules() {
    for b in g1.modules() {
      if g1.has_edge(&a.id, &b.id) != g2.has_edge(&a.id, &b.id) {
        return false;
      }
    }
  }

  for a in g2.modules() {
    for b in g2.modules() {
      if g1.has_edge(&a.id, &b.id) != g2.has_edge(&a.id, &b.id) {
        return false;
      }
    }
  }

  true
}

#[test]
fn test_patch_module_graph_1() {
  let mut module_graph = create_basic_graph();
  let changed_module_id: ModuleId = "a".into();
  let mut update_module_graph = ModuleGraph::new();
  let mut module_a = Module::new(changed_module_id.clone());
  module_a.module_type = ModuleType::Custom(changed_module_id.to_string());
  update_module_graph.add_module(module_a);
  update_module_graph.add_module(Module::new("b".into()));
  update_module_graph.add_module(Module::new("d".into()));
  update_module_graph
    .add_edge(&"a".into(), &"d".into(), Default::default())
    .unwrap();
  update_module_graph
    .add_edge(&"a".into(), &"b".into(), Default::default())
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

  assert_eq!(module_graph.modules().len(), 4);
  assert_eq!(module_graph.edge_count(), 3);
  assert!(module_graph.has_edge(&"a".into(), &"b".into()));
  assert!(module_graph.has_edge(&"a".into(), &"d".into()));
  assert!(module_graph.has_edge(&"b".into(), &"c".into()));
}

/// Static import changed to dynamic import
#[test]
fn test_patch_module_graph_2() {
  let mut module_graph = create_basic_graph();
  let mut update_module_graph = create_basic_graph();
  update_module_graph.remove_module(&"c".into());
  let changed_module_id: ModuleId = "a".into();

  update_module_graph
    .module_mut(&changed_module_id)
    .unwrap()
    .module_type = ModuleType::Custom(changed_module_id.to_string());

  update_module_graph
    .remove_edge(&"a".into(), &"b".into())
    .unwrap();
  update_module_graph
    .add_edge_item(
      &"a".into(),
      &"b".into(),
      ModuleGraphEdgeDataItem {
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
        ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
          kind: ResolveKind::DynamicImport,
          ..Default::default()
        }])
      )],
      removed: vec![("b".into(), ModuleGraphEdge::default())],
    }
  );

  let edge_info = module_graph
    .edge_info(&changed_module_id, &"b".into())
    .unwrap();
  assert!(edge_info.is_dynamic_import());
}

// TODO update these complex 1 2 3 tests
#[test]
fn test_patch_module_graph_complex_1() {
  let mut module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();
  update_module_graph.remove_module(&"F".into());
  update_module_graph.remove_module(&"G".into());
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
  update_module_graph
    .remove_edge(&"C".into(), &"F".into())
    .unwrap();
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

  update_module_graph.remove_module(&"A".into());
  update_module_graph.remove_module(&"C".into());
  update_module_graph.remove_module(&"G".into());

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

#[test]
fn test_patch_module_graph_complex_4() {
  let mut module_graph = ModuleGraph::new();
  module_graph.add_module(Module::new("A".into()));
  module_graph.add_module(Module::new("B".into()));
  module_graph.add_module(Module::new("C".into()));

  module_graph
    .add_edge(&"A".into(), &"B".into(), Default::default())
    .unwrap();
  module_graph
    .add_edge(&"B".into(), &"C".into(), Default::default())
    .unwrap();
  module_graph
    .add_edge(&"A".into(), &"C".into(), Default::default())
    .unwrap();

  let mut update_module_graph = ModuleGraph::new();
  update_module_graph.add_module(Module::new("A".into()));
  update_module_graph.add_module(Module::new("B".into()));

  update_module_graph
    .add_edge(&"A".into(), &"B".into(), Default::default())
    .unwrap();

  let diff_result = super::diff_module_graph(vec!["A".into()], &module_graph, &update_module_graph);

  patch_module_graph(
    vec!["A".into()],
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  assert_eq!(module_graph.modules().len(), 3);
  assert_eq!(module_graph.edge_count(), 2);
  assert!(module_graph.has_edge(&"A".into(), &"B".into()));
  assert!(module_graph.has_edge(&"B".into(), &"C".into()));
}

#[test]
fn test_patch_module_graph_add_and_remove() {
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

  let diff_result = super::diff_module_graph(vec!["a".into()], &module_graph, &update_module_graph);

  patch_module_graph(
    vec!["a".into()],
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  assert_eq!(module_graph.modules().len(), 5);
  assert_eq!(module_graph.edge_count(), 4);
  assert!(module_graph.has_edge(&"a".into(), &"e".into()));
  assert!(module_graph.has_edge(&"e".into(), &"b".into()));
  assert!(module_graph.has_edge(&"a".into(), &"d".into()));
  assert!(module_graph.has_edge(&"b".into(), &"c".into()));
  assert!(!module_graph.has_edge(&"a".into(), &"b".into()));
}

#[test]
fn test_patch_module_graph_remove_and_add() {
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

  let diff_result = super::diff_module_graph(vec!["a".into()], &module_graph, &update_module_graph);

  patch_module_graph(
    vec!["a".into()],
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  is_module_graph_equals(&module_graph, &create_basic_graph());
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

  let diff_result = super::diff_module_graph(
    vec!["a".into(), "b".into()],
    &module_graph,
    &update_module_graph,
  );

  patch_module_graph(
    vec!["a".into(), "b".into()],
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  is_module_graph_equals(&module_graph, &create_update_module_graph());
}
