use std::sync::Arc;

use farmfe_core::{
  HashSet,
  config::{
    Config, config_regex::ConfigRegex, partial_bundling::PartialBundlingEnforceResourceConfig,
  },
  context::CompilationContext,
  module::{
    Module, ModuleId, ModuleType,
    module_graph::{ModuleGraphEdge, ModuleGraphEdgeDataItem},
    module_group::{ModuleGroupId, ModuleGroupType},
  },
  plugin::{Plugin, PluginHookContext, ResolveKind},
  resource::resource_pot::ResourcePotId,
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
    regenerate_resources::{
      clear_resource_pot_of_modules_in_module_groups,
      generate_and_diff_resource_pots::generate_and_diff_resource_pots,
    },
  },
};

#[test]
fn newly_discovered_worker_keeps_its_resource_pot_with_shared_business_dependencies() {
  let make_graph = || {
    let mut graph = farmfe_core::module::module_graph::ModuleGraph::new();
    for id in ["entry", "page", "shared"] {
      let mut module = Module::new(id.into());
      module.module_type = ModuleType::Js;
      graph.add_module(module);
    }
    graph.entries.insert("entry".into(), "entry".into());
    graph
      .add_edge(&"entry".into(), &"page".into(), Default::default())
      .unwrap();
    graph
      .add_edge(&"entry".into(), &"shared".into(), Default::default())
      .unwrap();
    graph.update_execution_order_for_modules();
    graph
  };
  let mut graph = make_graph();
  let groups = module_group_graph_from_entries(&["entry".into()], &mut graph);
  let config = Config::default();
  let plugins: Vec<Arc<dyn Plugin>> = vec![Arc::new(
    farmfe_plugin_partial_bundling::FarmPluginPartialBundling::new(&config),
  )];
  let context = Arc::new(CompilationContext::new(config, plugins).unwrap());
  *context.module_graph.write() = graph;
  *context.module_group_graph.write() = groups;
  let initial = generate_resource_pot_map(&context, &PluginHookContext::default()).unwrap();
  context.resource_pot_map.write().replace(initial);

  let mut update = make_graph();
  for id in ["factory", "worker"] {
    let mut module = Module::new(id.into());
    module.module_type = ModuleType::Js;
    update.add_module(module);
  }
  update
    .add_edge(&"page".into(), &"factory".into(), Default::default())
    .unwrap();
  update
    .add_edge(
      &"factory".into(),
      &"worker".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::DynamicEntry {
          name: "import".into(),
          output_filename: None,
        },
        ..Default::default()
      }]),
    )
    .unwrap();
  update
    .add_edge(&"worker".into(), &"shared".into(), Default::default())
    .unwrap();
  update.update_execution_order_for_modules();
  let updated = vec!["page".into()];
  let (diff, removed, affected) = {
    let mut graph = context.module_graph.write();
    let mut groups = context.module_group_graph.write();
    let diff = diff_module_graph(updated.clone(), &graph, &update);
    let removed = patch_module_graph(updated.clone(), &diff, &mut graph, &mut update);
    let affected =
      patch_module_group_graph(updated.clone(), &diff, &removed, &mut graph, &mut groups);
    (diff, removed, affected)
  };
  clear_resource_pot_of_modules_in_module_groups(&affected, &context);
  let generated =
    generate_and_diff_resource_pots(&affected, &diff, &updated, &removed, &context).unwrap();
  let pots = context.resource_pot_map.read();
  for id in generated {
    assert!(
      pots.has_resource_pot(&id),
      "regenerated resource pot disappeared: {id}"
    );
  }
  let worker_group_id = ModuleGroupId::new(&"worker".into(), &ModuleGroupType::DynamicEntry);
  let groups = context.module_group_graph.read();
  let worker_group = groups.module_group(&worker_group_id).unwrap();
  assert!(!worker_group.resource_pots().is_empty());
  for id in worker_group.resource_pots() {
    let pot = pots
      .resource_pot(id)
      .expect("Worker group must refer to a live resource pot");
    assert!(pot.is_dynamic_entry);
    assert_eq!(pot.entry_module, Some("worker".into()));
    assert!(
      pot
        .modules()
        .into_iter()
        .any(|id| id == &ModuleId::from("shared"))
    );
  }
}

