use std::path::PathBuf;

use farmfe_core::{
  hashbrown::HashSet,
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdge},
    Module,
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
        source: "./A".to_string(),
        kind: ResolveKind::Import,
        order: 0,
      },
    )
    .unwrap();

  graph.entries = HashSet::from(["A".into(), "B".into()]);

  graph
}

/// @deprecated using macro fixture instead
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

#[macro_export]
macro_rules! fixture {
  ($pattern:expr, $op:expr) => {
    if cfg!(debug_assertions) {
      farmfe_testing_helpers::fixture_debug($pattern, file!(), $op);
      return;
    }

    farmfe_testing_helpers::fixture($pattern, $op);
  };
}

/// @deprecated using macro fixture instead
pub fn fixture_debug<F>(pattern: &str, test_file_path: &str, mut op: F)
where
  F: FnMut(PathBuf, PathBuf),
{
  // find closest Cargo.toml
  let mut file_path =
    RelativePath::new(test_file_path).to_logical_path(std::env::current_dir().unwrap());
  while let Some(parent) = file_path.parent() {
    if parent.join("Cargo.toml").exists() {
      break;
    }

    file_path = parent.to_path_buf();
  }

  if file_path.parent().is_none() {
    panic!("can't find Cargo.toml");
  }

  let base_dir = file_path.parent().unwrap().to_path_buf();
  let abs_pattern = RelativePath::new(pattern).to_path(base_dir.clone());
  let paths = glob(&abs_pattern.to_string_lossy()).unwrap();

  for path in paths {
    op(path.unwrap(), base_dir.clone());
  }
}
