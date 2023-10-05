use farmfe_core::config::partial_bundling::PartialBundlingConfig;
use farmfe_testing_helpers::construct_test_module_graph_complex;

use super::{common::create_test_module_pot, merge_module_pots, ModuleGroupModulePots};

#[test]
fn test_default() {
  let mut module_graph = construct_test_module_graph_complex();
  let mut module_group_module_pots = ModuleGroupModulePots::new("B".into());

  let size = 10 * 1024;
  let module_bucket_b = create_test_module_pot(&mut module_graph, &"B".into(), size, false);
  let module_bucket_e = create_test_module_pot(&mut module_graph, &"E".into(), size, false);
  let module_bucket_d = create_test_module_pot(&mut module_graph, &"D".into(), size * 2, true);
  let module_bucket_h = create_test_module_pot(&mut module_graph, &"H".into(), size, true);

  module_group_module_pots
    .add_module_pots("B_E".to_string(), vec![module_bucket_b, module_bucket_e]);
  module_group_module_pots.add_module_pots("D".to_string(), vec![module_bucket_d]);
  module_group_module_pots.add_module_pots("H".to_string(), vec![module_bucket_h]);

  let mut resource_pots = merge_module_pots(
    module_group_module_pots,
    &PartialBundlingConfig {
      target_concurrent_requests: 2,
      target_min_size: 20 * 1024,
      ..Default::default()
    },
    "B",
    &module_graph,
  );

  resource_pots.sort_by_key(|p| p.id.clone());

  assert_eq!(resource_pots.len(), 3);
  assert_eq!(resource_pots[0].immutable, false);
  assert_eq!(resource_pots[0].modules(), vec![&"B".into(), &"E".into()]);

  assert_eq!(resource_pots[1].immutable, true);
  assert_eq!(resource_pots[1].modules(), vec![&"D".into()]);

  assert_eq!(resource_pots[2].immutable, true);
  assert_eq!(resource_pots[2].modules(), vec![&"H".into()]);
}
