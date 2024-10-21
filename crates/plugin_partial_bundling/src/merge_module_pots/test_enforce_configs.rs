use farmfe_core::config::{partial_bundling::PartialBundlingConfig, Config};
use farmfe_testing_helpers::{assert_resource_pots, construct_test_module_graph_complex};

use super::{common::create_test_module_pot, merge_module_pots, ModuleGroupModulePots};

#[test]
fn test_enforce_configs_min_size() {
  let mut module_graph = construct_test_module_graph_complex();
  let mut module_group_module_pots = ModuleGroupModulePots::new("B".into());

  let size = 10 * 1024;
  let module_bucket_b = create_test_module_pot(&mut module_graph, &"B".into(), "B", size, false);
  let module_bucket_e = create_test_module_pot(&mut module_graph, &"E".into(), "E", size, false);
  let module_bucket_d = create_test_module_pot(&mut module_graph, &"D".into(), "D", size * 2, true);
  let module_bucket_h = create_test_module_pot(&mut module_graph, &"H".into(), "H", size, true);

  module_group_module_pots
    .add_module_pots("B_E".to_string(), vec![module_bucket_b, module_bucket_e]);
  module_group_module_pots.add_module_pots("D".to_string(), vec![module_bucket_d]);
  module_group_module_pots.add_module_pots("H".to_string(), vec![module_bucket_h]);

  let mut resource_pots = merge_module_pots(
    module_group_module_pots,
    &Config {
      partial_bundling: Box::new(PartialBundlingConfig {
        target_concurrent_requests: 2,
        target_min_size: 20 * 1024,
        enforce_target_min_size: true,
        ..Default::default()
      }),
      ..Default::default()
    },
    "B",
    &module_graph,
  );

  resource_pots.sort_by_key(|p| p.id.clone());

  assert_eq!(resource_pots.len(), 2);
  assert_resource_pots!(resource_pots)
}

#[test]
fn test_enforce_configs_concurrent_requests() {
  let mut module_graph = construct_test_module_graph_complex();
  let mut module_group_module_pots = ModuleGroupModulePots::new("B".into());

  let size = 10 * 1024;
  let module_bucket_b = create_test_module_pot(&mut module_graph, &"B".into(), "B", size, false);
  let module_bucket_e = create_test_module_pot(&mut module_graph, &"E".into(), "E", size, false);
  let module_bucket_d = create_test_module_pot(&mut module_graph, &"D".into(), "D", size * 2, true);
  let module_bucket_h = create_test_module_pot(&mut module_graph, &"H".into(), "H", size, true);

  module_group_module_pots
    .add_module_pots("B_E".to_string(), vec![module_bucket_b, module_bucket_e]);
  module_group_module_pots.add_module_pots("D".to_string(), vec![module_bucket_d]);
  module_group_module_pots.add_module_pots("H".to_string(), vec![module_bucket_h]);

  let mut resource_pots = merge_module_pots(
    module_group_module_pots,
    &Config {
      partial_bundling: Box::new(PartialBundlingConfig {
        target_concurrent_requests: 2,
        target_min_size: 20 * 1024,
        enforce_target_concurrent_requests: true,
        ..Default::default()
      }),
      ..Default::default()
    },
    "B",
    &module_graph,
  );

  resource_pots.sort_by_key(|p| p.id.clone());

  assert_eq!(resource_pots.len(), 2);
  assert_resource_pots!(resource_pots);
}
