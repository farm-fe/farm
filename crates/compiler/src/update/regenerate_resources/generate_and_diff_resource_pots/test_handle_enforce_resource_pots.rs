use std::collections::HashSet;
use std::sync::Arc;

use farmfe_core::{
  config::{
    config_regex::ConfigRegex, partial_bundling::PartialBundlingEnforceResourceConfig, Config,
  }, context::CompilationContext, farm_profile_function, module::{
    module_graph::{ModuleGraphEdge, ModuleGraphEdgeDataItem},
    Module,
  }, plugin::{Plugin, PluginHookContext, ResolveKind}
};
use farmfe_plugin_partial_bundling::module_group_graph_from_entries;
use farmfe_testing_helpers::{construct_test_module_graph, construct_test_module_graph_complex};

use crate::{
  generate::partial_bundling::generate_resource_pot_map,
  update::{
    diff_and_patch_module_graph::{diff_module_graph, patch_module_graph},
    patch_module_group_graph,
  },
};

use super::handle_enforce_resource_pots;

#[test]
fn test_handle_enforce_resource_pots() {
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

  let affected_modules = affected_groups
    .iter()
    .fold(vec![], |mut acc, group_id| {
      let group = module_group_graph.module_group(group_id).unwrap();
      acc.extend(group.modules().clone());
      acc
    })
    .into_iter()
    .collect::<Vec<_>>();
  assert_eq!(
    affected_modules.clone().into_iter().collect::<HashSet<_>>(),
    HashSet::from([
      "A".into(),
      "B".into(),
      "C".into(),
      "D".into(),
      "E".into(),
      "F".into(),
      "H".into()
    ])
  );

  let mut config = Config::default();
  config.partial_bundling.enforce_resources = vec![PartialBundlingEnforceResourceConfig {
    name: "test".into(),
    test: vec![
      ConfigRegex::new("F"),
      ConfigRegex::new("H"),
      ConfigRegex::new("G"),
    ],
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

  let (enforce_resource_pots, mut un_enforce_resource_pots) = handle_enforce_resource_pots(
    &affected_modules,
    &diff_result,
    &updated_modules,
    &removed_modules,
    &context,
  );

  assert_eq!(
    enforce_resource_pots,
    vec!["test_custom(\"__farm_unknown\")".to_string()]
  );
  un_enforce_resource_pots.sort();
  assert_eq!(
    un_enforce_resource_pots,
    vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()]
  );
}

/// the enforce resource pot should be unchanged when only one module is changed
#[test]
fn test_handle_enforce_resource_pots_one_module_changed() {
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

  let affected_modules = affected_groups
    .iter()
    .fold(vec![], |mut acc, group_id| {
      let group = module_group_graph.module_group(group_id).unwrap();
      acc.extend(group.modules().clone());
      acc
    })
    .into_iter()
    .collect::<Vec<_>>();

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

  let (enforce_resource_pots, un_enforce_resource_pots) = handle_enforce_resource_pots(
    &affected_modules,
    &diff_result,
    &updated_modules,
    &removed_modules,
    &context,
  );

  assert!(enforce_resource_pots.is_empty());
  assert_eq!(un_enforce_resource_pots, vec!["I".into()]);
}
