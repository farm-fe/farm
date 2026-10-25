use std::path::PathBuf;

use farmfe_core::{
  glob::{Glob, WalkOptions},
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdgeDataItem},
    module_group::{ModuleGroup, ModuleGroupGraph, ModuleGroupId, ModuleGroupType},
    Module, ModuleMetaData, ModuleType,
  },
  plugin::ResolveKind,
  relative_path::RelativePath,
  HashMap,
};

pub mod assert;
pub use insta;
use insta::Settings;

pub struct InstaHelper {}

impl InstaHelper {
  pub fn create_setting() -> Settings {
    let mut setting = Settings::clone_current();
    setting.set_sort_maps(true);
    setting.set_omit_expression(true);
    setting.set_input_file(file!());
    setting.set_prepend_module_to_snapshot(false);
    setting
  }
}

#[macro_export]
macro_rules! assert_debug_snapshot {
  ($ex:expr) => {
    farmfe_testing_helpers::InstaHelper::create_setting()
      .bind(|| farmfe_testing_helpers::insta::assert_debug_snapshot!($ex));
  };
}

#[macro_export]
macro_rules! assert_snapshot {
  ($ex:expr) => {
    farmfe_testing_helpers::InstaHelper::create_setting()
      .bind(|| farmfe_testing_helpers::insta::assert_snapshot!($ex));
  };
}

pub fn is_update_snapshot_from_env() -> bool {
  std::env::var("FARM_UPDATE_SNAPSHOTS").is_ok() || std::env::var("INSTA_UPDATE").is_ok()
}

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
    let mut m = Module::new(id);
    m.module_type = ModuleType::Js;
    m.meta = Box::new(ModuleMetaData::Script(Box::default()));

    graph.add_module(m);
  }

  let static_edges = vec![("A", "C", 0), ("B", "D", 0), ("B", "E", 1)];
  let dynamic_edges = vec![("A", "D", 1), ("C", "F", 0), ("D", "F", 0), ("E", "G", 0)];

  for (from, to, order) in static_edges {
    graph
      .add_edge_item(
        &from.into(),
        &to.into(),
        ModuleGraphEdgeDataItem {
          source: format!("./{to}"),
          kind: ResolveKind::Import,
          order,
        },
      )
      .unwrap();
  }

  for (from, to, order) in dynamic_edges {
    graph
      .add_edge_item(
        &from.into(),
        &to.into(),
        ModuleGraphEdgeDataItem {
          source: format!("./{to}"),
          kind: ResolveKind::DynamicImport,
          order,
        },
      )
      .unwrap();
  }

  graph
    .add_edge_item(
      &"F".into(),
      &"A".into(),
      ModuleGraphEdgeDataItem {
        source: "./A".to_string(),
        kind: ResolveKind::Import,
        order: 0,
      },
    )
    .unwrap();

  graph.entries =
    HashMap::from_iter([("A".into(), "A".to_string()), ("B".into(), "B".to_string())]);

  graph
}

