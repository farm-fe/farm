use std::path::PathBuf;

use farmfe_core::{
  module::{
    meta_data::script::ScriptModuleMetaData,
    module_graph::{ModuleGraph, ModuleGraphEdgeDataItem},
    module_group::{ModuleGroup, ModuleGroupGraph, ModuleGroupId, ModuleGroupType},
    Module, ModuleMetaData, ModuleType,
  },
  plugin::ResolveKind,
  relative_path::RelativePath,
  wax::Glob,
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
    m.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData::default()));

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
  let glob = Glob::new(pattern).unwrap();

  for path in glob.walk(base_dir.clone()).flatten() {
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
  let glob = Glob::new(pattern).unwrap();

  let mut exists = false;

  for path in glob.walk(base_dir.clone()).flatten() {
    exists = true;
    op(path.path().to_path_buf(), base_dir.clone());
  }

  if !exists {
    panic!("no fixtures found under {pattern}");
  }
}