#[test]
fn test_generate_and_diff_resource_pots() {
  let mut module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();
  update_module_graph.remove_module(&"G".into());
  update_module_graph
    .remove_edge(&"F".into(), &"A".into())
    .unwrap();
  let mut m_h = Module::new("H".into());
  m_h.module_type = ModuleType::Js;
  update_module_graph.add_module(m_h);
  update_module_graph
    .add_edge(&"B".into(), &"H".into(), Default::default())
    .unwrap();
  update_module_graph
    .add_edge(&"H".into(), &"F".into(), Default::default())
    .unwrap();

  update_module_graph.add_module(Module::new("I".into()));
  update_module_graph
    .add_edge(
      &"E".into(),
      &"I".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: ResolveKind::DynamicEntry {
          name: "EI".to_string(),
          output_filename: None,
        },
        ..Default::default()
      }]),
    )
    .unwrap();
  update_module_graph
    .update_edge(&"I".into(), &"F".into(), Default::default())
    .unwrap();

  let updated_modules = vec!["F".into(), "E".into(), "B".into()];
  let mut module_group_graph = module_group_graph_from_entries(
    &module_graph.entries.clone().into_keys().collect::<Vec<_>>(),
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

  let group_id_a = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);
  let group_id_d = ModuleGroupId::new(&"D".into(), &ModuleGroupType::DynamicImport);
  let group_id_f = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);
  let group_id_i = ModuleGroupId::new(&"I".into(), &ModuleGroupType::DynamicEntry);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([
      group_id_a.clone(),
      group_id_b.clone(),
      group_id_f.clone(),
      group_id_d.clone(),
      group_id_i.clone()
    ])
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

  clear_resource_pot_of_modules_in_module_groups(&affected_groups, &context);
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

  #[derive(Clone, Debug, PartialEq, Eq)]
  struct GroupResourcePotsSnapshotItem {
    pub id: ModuleGroupId,
    pub resource_pots: HashSet<ResourcePotId>,
  }

  impl PartialOrd for GroupResourcePotsSnapshotItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
      Some(self.cmp(other))
    }
  }
  impl Ord for GroupResourcePotsSnapshotItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
      self.id.cmp(&other.id)
    }
  }

  let module_group_graph = context.module_group_graph.read();
  let mut group_resource_pots = vec![];
  let module_group_a = module_group_graph.module_group(&group_id_a).unwrap();
  group_resource_pots.push(GroupResourcePotsSnapshotItem {
    id: module_group_a.id.clone(),
    resource_pots: module_group_a.resource_pots().clone(),
  });
  let module_group_b = module_group_graph.module_group(&group_id_b).unwrap();
  group_resource_pots.push(GroupResourcePotsSnapshotItem {
    id: module_group_b.id.clone(),
    resource_pots: module_group_b.resource_pots().clone(),
  });
  let module_group_d = module_group_graph.module_group(&group_id_d).unwrap();
  group_resource_pots.push(GroupResourcePotsSnapshotItem {
    id: module_group_d.id.clone(),
    resource_pots: module_group_d.resource_pots().clone(),
  });
  let module_group_f = module_group_graph.module_group(&group_id_f).unwrap();
  group_resource_pots.push(GroupResourcePotsSnapshotItem {
    id: module_group_f.id.clone(),
    resource_pots: module_group_f.resource_pots().clone(),
  });
  let module_group_i = module_group_graph.module_group(&group_id_i).unwrap();
  group_resource_pots.push(GroupResourcePotsSnapshotItem {
    id: module_group_i.id.clone(),
    resource_pots: module_group_i.resource_pots().clone(),
  });
  assert_sorted_iter_eq!(group_resource_pots);

  #[derive(Clone, Debug, PartialEq, Eq)]
  struct ModuleResourcePotSnapshotItem {
    pub id: ModuleId,
    pub resource_pots: HashSet<ResourcePotId>,
  }
  impl PartialOrd for ModuleResourcePotSnapshotItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
      Some(self.cmp(other))
    }
  }
  impl Ord for ModuleResourcePotSnapshotItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
      self.id.cmp(&other.id)
    }
  }
  let module_group_a = module_group_graph.module_group(&group_id_a).unwrap();
  assert_eq!(
    module_group_a.resource_pots(),
    &HashSet::from_iter(["A_66be131002b8b5af1132bafd62635f07_js".to_string()])
  );
  let module_group_b = module_group_graph.module_group(&group_id_b).unwrap();
  assert_eq!(
    module_group_b.resource_pots(),
    &HashSet::from_iter([
      "B_2f5d6a63eb2504d486bf13df147c043a_js".to_string(),
      "B_3f39d5c348e5b79d06e842c114e6cc57_js".to_string(),
      "test__js".to_string()
    ])
  );
  let module_group_d = module_group_graph.module_group(&group_id_d).unwrap();
  assert_eq!(
    module_group_d.resource_pots(),
    &HashSet::from_iter(["B_3f39d5c348e5b79d06e842c114e6cc57_js".to_string()])
  );
  let module_group_f = module_group_graph.module_group(&group_id_f).unwrap();
  assert_eq!(
    module_group_f.resource_pots(),
    &HashSet::from_iter(["test__js".to_string()])
  );

  let module_graph = context.module_graph.read();
  let mut module_resource_pots = vec![];
  // F, E, B, H
  let module_f = module_graph.module(&"F".into()).unwrap();
  module_resource_pots.push(ModuleResourcePotSnapshotItem {
    id: module_f.id.clone(),
    resource_pots: module_f.resource_pots.clone(),
  });
  let module_e = module_graph.module(&"E".into()).unwrap();
  module_resource_pots.push(ModuleResourcePotSnapshotItem {
    id: module_e.id.clone(),
    resource_pots: module_e.resource_pots.clone(),
  });
  let module_b = module_graph.module(&"B".into()).unwrap();
  module_resource_pots.push(ModuleResourcePotSnapshotItem {
    id: module_b.id.clone(),
    resource_pots: module_b.resource_pots.clone(),
  });
  let module_h = module_graph.module(&"H".into()).unwrap();
  module_resource_pots.push(ModuleResourcePotSnapshotItem {
    id: module_h.id.clone(),
    resource_pots: module_h.resource_pots.clone(),
  });
  let module_i = module_graph.module(&"I".into()).unwrap();
  module_resource_pots.push(ModuleResourcePotSnapshotItem {
    id: module_i.id.clone(),
    resource_pots: module_i.resource_pots.clone(),
  });
  assert_sorted_iter_eq!(module_resource_pots);

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
  let resource_pot_test = resource_pot_map.resource_pot(&"test__js".into()).unwrap();
  assert_eq!(resource_pot_test.entry_module, None);
  assert_eq!(resource_pot_test.modules(), vec![&"F".into(), &"H".into()]);
  assert_eq!(
    resource_pot_test.module_groups,
    HashSet::from_iter([group_id_f.clone(), group_id_b.clone()])
  );

  let resource_pot_b_2f5d = resource_pot_map
    .resource_pot(&"B_2f5d6a63eb2504d486bf13df147c043a_js".into())
    .unwrap();
  assert_eq!(resource_pot_b_2f5d.entry_module, Some("B".into()));
  assert_eq!(
    resource_pot_b_2f5d.modules(),
    vec![&"B".into(), &"E".into()]
  );
  assert_eq!(
    resource_pot_b_2f5d.module_groups,
    HashSet::from_iter([group_id_b.clone()])
  );

  let resource_pot_b_3f39 = resource_pot_map
    .resource_pot(&"B_3f39d5c348e5b79d06e842c114e6cc57_js".into())
    .unwrap();
  assert_eq!(resource_pot_b_3f39.entry_module, None);
  assert_eq!(resource_pot_b_3f39.modules(), vec![&"D".into()]);
  assert_eq!(
    resource_pot_b_3f39.module_groups,
    HashSet::from_iter([group_id_d.clone(), group_id_b.clone()])
  );

  let resource_pot_a_66be = resource_pot_map
    .resource_pot(&"A_66be131002b8b5af1132bafd62635f07_js".into())
    .unwrap();
  assert_eq!(resource_pot_a_66be.entry_module, Some("A".into()));
  assert_eq!(
    resource_pot_a_66be.modules(),
    vec![&"A".into(), &"C".into()]
  );
  assert_eq!(
    resource_pot_a_66be.module_groups,
    HashSet::from_iter([group_id_a.clone()])
  );
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
    &module_graph.entries.clone().into_keys().collect::<Vec<_>>(),
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

  let group_id_i = ModuleGroupId::new(&"I".into(), &ModuleGroupType::DynamicImport);

  assert_eq!(affected_groups, HashSet::from_iter([group_id_i]));

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

  clear_resource_pot_of_modules_in_module_groups(&affected_groups, &context);
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

