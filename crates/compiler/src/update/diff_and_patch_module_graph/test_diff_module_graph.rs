use farmfe_core::{
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdge, ModuleGraphEdgeDataItem},
    Module,
  },
  plugin::ResolveKind,
};
use farmfe_testing_helpers::construct_test_module_graph;
use rustc_hash::FxHashSet;

use crate::update::diff_and_patch_module_graph::ModuleDepsDiffResult;

#[test]
fn test_diff_module_graph_complex_1() {
  let module_graph = construct_test_module_graph();
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
  assert!(diff_result.added_modules.is_empty());
  assert!(diff_result.removed_modules.is_empty());
  assert_eq!(
    diff_result.deps_changes,
    Vec::from([(
      "A".into(),
      ModuleDepsDiffResult {
        added: vec![],
        removed: vec![(
          "D".into(),
          ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
            order: 1,
            kind: ResolveKind::DynamicImport,
            source: "./D".to_string(),
          }])
        )],
      }
    ),])
  );
}

#[test]
fn test_diff_module_graph_complex_2() {
  let module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();

  update_module_graph
    .remove_edge(&"C".into(), &"F".into())
    .unwrap();
  update_module_graph
    .remove_edge(&"F".into(), &"A".into())
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

  assert_eq!(update_module_graph.modules().len(), 5);
  assert_eq!(update_module_graph.edge_count(), 3);

  let diff_result = super::diff_module_graph(
    vec!["B".into(), "A".into(), "C".into(), "F".into()],
    &module_graph,
    &update_module_graph,
  );

  diff_result.readable_print();

  let added_modules = diff_result.added_modules;
  let removed_modules = diff_result.removed_modules;
  let diff_result = diff_result.deps_changes;

  assert_eq!(added_modules, FxHashSet::from_iter(["H".into()]));
  assert_eq!(
    removed_modules,
    FxHashSet::from_iter(["D".into(), "E".into(), "G".into()])
  );

  assert_eq!(
    diff_result,
    Vec::from([
      (
        "B".into(),
        ModuleDepsDiffResult {
          added: vec![("H".into(), Default::default())],
          removed: vec![
            (
              "D".into(),
              ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
                source: "./D".to_string(),
                ..Default::default()
              }])
            ),
            (
              "E".into(),
              ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
                source: "./E".to_string(),
                order: 1,
                ..Default::default()
              }])
            )
          ],
        }
      ),
      (
        "A".into(),
        ModuleDepsDiffResult {
          added: vec![],
          removed: vec![(
            "D".into(),
            ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
              kind: ResolveKind::DynamicImport,
              source: "./D".to_string(),
              order: 1,
            }])
          )],
        }
      ),
      (
        "C".into(),
        ModuleDepsDiffResult {
          added: vec![],
          removed: vec![(
            "F".into(),
            ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
              kind: ResolveKind::DynamicImport,
              source: "./F".to_string(),
              order: 0,
            }])
          )],
        }
      ),
      (
        "F".into(),
        ModuleDepsDiffResult {
          added: vec![],
          removed: vec![(
            "A".into(),
            ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
              kind: ResolveKind::Import,
              source: "./A".to_string(),
              order: 0,
            }])
          )],
        }
      ),
      (
        "H".into(),
        ModuleDepsDiffResult {
          added: vec![("F".into(), Default::default())],
          removed: vec![],
        }
      ),
      (
        "D".into(),
        ModuleDepsDiffResult {
          added: vec![],
          removed: vec![(
            "F".into(),
            ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
              source: "./F".to_string(),
              kind: ResolveKind::DynamicImport,
              ..Default::default()
            }])
          )],
        }
      ),
      (
        "E".into(),
        ModuleDepsDiffResult {
          added: vec![],
          removed: vec![(
            "G".into(),
            ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
              source: "./G".to_string(),
              kind: ResolveKind::DynamicImport,
              ..Default::default()
            }])
          )],
        }
      ),
    ])
  );
}

#[test]
fn test_diff_module_graph_complex_3() {
  let module_graph = construct_test_module_graph();
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
  assert_eq!(
    diff_result.added_modules,
    FxHashSet::from_iter(["H".into()])
  );
  assert!(diff_result.removed_modules.is_empty());
  assert_eq!(
    diff_result.deps_changes,
    Vec::from([
      (
        "F".into(),
        ModuleDepsDiffResult {
          added: vec![],
          removed: vec![(
            "A".into(),
            ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
              source: "./A".to_string(),
              ..Default::default()
            }])
          )],
        }
      ),
      (
        "B".into(),
        ModuleDepsDiffResult {
          added: vec![("H".into(), Default::default())],
          removed: vec![]
        }
      ),
      (
        "H".into(),
        ModuleDepsDiffResult {
          added: vec![("F".into(), Default::default())],
          removed: vec![]
        }
      )
    ])
  );
}

#[test]
fn test_diff_module_graph_complex_4() {
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

  assert!(diff_result.added_modules.is_empty());
  assert!(diff_result.removed_modules.is_empty());
  assert_eq!(
    diff_result.deps_changes,
    Vec::from([(
      "A".into(),
      ModuleDepsDiffResult {
        added: vec![],
        removed: vec![("C".into(), ModuleGraphEdge::default())],
      }
    ),])
  );
}

fn get_edge_info(kind: ResolveKind) -> ModuleGraphEdge {
  ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
    kind,
    ..Default::default()
  }])
}

#[test]
fn test_diff_module_graph_complex_5() {
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

  let diff_result = super::diff_module_graph(vec!["D".into()], &module_graph, &update_module_graph);

  assert_eq!(
    diff_result.added_modules,
    FxHashSet::from_iter(["H".into()])
  );
  assert_eq!(diff_result.removed_modules, FxHashSet::from_iter([]));
  assert_eq!(
    diff_result.deps_changes,
    Vec::from([
      (
        "D".into(),
        ModuleDepsDiffResult {
          added: vec![("H".into(), get_edge_info(ResolveKind::Import))],
          removed: vec![("I.module.css".into(), get_edge_info(ResolveKind::Import))],
        }
      ),
      (
        "H".into(),
        ModuleDepsDiffResult {
          added: vec![("I.module.css".into(), get_edge_info(ResolveKind::Import))],
          removed: vec![]
        }
      ),
    ])
  )
}