/// construct a test module group graph using module graph like below:
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
pub fn construct_test_module_group_graph() -> ModuleGroupGraph {
  let mut module_group_graph = ModuleGroupGraph::new();

  let mut module_group_a = ModuleGroup::new("A".into(), ModuleGroupType::Entry);
  let module_group_a_id = module_group_a.id.clone();
  module_group_a.add_module("A".into());
  module_group_a.add_module("C".into());
  module_group_graph.add_module_group(module_group_a);

  let mut module_group_b = ModuleGroup::new("B".into(), ModuleGroupType::Entry);
  let module_group_b_id = module_group_b.id.clone();
  module_group_b.add_module("B".into());
  module_group_b.add_module("D".into());
  module_group_b.add_module("E".into());
  module_group_graph.add_module_group(module_group_b);

  let mut module_group_d = ModuleGroup::new("D".into(), ModuleGroupType::DynamicImport);
  let module_group_d_id = module_group_d.id.clone();
  module_group_d.add_module("D".into());
  module_group_graph.add_module_group(module_group_d);

  let mut module_group_f = ModuleGroup::new("F".into(), ModuleGroupType::DynamicImport);
  let module_group_f_id = module_group_f.id.clone();
  module_group_f.add_module("F".into());
  module_group_f.add_module("A".into());
  module_group_f.add_module("C".into());
  module_group_graph.add_module_group(module_group_f);

  let mut module_group_g = ModuleGroup::new("G".into(), ModuleGroupType::DynamicImport);
  let module_group_g_id = module_group_g.id.clone();
  module_group_g.add_module("G".into());
  module_group_graph.add_module_group(module_group_g);

  let edges = vec![
    (module_group_a_id.clone(), module_group_d_id.clone()),
    (module_group_a_id.clone(), module_group_f_id.clone()),
    (module_group_d_id.clone(), module_group_f_id.clone()),
    (module_group_b_id.clone(), module_group_f_id.clone()),
    (module_group_b_id.clone(), module_group_g_id.clone()),
    (module_group_f_id.clone(), module_group_d_id.clone()),
    (module_group_f_id.clone(), module_group_f_id.clone()),
  ];

  for (from, to) in edges {
    module_group_graph.add_edge(&from, &to);
  }

  module_group_graph
}

/// construct a test module graph like below:
/// ```plain
///           A   B
///          /|\ / \
///         C | D   E
///          \|/ \  |
///           F  |  G
///            \ | /
///             \|/
///              H
/// ```
/// * **dynamic dependencies**: `A -> D`, `C -> F`, `D -> F`, `E -> G`
/// * **cyclic dependencies**: `F -> A`
/// * others are static dependencies
pub fn construct_test_module_graph_complex() -> ModuleGraph {
  let mut test_module_graph = construct_test_module_graph();
  let mut module_h = Module::new("H".into());
  module_h.module_type = ModuleType::Js;
  test_module_graph.add_module(module_h);

  let static_edges = vec![("D", "H", 1), ("F", "H", 0), ("G", "H", 0)];

  for (from, to, order) in static_edges {
    test_module_graph
      .add_edge_item(
        &from.into(),
        &to.into(),
        ModuleGraphEdgeDataItem {
          source: format!("./{to}"),
          kind: ResolveKind::Import,
          order,
        },
      )
      .unwrap();
  }

  test_module_graph.update_execution_order_for_modules();

  test_module_graph
}

/// construct a test module group graph using module graph like below:
/// ```plain
///           A   B
///          /|\ / \
///         C | D   E
///          \|/ \  |
///           F  |  G
///            \ | /
///             \|/
///              H
/// ```
/// * **dynamic dependencies**: `A -> D`, `C -> F`, `D -> F`, `E -> G`
/// * **cyclic dependencies**: `F -> A`
/// * others are static dependencies
pub fn construct_test_module_group_graph_complex() -> ModuleGroupGraph {
  let mut module_group_graph = construct_test_module_group_graph();

  let module_group_b = module_group_graph
    .module_group_mut(&ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry))
    .unwrap();
  module_group_b.add_module("H".into());

  let module_group_d = module_group_graph
    .module_group_mut(&ModuleGroupId::new(
      &"D".into(),
      &ModuleGroupType::DynamicImport,
    ))
    .unwrap();
  module_group_d.add_module("H".into());

  let module_group_f = module_group_graph
    .module_group_mut(&ModuleGroupId::new(
      &"F".into(),
      &ModuleGroupType::DynamicImport,
    ))
    .unwrap();
  module_group_f.add_module("H".into());

  let module_group_g = module_group_graph
    .module_group_mut(&ModuleGroupId::new(
      &"G".into(),
      &ModuleGroupType::DynamicImport,
    ))
    .unwrap();
  module_group_g.add_module("H".into());

  module_group_graph
}

