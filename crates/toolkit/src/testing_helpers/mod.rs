use std::path::PathBuf;

use farmfe_core::{
  hashbrown::HashSet,
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdge},
    Module, ModuleType,
  },
  plugin::ResolveKind,
  relative_path::RelativePath,
};
use glob::glob;

/// construct a test module graph like below:
/// ```plain
///           A   B
///          / \ / \
///         C   D   E
///          \ /    |
///           F     G
/// ```
/// * **dynamic dependencies**: `A -> D`, `C -> F`, `D -> F`, `E -> G`
/// * **cyclic dependencies**: `F -> A`
/// * others are static dependencies
pub fn construct_test_module_graph() -> ModuleGraph {
  let module_ids = vec!["A", "B", "C", "D", "E", "F", "G"]
    .into_iter()
    .map(|i| i.into());
  let mut graph = ModuleGraph::new();

  for id in module_ids {
    let m = Module::new(id);

    graph.add_module(m);
  }

  let static_edges = vec![("A", "C", 0), ("B", "D", 0), ("B", "E", 1)];
  let dynamic_edges = vec![("A", "D", 1), ("C", "F", 0), ("D", "F", 0), ("E", "G", 0)];

  for (from, to, order) in static_edges {
    graph
      .add_edge(
        &from.into(),
        &to.into(),
        ModuleGraphEdge {
          source: format!("./{}", to),
          kind: ResolveKind::Import,
          order,
        },
      )
      .unwrap();
  }

  for (from, to, order) in dynamic_edges {
    graph
      .add_edge(
        &from.into(),
        &to.into(),
        ModuleGraphEdge {
          source: format!("./{}", to),
          kind: ResolveKind::DynamicImport,
          order,
        },
      )
      .unwrap();
  }

  graph
    .add_edge(
      &"F".into(),
      &"A".into(),
      ModuleGraphEdge {
        source: "./F".to_string(),
        kind: ResolveKind::Import,
        order: 0,
      },
    )
    .unwrap();

  graph.entries = HashSet::from(["A".into(), "B".into()]);

  graph
}

pub fn fixture<F>(pattern: &str, mut op: F)
where
  F: FnMut(PathBuf, PathBuf),
{
  let base_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
  let abs_pattern = RelativePath::new(pattern).to_path(base_dir.clone());
  let paths = glob(&abs_pattern.to_string_lossy()).unwrap();

  for path in paths {
    op(path.unwrap(), base_dir.clone());
  }
}
