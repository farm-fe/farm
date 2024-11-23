use std::collections::HashSet;

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
  // test module A, B, C, D, E, F, G, H
  let module_a = module_graph.module(&"A".into()).unwrap();
  assert_eq!(
    module_a.module_groups,
    HashSet::from_iter(["A".into(), "F".into()])
  );
  let module_b = module_graph.module(&"B".into()).unwrap();
  assert_eq!(module_b.module_groups, HashSet::from_iter(["B".into()]));
  let module_c = module_graph.module(&"C".into()).unwrap();
  assert_eq!(
    module_c.module_groups,
    HashSet::from_iter(["A".into(), "F".into()])
  );
  let module_d = module_graph.module(&"D".into()).unwrap();
  assert_eq!(
    module_d.module_groups,
    HashSet::from_iter(["B".into(), "D".into()])
  );
  let module_e = module_graph.module(&"E".into()).unwrap();
  assert_eq!(module_e.module_groups, HashSet::from_iter(["B".into()]));
  let module_f = module_graph.module(&"F".into()).unwrap();
  assert_eq!(module_f.module_groups, HashSet::from_iter(["F".into()]));
  let module_g = module_graph.module(&"G".into()).unwrap();
  assert_eq!(module_g.module_groups, HashSet::from_iter(["G".into()]));
  let module_h = module_graph.module(&"H".into()).unwrap();
  assert_eq!(
    module_h.module_groups,
    HashSet::from_iter(["G".into(), "F".into(), "B".into(), "D".into()])
  );
}

#[test]
fn module_group_graph_toposort() {
  let module_group_graph = construct_test_module_group_graph_complex();
  let toposorted = module_group_graph.toposort(vec!["A".into(), "B".into()]);

  assert_eq!(
    toposorted,
    vec!["B".into(), "G".into(), "A".into(), "D".into(), "F".into()]
  );
}
