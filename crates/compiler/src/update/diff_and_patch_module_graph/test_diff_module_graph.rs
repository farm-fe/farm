use farmfe_core::{
  hashbrown::HashSet,
  module::{module_graph::ModuleGraphEdge, Module},
  plugin::ResolveKind,
};
use farmfe_testing_helpers::construct_test_module_graph;

use crate::update::diff_and_patch_module_graph::ModuleDepsDiffResult;

#[test]
fn test_diff_module_graph_complex_1() {
  let module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();

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
          ModuleGraphEdge {
            order: 1,
            kind: ResolveKind::DynamicImport,
            sources: vec!["./D".to_string()],
          }
        )],
      }
    ),])
  );
}

#[test]
fn test_diff_module_graph_complex_2() {
  let module_graph = construct_test_module_graph();
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

  let diff_result = super::diff_module_graph(
    vec!["B".into(), "A".into()],
    &module_graph,
    &update_module_graph,
  );

  let added_modules = diff_result.added_modules;
  let removed_modules = diff_result.removed_modules;
  let diff_result = diff_result.deps_changes;

  assert_eq!(added_modules, HashSet::from(["H".into()]));

  assert_eq!(
    removed_modules,
    HashSet::from(["D".into(), "E".into(), "G".into()])
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
              ModuleGraphEdge {
                sources: vec!["./D".to_string()],
                ..Default::default()
              }
            ),
            (
              "E".into(),
              ModuleGraphEdge {
                sources: vec!["./E".to_string()],
                order: 1,
                ..Default::default()
              }
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
        "D".into(),
        ModuleDepsDiffResult {
          added: vec![],
          removed: vec![(
            "F".into(),
            ModuleGraphEdge {
              sources: vec!["./F".to_string()],
              kind: ResolveKind::DynamicImport,
              ..Default::default()
            }
          )],
        }
      ),
      (
        "E".into(),
        ModuleDepsDiffResult {
          added: vec![],
          removed: vec![(
            "G".into(),
            ModuleGraphEdge {
              sources: vec!["./G".to_string()],
              kind: ResolveKind::DynamicImport,
              ..Default::default()
            }
          )],
        }
      ),
      (
        "A".into(),
        ModuleDepsDiffResult {
          added: vec![],
          removed: vec![(
            "D".into(),
            ModuleGraphEdge {
              kind: ResolveKind::DynamicImport,
              sources: vec!["./D".to_string()],
              order: 1,
            }
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
  assert_eq!(diff_result.added_modules, HashSet::from(["H".into()]));
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
            ModuleGraphEdge {
              sources: vec!["./A".to_string()],
              ..Default::default()
            }
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