// test remove module D from module graph
#[test]
fn test_generate_and_diff_resource_pots_remove_module() {
  let mut module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();
  update_module_graph.remove_module(&"D".into());
  update_module_graph.remove_module(&"F".into());
  update_module_graph.remove_module(&"G".into());

  let module_group_graph = module_group_graph_from_entries(
    &module_graph.entries.clone().into_keys().collect::<Vec<_>>(),
    &mut module_graph,
  );

  let config = Config::default();
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

  let mut module_graph = context.module_graph.write();
  let mut module_group_graph = context.module_group_graph.write();

  let updated_modules = vec!["A".into(), "B".into()];

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

  drop(module_graph);
  drop(module_group_graph);

  let group_id_a = ModuleGroupId::new(&"A".into(), &ModuleGroupType::Entry);
  let group_id_b = ModuleGroupId::new(&"B".into(), &ModuleGroupType::Entry);
  let group_id_f = ModuleGroupId::new(&"F".into(), &ModuleGroupType::DynamicImport);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([group_id_a, group_id_b, group_id_f])
  );

  clear_resource_pot_of_modules_in_module_groups(&affected_groups, &context);
  let new_resource_pot_ids = generate_and_diff_resource_pots(
    &affected_groups,
    &diff_result,
    &updated_modules,
    &removed_modules,
    &context,
  )
  .unwrap();

  assert!(new_resource_pot_ids.is_empty());

  let module_graph = context.module_graph.read();
  let module_group_graph = context.module_group_graph.read();

  let module_d = module_graph.module(&"D".into());
  assert!(module_d.is_none());
  assert!(!module_group_graph.has(&ModuleGroupId::new(
    &"D".into(),
    &ModuleGroupType::DynamicImport,
  )));
}