/// @deprecated using macro fixture instead
pub fn fixture<F>(pattern: &str, mut op: F)
where
  F: FnMut(PathBuf, PathBuf),
{
  let base_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
  // Fixture discovery includes hidden paths and dependency-named fixture folders.
  // Production import.meta.glob keeps its separate, non-exhaustive default.
  let glob = Glob::with_options(
    pattern,
    WalkOptions {
      exhaustive: true,
      ..WalkOptions::default()
    },
  )
  .unwrap_or_else(|error| panic!("invalid fixture glob {pattern}: {error}"));

  for entry in glob.walk(&base_dir) {
    let path = entry.unwrap_or_else(|error| {
      panic!(
        "failed to walk fixtures under {}: {error}",
        base_dir.display()
      )
    });
    op(path.path().to_path_buf(), base_dir.clone());
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
  let test_path = std::path::Path::new(test_file_path);
  let mut file_path = if test_path.is_absolute() {
    test_path.to_path_buf()
  } else {
    RelativePath::new(test_file_path).to_logical_path(std::env::current_dir().unwrap())
  };
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
  let glob = Glob::with_options(
    pattern,
    WalkOptions {
      exhaustive: true,
      ..WalkOptions::default()
    },
  )
  .unwrap_or_else(|error| panic!("invalid fixture glob {pattern}: {error}"));

  let mut exists = false;

  for entry in glob.walk(&base_dir) {
    let path = entry.unwrap_or_else(|error| {
      panic!(
        "failed to walk fixtures under {}: {error}",
        base_dir.display()
      )
    });
    exists = true;
    op(path.path().to_path_buf(), base_dir.clone());
  }

  if !exists {
    panic!("no fixtures found under {pattern}");
  }
}

#[cfg(test)]
mod glob_migration_tests {
  use super::fixture_debug;

  fn fixture_root() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("Cargo.toml"), "# fixture marker").unwrap();
    std::fs::create_dir_all(root.path().join("src")).unwrap();
    std::fs::write(root.path().join("src/lib.rs"), "// fixture marker").unwrap();
    root
  }

  #[test]
  fn fixture_glob_keeps_hidden_and_dependency_named_fixture_folders() {
    let root = fixture_root();
    for directory in [
      "tests/normal",
      "tests/.hidden",
      "tests/node_modules/example",
    ] {
      let directory = root.path().join(directory);
      std::fs::create_dir_all(&directory).unwrap();
      std::fs::write(directory.join("input.js"), "export default 1").unwrap();
    }
    let mut files = Vec::new();
    fixture_debug(
      "tests/**/input.js",
      root.path().join("src/lib.rs").to_str().unwrap(),
      |file, base| {
        assert_eq!(base, root.path());
        files.push(
          file
            .strip_prefix(base)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/"),
        );
      },
    );
    files.sort();
    assert_eq!(
      files,
      vec![
        "tests/.hidden/input.js",
        "tests/node_modules/example/input.js",
        "tests/normal/input.js"
      ]
    );
  }

  #[test]
  #[should_panic(expected = "invalid fixture glob")]
  fn fixture_glob_reports_invalid_patterns() {
    let root = fixture_root();
    fixture_debug(
      "",
      root.path().join("src/lib.rs").to_str().unwrap(),
      |_, _| {},
    );
  }

  #[cfg(unix)]
  #[test]
  #[should_panic(expected = "failed to walk fixtures")]
  fn fixture_glob_does_not_hide_walk_errors() {
    let root = fixture_root();
    std::fs::create_dir_all(root.path().join("tests")).unwrap();
    std::os::unix::fs::symlink(root.path().join("tests"), root.path().join("tests/loop")).unwrap();
    fixture_debug(
      "tests/**/*.js",
      root.path().join("src/lib.rs").to_str().unwrap(),
      |_, _| {},
    );
  }
}
