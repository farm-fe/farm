use farmfe_core::{
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdge, ModuleGraphEdgeDataItem},
    Module,
  },
  plugin::ResolveKind,
  HashMap, HashSet,
};
use farmfe_testing_helpers::construct_test_module_graph;

use crate::update::diff_and_patch_module_graph::ModuleDepsDiffResult;

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
  module_graph.entries = HashMap::from_iter([("a".into(), "a".to_string())]);

  module_graph
}

/// ```ignore
/// 1. when the deps not changed
/// module_graph:
/// a -> b -> c
///   \-> d
/// update_module_graph:
/// a(changed) -> b -> c
///   \-> d
/// diff_result:
/// (ModuleDepsDiffResult { added: [], removed: [] }, HashSet::new(), HashSet::new())
/// ```
#[test]
fn test_diff_module_deps_1() {
  let module_graph = create_basic_graph();
  let mut update_module_graph = ModuleGraph::new();
  update_module_graph.add_module(Module::new("a".into()));
  update_module_graph.add_module(Module::new("b".into()));
  update_module_graph.add_module(Module::new("d".into()));
  update_module_graph
    .add_edge(&"a".into(), &"b".into(), ModuleGraphEdge::default())
    .unwrap();
  update_module_graph
    .add_edge(&"a".into(), &"d".into(), ModuleGraphEdge::default())
    .unwrap();
  let (diff_result, added_modules, removed_modules) =
    super::diff_module_deps(&vec!["a".into()], &module_graph, &update_module_graph);

  assert!(added_modules.is_empty());
  assert!(removed_modules.is_empty());
  assert!(diff_result.is_empty());
}

/// ```ignore
/// 2. when the deps changed
/// module_graph:
/// a -> b -> c
///  \-> d
/// update_module_graph:
/// a(changed) ->(dep removed) b -> c
///   \-> d
///   \->(dep added) f
/// diff_result:
///   ({
///     a: ModuleDepsDiffResult { added: [f], removed: [b] }
///     b: ModuleDepsDiffResult { added: [], removed: [c] }
///    }, [f], [b, c])
/// ```
#[test]
fn test_diff_module_deps_2() {
  let module_graph = create_basic_graph();
  let mut update_module_graph = ModuleGraph::new();
  update_module_graph.add_module(Module::new("a".into()));
  update_module_graph.add_module(Module::new("d".into()));
  update_module_graph
    .add_edge(&"a".into(), &"d".into(), ModuleGraphEdge::default())
    .unwrap();
  update_module_graph.add_module(Module::new("f".into()));
  update_module_graph
    .add_edge(&"a".into(), &"f".into(), ModuleGraphEdge::default())
    .unwrap();

  let (diff_result, added_modules, removed_modules) =
    super::diff_module_deps(&vec!["a".into()], &module_graph, &update_module_graph);
  assert_eq!(added_modules, HashSet::from_iter(["f".into()]));
  assert_eq!(
    removed_modules,
    HashSet::from_iter(["b".into(), "c".into()])
  );

  assert_eq!(
    diff_result,
    Vec::from([
      (
        "a".into(),
        ModuleDepsDiffResult {
          added: vec![("f".into(), ModuleGraphEdge::default())],
          removed: vec![("b".into(), ModuleGraphEdge::default())]
        }
      ),
      (
        "b".into(),
        ModuleDepsDiffResult {
          added: vec![],
          removed: vec![("c".into(), ModuleGraphEdge::default())]
        }
      )
    ])
  );
}

/// ```ignore
/// 3. when the deps added with new module depend on existing module
/// module_graph:
/// a -> b -> c
/// update_module_graph:
/// a(changed) -> b -> c
///  \->(dep added) d -> c(existing module)
/// diff_result:
///  ({
///    a: ModuleDepsDiffResult { added: [d], removed: [] }
///    d: ModuleDepsDiffResult { added: [c], removed: [] }
///  }, [d], [])
/// ```
#[test]
fn test_diff_module_deps_3() {
  let mut module_graph = create_basic_graph();
  module_graph.remove_module(&"d".into());

  let mut update_module_graph = create_basic_graph();
  update_module_graph
    .add_edge(&"d".into(), &"c".into(), ModuleGraphEdge::default())
    .unwrap();
  update_module_graph
    .remove_edge(&"b".into(), &"c".into())
    .unwrap();

  let (diff_result, added_modules, removed_modules) =
    super::diff_module_deps(&vec!["a".into()], &module_graph, &update_module_graph);
  assert_eq!(added_modules, HashSet::from_iter(["d".into()]));
  assert!(removed_modules.is_empty());

  assert_eq!(
    diff_result,
    Vec::from([
      (
        "a".into(),
        ModuleDepsDiffResult {
          added: vec![("d".into(), ModuleGraphEdge::default())],
          removed: vec![]
        }
      ),
      (
        "d".into(),
        ModuleDepsDiffResult {
          added: vec![("c".into(), ModuleGraphEdge::default())],
          removed: vec![]
        }
      )
    ])
  );
}

