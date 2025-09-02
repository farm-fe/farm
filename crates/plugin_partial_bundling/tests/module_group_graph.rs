use farmfe_core::{
  module::module_group::{ModuleGroupId, ModuleGroupType},
  HashSet,
};
use farmfe_plugin_partial_bundling::module_group_graph_from_entries;
use farmfe_testing_helpers::{
  construct_test_module_graph_complex, construct_test_module_group_graph_complex,
};

#[test]
fn module_group_graph() {
  let mut module_graph = construct_test_module_graph_complex();
  let module_group_graph =
    module_group_graph_from_entries(&vec!["A".into(), "B".into()], &mut module_graph);

  assert_eq!(
    module_group_graph,
    construct_test_module_group_graph_complex()
  );
  let group_a_id = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  let group_b_id = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);
  let group_d_id = ModuleGroupId::new(&"D".into(), &ModuleGroupType::DynamicImport);
  let group_f_id = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);
  let group_g_id = ModuleGroupId::new(&"G".into(), &ModuleGroupType::DynamicImport);

  // test module A, B, C, D, E, F, G, H
  let module_a = module_graph.module(&"A".into()).unwrap();
  assert_eq!(
    module_a.module_groups,
    HashSet::from_iter([group_a_id.clone(), group_f_id.clone()])
  );
  let module_b = module_graph.module(&"B".into()).unwrap();
  assert_eq!(
    module_b.module_groups,
    HashSet::from_iter([group_b_id.clone()])
  );
  let module_c = module_graph.module(&"C".into()).unwrap();
  assert_eq!(
    module_c.module_groups,
    HashSet::from_iter([group_a_id.clone(), group_f_id.clone()])
  );
  let module_d = module_graph.module(&"D".into()).unwrap();
  assert_eq!(
    module_d.module_groups,
    HashSet::from_iter([group_b_id.clone(), group_d_id.clone()])
  );
  let module_e = module_graph.module(&"E".into()).unwrap();
  assert_eq!(
    module_e.module_groups,
    HashSet::from_iter([group_b_id.clone()])
  );
  let module_f = module_graph.module(&"F".into()).unwrap();
  assert_eq!(
    module_f.module_groups,
    HashSet::from_iter([group_f_id.clone()])
  );
  let module_g = module_graph.module(&"G".into()).unwrap();
  assert_eq!(
    module_g.module_groups,
    HashSet::from_iter([group_g_id.clone()])
  );
  let module_h = module_graph.module(&"H".into()).unwrap();
  assert_eq!(
    module_h.module_groups,
    HashSet::from_iter([group_g_id, group_f_id, group_b_id, group_d_id])
  );
}

#[test]
fn module_group_graph_toposort() {
  let module_group_graph = construct_test_module_group_graph_complex();
  let group_a_id = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  let group_b_id = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);
  let group_d_id = ModuleGroupId::new(&"D".into(), &ModuleGroupType::DynamicImport);
  let group_f_id = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);
  let group_g_id = ModuleGroupId::new(&"G".into(), &ModuleGroupType::DynamicImport);

  let toposorted = module_group_graph.toposort(vec![group_a_id.clone(), group_b_id.clone()]);

  assert_eq!(
    toposorted,
    vec![
      group_b_id.clone(),
      group_g_id.clone(),
      group_a_id.clone(),
      group_d_id.clone(),
      group_f_id.clone()
    ]
  );
}
