use std::collections::HashMap;

use farmfe_core::{
  module::{module_graph::{ModuleGraph, ModuleGraphEdgeDataItem}, Module},
  plugin::ResolveKind,
};
use farmfe_plugin_partial_bundling::module_group_graph_from_entries;

#[test]
fn entry_can_also_be_dynamic_import_target() {
  let mut graph = ModuleGraph::new();
  graph.add_module(Module::new("index.ts".into()));
  graph.add_module(Module::new("a.ts".into()));
  graph
    .add_edge_item(
      &"index.ts".into(),
      &"a.ts".into(),
      ModuleGraphEdgeDataItem {
        source: "./a".into(),
        kind: ResolveKind::DynamicImport,
        order: 0,
      },
    )
    .unwrap();
  graph.entries = HashMap::from([
    ("index.ts".into(), "main".into()),
    ("a.ts".into(), "a".into()),
  ]);

  let graph_groups = module_group_graph_from_entries(
    &graph.entries.keys().cloned().collect(),
    &mut graph,
  );

  assert_eq!(graph_groups.len(), 2);
  assert!(graph_groups.has(&"index.ts".into()));
  assert!(graph_groups.has(&"a.ts".into()));
  assert!(graph_groups.has_edge(&"index.ts".into(), &"a.ts".into()));
}