/// ```ignore
/// 4. when the deps removed with removed module  depend on existing module
/// module_graph:
/// a -> b -> c
///  \-> d -> c
/// update_module_graph:
/// a(changed) -> b -> c
///  \->(dep removed) d -> c(existing module)
/// diff_result:
/// ({
///  a: ModuleDepsDiffResult { added: [], removed: [d] }
///  d: ModuleDepsDiffResult { added: [], removed: [c] }
/// }, [], [d])
/// ```
#[test]
fn test_diff_module_deps_4() {
  let mut module_graph = create_basic_graph();
  module_graph
    .add_edge(&"d".into(), &"c".into(), Default::default())
    .unwrap();

  let mut update_module_graph = ModuleGraph::new();
  update_module_graph.add_module(Module::new("a".into()));
  update_module_graph.add_module(Module::new("b".into()));
  update_module_graph
    .add_edge(&"a".into(), &"b".into(), ModuleGraphEdge::default())
    .unwrap();

  let (diff_result, added_modules, removed_modules) =
    super::diff_module_deps(&vec!["a".into()], &module_graph, &update_module_graph);
  assert!(added_modules.is_empty());
  assert_eq!(removed_modules, HashSet::from_iter(["d".into()]));

  assert_eq!(
    diff_result,
    Vec::from([
      (
        "a".into(),
        ModuleDepsDiffResult {
          added: vec![],
          removed: vec![("d".into(), ModuleGraphEdge::default())],
        }
      ),
      (
        "d".into(),
        ModuleDepsDiffResult {
          added: vec![],
          removed: vec![("c".into(), ModuleGraphEdge::default())],
        }
      ),
    ])
  );
}

#[test]
fn test_diff_module_deps_complex_1() {
  let module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();
  update_module_graph.remove_module(&"D".into());
  update_module_graph.remove_module(&"B".into());
  update_module_graph.remove_module(&"E".into());
  update_module_graph.remove_module(&"G".into());
  assert_eq!(update_module_graph.modules().len(), 3);

  let (diff_result, added_modules, removed_modules) =
    super::diff_module_deps(&vec!["A".into()], &module_graph, &update_module_graph);
  assert!(added_modules.is_empty());
  assert!(removed_modules.is_empty());
  assert_eq!(
    diff_result,
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
fn test_diff_module_deps_complex_2() {
  let module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();

  update_module_graph.remove_module(&"A".into());
  update_module_graph.remove_module(&"C".into());
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

  let (diff_result, added_modules, removed_modules) =
    super::diff_module_deps(&vec!["B".into()], &module_graph, &update_module_graph);
  assert_eq!(added_modules, HashSet::from_iter(["H".into()]));
  assert_eq!(
    removed_modules,
    HashSet::from_iter(["E".into(), "G".into()])
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
        "H".into(),
        ModuleDepsDiffResult {
          added: vec![("F".into(), Default::default())],
          removed: vec![],
        }
      ),
      (
        "E".into(),
        ModuleDepsDiffResult {
          added: vec![],
          removed: vec![(
            "G".into(),
            ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
              kind: ResolveKind::DynamicImport,
              source: "./G".to_string(),
              ..Default::default()
            }])
          )],
        }
      ),
    ])
  );
}

