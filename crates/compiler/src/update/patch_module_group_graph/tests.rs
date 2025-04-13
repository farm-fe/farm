use farmfe_core::{
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdge, ModuleGraphEdgeDataItem},
    module_group::{ModuleGroupId, ModuleGroupType},
    Module,
  },
  plugin::ResolveKind,
  HashMap, HashSet,
};
use farmfe_plugin_partial_bundling::{
  module_group_graph_from_entries, module_group_graph_from_module_graph,
};
use farmfe_testing_helpers::{construct_test_module_graph, construct_test_module_graph_complex};

use crate::update::diff_and_patch_module_graph::{diff_module_graph, patch_module_graph};

use super::patch_module_group_graph;

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

  module_graph
}

#[test]
fn test_patch_module_group_graph_1() {
  let mut module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();
  update_module_graph.remove_module(&"F".into());
  update_module_graph.remove_module(&"G".into());
  update_module_graph
    .remove_edge(&"A".into(), &"D".into())
    .unwrap();
  let entries = vec!["A".into(), "B".into()];
  let start_points = vec!["A".into(), "C".into(), "D".into(), "E".into()];
  let mut module_group_graph = module_group_graph_from_entries(&entries, &mut module_graph);

  let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);
  let removed_modules = patch_module_graph(
    start_points.clone(),
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  let affected_groups = patch_module_group_graph(
    start_points.clone(),
    &diff_result,
    &removed_modules,
    &mut module_graph,
    &mut module_group_graph,
  );

  let group_id_a = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([group_id_a.clone(), group_id_b.clone()])
  );

  let update_module_group_graph = module_group_graph_from_entries(&entries, &mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  let module_a = module_graph.module(&"A".into()).unwrap();
  assert_eq!(
    module_a.module_groups,
    HashSet::from_iter([group_id_a.clone()])
  );
  let module_b = module_graph.module(&"B".into()).unwrap();
  assert_eq!(
    module_b.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_c = module_graph.module(&"C".into()).unwrap();
  assert_eq!(module_c.module_groups, HashSet::from_iter([group_id_a]));
  let module_d = module_graph.module(&"D".into()).unwrap();
  assert_eq!(
    module_d.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_e = module_graph.module(&"E".into()).unwrap();
  assert_eq!(
    module_e.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
}

#[test]
fn test_patch_module_group_graph_2() {
  let mut module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();

  update_module_graph.remove_module(&"D".into());
  update_module_graph.remove_module(&"E".into());
  update_module_graph.remove_module(&"G".into());
  update_module_graph
    .remove_edge(&"C".into(), &"F".into())
    .unwrap();
  update_module_graph.add_module(Module::new("H".into()));
  update_module_graph
    .add_edge(&"B".into(), &"H".into(), Default::default())
    .unwrap();
  update_module_graph
    .add_edge(&"H".into(), &"F".into(), Default::default())
    .unwrap();

  let start_points = vec!["B".into(), "A".into()];
  let updated_modules = vec!["B".into(), "A".into()];

  let mut module_group_graph = module_group_graph_from_entries(&start_points, &mut module_graph);

  let diff_result = diff_module_graph(updated_modules.clone(), &module_graph, &update_module_graph);
  let removed_modules = patch_module_graph(
    updated_modules.clone(),
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  let affected_groups = patch_module_group_graph(
    updated_modules.clone(),
    &diff_result,
    &removed_modules,
    &mut module_graph,
    &mut module_group_graph,
  );

  let group_id_a = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);
  let group_id_f = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([group_id_a.clone(), group_id_b.clone(), group_id_f.clone()])
  );
  let module_group_b = module_group_graph.module_group(&group_id_b).unwrap();
  assert_eq!(
    module_group_b.modules(),
    &HashSet::from_iter(["B".into(), "H".into(), "F".into(), "C".into(), "A".into()])
  );

  let update_module_group_graph = module_group_graph_from_entries(&start_points, &mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  let module_a = module_graph.module(&"A".into()).unwrap();
  assert_eq!(
    module_a.module_groups,
    HashSet::from_iter([group_id_a.clone(), group_id_b.clone(), group_id_f.clone()])
  );
  let module_b = module_graph.module(&"B".into()).unwrap();
  assert_eq!(
    module_b.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_c = module_graph.module(&"C".into()).unwrap();
  assert_eq!(
    module_c.module_groups,
    HashSet::from_iter([group_id_a.clone(), group_id_b.clone(), group_id_f.clone()])
  );
  let module_f = module_graph.module(&"F".into()).unwrap();
  assert_eq!(
    module_f.module_groups,
    HashSet::from_iter([group_id_b.clone(), group_id_f.clone()])
  );
  let module_h = module_graph.module(&"H".into()).unwrap();
  assert_eq!(
    module_h.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
}

#[test]
fn test_patch_module_group_graph_3() {
  let mut module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();
  update_module_graph.remove_module(&"C".into());
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

  let updated_modules = vec!["F".into(), "E".into(), "B".into()];
  let mut module_group_graph = module_group_graph_from_entries(
    &module_graph.entries.clone().into_keys().collect(),
    &mut module_graph,
  );
  let diff_result = diff_module_graph(updated_modules.clone(), &module_graph, &update_module_graph);

  let removed_modules = patch_module_graph(
    updated_modules.clone(),
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  let affected_groups = patch_module_group_graph(
    updated_modules,
    &diff_result,
    &removed_modules,
    &mut module_graph,
    &mut module_group_graph,
  );

  let group_id_a = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);
  let group_id_d = ModuleGroupId::new(&"D".into(), &ModuleGroupType::DynamicImport);
  let group_id_f = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([
      group_id_a.clone(),
      group_id_b.clone(),
      group_id_d.clone(),
      group_id_f.clone()
    ])
  );

  let update_module_group_graph = module_group_graph_from_entries(
    &module_graph.entries.clone().into_keys().collect(),
    &mut module_graph,
  );

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  let module_a = module_graph.module(&"A".into()).unwrap();
  assert_eq!(
    module_a.module_groups,
    HashSet::from_iter([group_id_a.clone()])
  );
  let module_b = module_graph.module(&"B".into()).unwrap();
  assert_eq!(
    module_b.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_c = module_graph.module(&"C".into()).unwrap();
  assert_eq!(
    module_c.module_groups,
    HashSet::from_iter([group_id_a.clone()])
  );
  let module_d = module_graph.module(&"D".into()).unwrap();
  assert_eq!(
    module_d.module_groups,
    HashSet::from_iter([group_id_b.clone(), group_id_d.clone()])
  );
  let module_e = module_graph.module(&"E".into()).unwrap();
  assert_eq!(
    module_e.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_f = module_graph.module(&"F".into()).unwrap();
  assert_eq!(
    module_f.module_groups,
    HashSet::from_iter([group_id_f.clone(), group_id_b.clone()])
  );
  let module_h = module_graph.module(&"H".into()).unwrap();
  assert_eq!(
    module_h.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
}

fn get_edge_info(kind: ResolveKind) -> ModuleGraphEdge {
  ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
    kind,
    ..Default::default()
  }])
}

#[test]
fn test_patch_module_group_graph_css_modules() {
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

  let start_points = vec!["D".into()];
  let mut module_group_graph = module_group_graph_from_entries(
    &module_graph.entries.clone().into_keys().collect(),
    &mut module_graph,
  );
  let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);
  let removed_modules = patch_module_graph(
    start_points.clone(),
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  let affected_groups = patch_module_group_graph(
    start_points,
    &diff_result,
    &removed_modules,
    &mut module_graph,
    &mut module_group_graph,
  );

  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);
  let group_id_d = ModuleGroupId::new(&"D".into(), &ModuleGroupType::DynamicImport);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([group_id_b.clone(), group_id_d.clone()])
  );

  let update_module_group_graph = module_group_graph_from_entries(
    &module_graph.entries.clone().into_keys().collect(),
    &mut module_graph,
  );

  assert_eq!(module_group_graph, update_module_group_graph);
}

#[test]
fn test_patch_module_group_graph_add_and_remove() {
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

  let start_points = vec!["a".into()];
  let mut module_group_graph = module_group_graph_from_entries(&start_points, &mut module_graph);
  let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);
  let removed_modules = patch_module_graph(
    start_points.clone(),
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  let affected_groups = patch_module_group_graph(
    start_points.clone(),
    &diff_result,
    &removed_modules,
    &mut module_graph,
    &mut module_group_graph,
  );

  let group_id_a = ModuleGroupId::new(&"a".into(), &ModuleGroupType::Entry);

  assert_eq!(affected_groups, HashSet::from_iter([group_id_a.clone()]));

  let update_module_group_graph = module_group_graph_from_entries(&start_points, &mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  for module in module_graph.modules() {
    assert_eq!(
      module.module_groups,
      HashSet::from_iter([group_id_a.clone()])
    );
  }
}

#[test]
fn test_patch_module_group_graph_remove_and_add() {
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
  let mut module_group_graph =
    module_group_graph_from_entries(&vec!["a".into()], &mut module_graph);

  let diff_result = diff_module_graph(vec!["a".into()], &module_graph, &update_module_graph);

  let removed_modules = patch_module_graph(
    vec!["a".into()],
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  let affected_groups = patch_module_group_graph(
    vec!["a".into()],
    &diff_result,
    &removed_modules,
    &mut module_graph,
    &mut module_group_graph,
  );

  let group_id_a = ModuleGroupId::new(&"a".into(), &ModuleGroupType::Entry);

  assert_eq!(affected_groups, HashSet::from_iter([group_id_a.clone()]));

  let update_module_group_graph =
    module_group_graph_from_entries(&vec!["a".into()], &mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  for module in module_graph.modules() {
    assert_eq!(
      module.module_groups,
      HashSet::from_iter([group_id_a.clone()])
    );
  }
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
  let mut module_group_graph =
    module_group_graph_from_entries(&vec!["a".into()], &mut module_graph);

  let diff_result = diff_module_graph(
    vec!["a".into(), "b".into()],
    &module_graph,
    &update_module_graph,
  );

  let removed_modules = patch_module_graph(
    vec!["a".into(), "b".into()],
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  let affected_groups = patch_module_group_graph(
    vec!["a".into(), "b".into()],
    &diff_result,
    &removed_modules,
    &mut module_graph,
    &mut module_group_graph,
  );

  let group_id_a = ModuleGroupId::new(&"a".into(), &ModuleGroupType::Entry);

  assert_eq!(affected_groups, HashSet::from_iter([group_id_a.clone()]));

  let update_module_group_graph =
    module_group_graph_from_entries(&vec!["a".into()], &mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  for module in module_graph.modules() {
    assert_eq!(
      module.module_groups,
      HashSet::from_iter([group_id_a.clone()])
    );
  }
}

#[test]
fn test_patch_module_group_graph_remove_normal_edge() {
  let mut module_graph = construct_test_module_graph_complex();
  let module_i = Module::new("I".into());
  module_graph.add_module(module_i);
  module_graph
    .add_edge(&"D".into(), &"I".into(), Default::default())
    .unwrap();
  let mut update_module_graph = construct_test_module_graph_complex();
  update_module_graph.remove_module(&"F".into());
  update_module_graph.remove_module(&"G".into());
  update_module_graph.remove_module(&"A".into());
  update_module_graph.remove_module(&"C".into());
  update_module_graph.remove_module(&"D".into());
  update_module_graph.remove_module(&"H".into());

  let entries = vec!["A".into(), "B".into()];
  let start_points = vec!["B".into()];
  let mut module_group_graph = module_group_graph_from_entries(&entries, &mut module_graph);

  let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);
  let removed_modules = patch_module_graph(
    start_points.clone(),
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  let affected_groups = patch_module_group_graph(
    start_points.clone(),
    &diff_result,
    &removed_modules,
    &mut module_graph,
    &mut module_group_graph,
  );

  let group_id_a = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  let group_id_d = ModuleGroupId::new(&"D".into(), &ModuleGroupType::DynamicImport);
  let group_id_g = ModuleGroupId::new(&"G".into(), &ModuleGroupType::DynamicImport);
  let group_id_f = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);
  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([
      group_id_d.clone(),
      group_id_b.clone(),
      group_id_g.clone(),
      group_id_a.clone(),
      group_id_f.clone()
    ])
  );

  let update_module_group_graph = module_group_graph_from_entries(&entries, &mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  let module_a = module_graph.module(&"A".into()).unwrap();
  assert_eq!(
    module_a.module_groups,
    HashSet::from_iter([group_id_a.clone(), group_id_f.clone()])
  );
  let module_b = module_graph.module(&"B".into()).unwrap();
  assert_eq!(
    module_b.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_c = module_graph.module(&"C".into()).unwrap();
  assert_eq!(
    module_c.module_groups,
    HashSet::from_iter([group_id_a, group_id_f.clone()])
  );
  let module_d = module_graph.module(&"D".into()).unwrap();
  assert_eq!(
    module_d.module_groups,
    HashSet::from_iter([group_id_d.clone()])
  );
  let module_e = module_graph.module(&"E".into()).unwrap();
  assert_eq!(
    module_e.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_h = module_graph.module(&"H".into()).unwrap();
  assert_eq!(
    module_h.module_groups,
    HashSet::from_iter([group_id_d.clone(), group_id_f.clone(), group_id_g.clone()])
  );
  let module_i = module_graph.module(&"I".into()).unwrap();
  assert_eq!(
    module_i.module_groups,
    HashSet::from_iter([group_id_d.clone()])
  );
}

#[test]
fn test_patch_module_group_graph_remove_dynamic_import_edge() {
  let mut module_graph = construct_test_module_graph_complex();
  let module_i = Module::new("I".into());
  module_graph.add_module(module_i);
  module_graph
    .add_edge(&"D".into(), &"I".into(), Default::default())
    .unwrap();
  let mut update_module_graph = construct_test_module_graph_complex();
  update_module_graph.remove_module(&"F".into());
  update_module_graph.remove_module(&"G".into());
  update_module_graph.remove_module(&"B".into());
  update_module_graph.remove_module(&"E".into());
  update_module_graph.remove_module(&"D".into());
  update_module_graph.remove_module(&"H".into());

  let entries = vec!["A".into(), "B".into()];
  let start_points = vec!["A".into()];
  let mut module_group_graph = module_group_graph_from_entries(&entries, &mut module_graph);

  let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);
  let removed_modules = patch_module_graph(
    start_points.clone(),
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  let affected_groups = patch_module_group_graph(
    start_points.clone(),
    &diff_result,
    &removed_modules,
    &mut module_graph,
    &mut module_group_graph,
  );

  let group_id_a = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  // let group_id_d = ModuleGroupId::new(&"D".into(), &ModuleGroupType::DynamicImport);
  let group_id_g = ModuleGroupId::new(&"G".into(), &ModuleGroupType::DynamicImport);
  let group_id_f = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);
  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([
      // group_id_d.clone(),
      group_id_b.clone(),
      group_id_g.clone(),
      group_id_a.clone(),
      group_id_f.clone()
    ])
  );

  let update_module_group_graph = module_group_graph_from_entries(&entries, &mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);

  // makes sure that module_groups field of each module is correct
  let module_a = module_graph.module(&"A".into()).unwrap();
  assert_eq!(
    module_a.module_groups,
    HashSet::from_iter([group_id_a.clone(), group_id_f.clone()])
  );
  let module_b = module_graph.module(&"B".into()).unwrap();
  assert_eq!(
    module_b.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_c = module_graph.module(&"C".into()).unwrap();
  assert_eq!(
    module_c.module_groups,
    HashSet::from_iter([group_id_a, group_id_f.clone()])
  );
  let module_d = module_graph.module(&"D".into()).unwrap();
  assert_eq!(
    module_d.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_e = module_graph.module(&"E".into()).unwrap();
  assert_eq!(
    module_e.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
  let module_h = module_graph.module(&"H".into()).unwrap();
  assert_eq!(
    module_h.module_groups,
    HashSet::from_iter([group_id_b.clone(), group_id_f.clone(), group_id_g.clone()])
  );
  let module_i = module_graph.module(&"I".into()).unwrap();
  assert_eq!(
    module_i.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );
}

#[test]
fn test_patch_module_group_graph_update_dynamic_import_edge() {
  let mut module_graph = construct_test_module_graph_complex();
  // make E -> G non-dynamic import, then in update_module_graph make it dynamic import
  module_graph
    .update_edge(
      &"E".into(),
      &"G".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::Import,
        ..Default::default()
      }]),
    )
    .unwrap();
  let mut update_module_graph = construct_test_module_graph_complex();
  update_module_graph
    .update_edge(
      &"B".into(),
      &"D".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::DynamicImport,
        ..Default::default()
      }]),
    )
    .unwrap();
  update_module_graph.remove_module(&"F".into());
  update_module_graph.remove_module(&"A".into());
  update_module_graph.remove_module(&"C".into());
  update_module_graph.remove_module(&"H".into());
  update_module_graph.entries =
    HashMap::from_iter([("B".into(), "B".to_string()), ("E".into(), "E".to_string())]);
  update_module_graph.update_execution_order_for_modules();

  let mut module_group_graph = module_group_graph_from_module_graph(&mut module_graph);
  let start_points = vec!["B".into(), "E".into()];
  let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);
  let removed_modules = patch_module_graph(
    start_points.clone(),
    &diff_result,
    &mut module_graph,
    &mut update_module_graph,
  );

  let affected_groups = patch_module_group_graph(
    start_points,
    &diff_result,
    &removed_modules,
    &mut module_graph,
    &mut module_group_graph,
  );

  let group_id_a = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);
  let group_id_f = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);
  let group_id_g = ModuleGroupId::new(&"G".into(), &ModuleGroupType::DynamicImport);
  let group_id_d = ModuleGroupId::new(&"D".into(), &ModuleGroupType::DynamicImport);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([
      group_id_a.clone(),
      group_id_b.clone(),
      group_id_f.clone(),
      group_id_g.clone(),
      group_id_d.clone()
    ])
  );

  let update_module_group_graph = module_group_graph_from_module_graph(&mut module_graph);

  assert_eq!(module_group_graph, update_module_group_graph);
}
