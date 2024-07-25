use std::collections::HashSet;
use std::sync::Arc;

use farmfe_core::{
  config::{
    config_regex::ConfigRegex, partial_bundling::PartialBundlingEnforceResourceConfig, Config,
  },
  context::CompilationContext,
  module::{
    module_graph::{ModuleGraphEdge, ModuleGraphEdgeDataItem},
    Module,
  },
  plugin::{Plugin, PluginHookContext, ResolveKind},
};
use farmfe_plugin_partial_bundling::module_group_graph_from_entries;
use farmfe_testing_helpers::{
  assert_debug_snapshot, assert_resource_pots, assert_sorted_iter_eq, construct_test_module_graph,
  construct_test_module_graph_complex,
};

use crate::{
  generate::partial_bundling::generate_resource_pot_map,
  update::{
    diff_and_patch_module_graph::{diff_module_graph, patch_module_graph},
    patch_module_group_graph,
    regenerate_resources::generate_and_diff_resource_pots::generate_and_diff_resource_pots,
  },
};

#[test]
fn test_generate_and_diff_resource_pots() {
  let mut module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();
  update_module_graph.remove_module(&"G".into());
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
    &module_graph
      .entries
      .clone()
      .into_iter()
      .map(|(entry, _)| entry)
      .collect(),
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
    updated_modules.clone(),
    &diff_result,
    &removed_modules,
    &mut module_graph,
    &mut module_group_graph,
  );
  assert_eq!(
    affected_groups,
    HashSet::from(["A".into(), "B".into(), "F".into(), "D".into()])
  );

  let mut config = Config::default();
  config.partial_bundling.enforce_resources = vec![PartialBundlingEnforceResourceConfig {
    name: "test".into(),
    test: vec![ConfigRegex::new("F"), ConfigRegex::new("H")],
  }];
  let plugins: Vec<Arc<dyn Plugin + 'static>> = vec![Arc::new(
    farmfe_plugin_partial_bundling::FarmPluginPartialBundling::new(&config),
  )];
  let context = Arc::new(CompilationContext::new(config, plugins).unwrap());

  {
    let mut mg = context.module_graph.write();
    *mg = module_graph;
  }

  {
    let mut mgg = context.module_group_graph.write();
    *mgg = module_group_graph;
  }

  let resource_pot_map =
    generate_resource_pot_map(&context, &PluginHookContext::default()).unwrap();
  context.resource_pot_map.write().replace(resource_pot_map);

  let mut resource_pot_ids = generate_and_diff_resource_pots(
    &affected_groups,
    &diff_result,
    &updated_modules,
    &removed_modules,
    &context,
  )
  .unwrap();
  resource_pot_ids.sort();
  assert_debug_snapshot!(resource_pot_ids);

  let module_group_graph = context.module_group_graph.read();

  let module_group_a = module_group_graph.module_group(&"A".into()).unwrap();
  assert_sorted_iter_eq!(module_group_a.resource_pots());
  let module_group_b = module_group_graph.module_group(&"B".into()).unwrap();
  assert_sorted_iter_eq!(module_group_b.resource_pots());
  let module_group_d = module_group_graph.module_group(&"D".into()).unwrap();
  assert_sorted_iter_eq!(module_group_d.resource_pots());
  let module_group_f = module_group_graph.module_group(&"F".into()).unwrap();
  assert_sorted_iter_eq!(module_group_f.resource_pots());

  let module_graph = context.module_graph.read();
  // F, E, B, H
  let module_f = module_graph.module(&"F".into()).unwrap();
  assert_debug_snapshot!(module_f.resource_pot.as_ref().unwrap());
  let module_e = module_graph.module(&"E".into()).unwrap();
  assert_debug_snapshot!(module_e.resource_pot.as_ref().unwrap());
  let module_b = module_graph.module(&"B".into()).unwrap();
  assert_debug_snapshot!(module_b.resource_pot.as_ref().unwrap());
  let module_h = module_graph.module(&"H".into()).unwrap();
  assert_debug_snapshot!(module_h.resource_pot.as_ref().unwrap());

  let resource_pot_map = context.resource_pot_map.read();
  println!(
    "{:?}",
    resource_pot_map
      .resource_pots()
      .iter()
      .map(|rp| rp.id.clone())
      .collect::<Vec<_>>()
  );

  let mut resource_pots = resource_pot_map.resource_pots();

  resource_pots.sort_by(|a, b| a.id.cmp(&b.id));

  assert_resource_pots!(resource_pots);
}

/// the enforce resource pot should be unchanged when only one module is changed
#[test]
fn test_generate_and_diff_resource_pots_one_module_changed() {
  let mut module_graph = construct_test_module_graph_complex();
  let module_i = Module::new("I".into());
  module_graph.add_module(module_i);
  module_graph
    .add_edge(
      &"E".into(),
      &"I".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::DynamicImport,
        ..Default::default()
      }]),
    )
    .unwrap();
  let mut update_module_graph = construct_test_module_graph_complex();
  let module_i = Module::new("I".into());
  update_module_graph.add_module(module_i);
  update_module_graph
    .add_edge(
      &"E".into(),
      &"I".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::DynamicImport,
        ..Default::default()
      }]),
    )
    .unwrap();
  let mut module_group_graph = module_group_graph_from_entries(
    &module_graph
      .entries
      .clone()
      .into_iter()
      .map(|(entry, _)| entry)
      .collect(),
    &mut module_graph,
  );
  let updated_modules = vec!["I".into()];

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
  assert_eq!(affected_groups, HashSet::from(["I".into()]));

  let mut config = Config::default();
  config.partial_bundling.enforce_resources = vec![PartialBundlingEnforceResourceConfig {
    name: "test".into(),
    test: vec![ConfigRegex::new("F"), ConfigRegex::new("H")],
  }];
  let plugins: Vec<Arc<dyn Plugin + 'static>> = vec![Arc::new(
    farmfe_plugin_partial_bundling::FarmPluginPartialBundling::new(&config),
  )];
  let context = Arc::new(CompilationContext::new(config, plugins).unwrap());

  {
    let mut mg = context.module_graph.write();
    *mg = module_graph;
  }

  {
    let mut mgg = context.module_group_graph.write();
    *mgg = module_group_graph;
  }

  let resource_pot_map =
    generate_resource_pot_map(&context, &PluginHookContext::default()).unwrap();
  context.resource_pot_map.write().replace(resource_pot_map);

  let new_resource_pot_ids = generate_and_diff_resource_pots(
    &affected_groups,
    &diff_result,
    &updated_modules,
    &removed_modules,
    &context,
  )
  .unwrap();

  assert!(new_resource_pot_ids.is_empty());
}