#[test]
fn test_diff_module_deps_complex_3() {
  let module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();
  update_module_graph.remove_module(&"A".into());
  update_module_graph.remove_module(&"C".into());
  update_module_graph.remove_module(&"D".into());
  update_module_graph.remove_module(&"E".into());
  update_module_graph.remove_module(&"G".into());
  update_module_graph.add_module(Module::new("H".into()));
  update_module_graph
    .add_edge(&"F".into(), &"H".into(), Default::default())
    .unwrap();
  update_module_graph
    .add_edge(&"H".into(), &"B".into(), Default::default())
    .unwrap();

  let (diff_result, added_modules, removed_modules) =
    super::diff_module_deps(&vec!["F".into()], &module_graph, &update_module_graph);
  assert_eq!(added_modules, HashSet::from_iter(["H".into()]));
  assert!(removed_modules.is_empty());
  assert_eq!(
    diff_result,
    Vec::from([
      (
        "F".into(),
        ModuleDepsDiffResult {
          added: vec![("H".into(), ModuleGraphEdge::default())],
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
        "H".into(),
        ModuleDepsDiffResult {
          added: vec![("B".into(), Default::default())],
          removed: vec![],
        }
      )
    ])
  );
}

#[test]
fn test_diff_module_deps_complex_4() {
  let module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();
  update_module_graph.remove_module(&"A".into());
  update_module_graph.remove_module(&"C".into());
  update_module_graph.remove_module(&"D".into());
  update_module_graph.remove_module(&"B".into());
  update_module_graph.remove_module(&"G".into());

  update_module_graph
    .add_edge(&"E".into(), &"F".into(), Default::default())
    .unwrap();
  assert_eq!(update_module_graph.modules().len(), 2);
  assert_eq!(update_module_graph.edge_count(), 1);

  let (diff_result, added_modules, removed_modules) =
    super::diff_module_deps(&vec!["E".into()], &module_graph, &update_module_graph);
  assert!(added_modules.is_empty());
  assert_eq!(removed_modules, HashSet::from_iter(["G".into()]));
  assert_eq!(
    diff_result,
    Vec::from([(
      "E".into(),
      ModuleDepsDiffResult {
        added: vec![("F".into(), ModuleGraphEdge::default())],
        removed: vec![(
          "G".into(),
          ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
            source: "./G".to_string(),
            kind: ResolveKind::DynamicImport,
            ..Default::default()
          }])
        )],
      }
    )])
  );
}

#[test]
fn test_diff_module_deps_add_and_remove() {
  let module_graph = create_basic_graph();
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

  let (diff_result, added_modules, removed_modules) =
    super::diff_module_deps(&vec!["a".into()], &module_graph, &update_module_graph);
  assert_eq!(added_modules, HashSet::from_iter(["e".into()]));
  assert_eq!(removed_modules, HashSet::from_iter([]));
  assert_eq!(
    diff_result,
    Vec::from([
      (
        "a".into(),
        ModuleDepsDiffResult {
          added: vec![("e".into(), ModuleGraphEdge::default())],
          removed: vec![("b".into(), ModuleGraphEdge::default())],
        }
      ),
      (
        "e".into(),
        ModuleDepsDiffResult {
          added: vec![("b".into(), ModuleGraphEdge::default())],
          removed: vec![],
        }
      ),
    ])
  );
}

#[test]
fn test_diff_module_deps_remove_and_add() {
  let update_module_graph = create_basic_graph();
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

  let (diff_result, added_modules, removed_modules) =
    super::diff_module_deps(&vec!["a".into()], &module_graph, &update_module_graph);
  assert_eq!(removed_modules, HashSet::from_iter(["e".into()]));
  assert_eq!(added_modules, HashSet::from_iter([]));
  assert_eq!(
    diff_result,
    Vec::from([
      (
        "a".into(),
        ModuleDepsDiffResult {
          added: vec![("b".into(), ModuleGraphEdge::default())],
          removed: vec![("e".into(), ModuleGraphEdge::default())],
        }
      ),
      (
        "e".into(),
        ModuleDepsDiffResult {
          added: vec![],
          removed: vec![("b".into(), ModuleGraphEdge::default())],
        }
      )
    ])
  );
}

#[test]
fn test_diff_module_deps_remove_and_add_complex() {
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
  let mut module_graph = create_basic_graph();
  module_graph.remove_module(&"c".into());

  let (diff_result, added_modules, removed_modules) = super::diff_module_deps(
    &vec!["a".into(), "b".into()],
    &module_graph,
    &update_module_graph,
  );
  assert_eq!(
    added_modules,
    HashSet::from_iter(["e".into(), "c".into()])
  );
  assert_eq!(removed_modules, HashSet::from_iter([]));
  assert_eq!(
    diff_result,
    Vec::from([
      (
        "a".into(),
        ModuleDepsDiffResult {
          added: vec![],
          removed: vec![("d".into(), ModuleGraphEdge::default())],
        }
      ),
      (
        "b".into(),
        ModuleDepsDiffResult {
          added: vec![("c".into(), ModuleGraphEdge::default())],
          removed: vec![],
        }
      ),
      (
        "c".into(),
        ModuleDepsDiffResult {
          added: vec![("e".into(), ModuleGraphEdge::default())],
          removed: vec![],
        }
      ),
      (
        "e".into(),
        ModuleDepsDiffResult {
          added: vec![("d".into(), ModuleGraphEdge::default())],
          removed: vec![],
        }
      )
    ])
  );
}