#[test]
fn shared_groups_keep_all_new_and_enforced_pots_when_pruned_once() {
  use farmfe_core::{
    module::module_group::ModuleGroup,
    resource::resource_pot::{ResourcePot, ResourcePotType},
  };
  let context = Arc::new(CompilationContext::new(Config::default(), vec![]).unwrap());
  let obsolete = ResourcePot::new("obsolete", "", ResourcePotType::Js);
  let enforced = ResourcePot::new("enforced", "", ResourcePotType::Js);
  let mut first = ModuleGroup::new("first".into(), ModuleGroupType::Entry);
  let mut second = ModuleGroup::new("second".into(), ModuleGroupType::Entry);
  for group in [&mut first, &mut second] {
    group.add_resource_pot(obsolete.id.clone());
    group.add_resource_pot(enforced.id.clone());
  }
  let first_id = first.id.clone();
  let second_id = second.id.clone();
  context.module_group_graph.write().add_module_group(first);
  context.module_group_graph.write().add_module_group(second);
  let obsolete_id = obsolete.id.clone();
  let enforced_id = enforced.id.clone();
  context.resource_pot_map.write().add_resource_pot(obsolete);
  context.resource_pot_map.write().add_resource_pot(enforced);
  let mut a = ResourcePot::new("new-a", "", ResourcePotType::Js);
  let mut b = ResourcePot::new("new-b", "", ResourcePotType::Js);
  let mut c = ResourcePot::new("new-c", "", ResourcePotType::Js);
  a.module_groups.insert(first_id.clone());
  b.module_groups.insert(first_id.clone());
  b.module_groups.insert(second_id.clone());
  c.module_groups.insert(second_id.clone());
  let expected_first = HashSet::from_iter([a.id.clone(), b.id.clone(), enforced_id.clone()]);
  let expected_second = HashSet::from_iter([b.id.clone(), c.id.clone(), enforced_id.clone()]);
  let expected_new = HashSet::from_iter([a.id.clone(), b.id.clone(), c.id.clone()]);
  let changed =
    super::diff_and_patch_resource_pot_map(vec![a, b, c], &vec![enforced_id.clone()], &context);
  assert_eq!(changed, expected_new);
  let groups = context.module_group_graph.read();
  assert_eq!(
    groups.module_group(&first_id).unwrap().resource_pots(),
    &expected_first
  );
  assert_eq!(
    groups.module_group(&second_id).unwrap().resource_pots(),
    &expected_second
  );
  let pots = context.resource_pot_map.read();
  assert!(!pots.has_resource_pot(&obsolete_id));
  assert!(pots.has_resource_pot(&enforced_id));
  for id in changed {
    assert!(pots.has_resource_pot(&id));
  }
}
