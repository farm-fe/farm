use std::sync::Arc;

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::{
    module_graph::{ModuleGraphEdge, ModuleGraphEdgeDataItem},
    module_group::{ModuleGroupId, ModuleGroupType},
    Module, ModuleType,
  },
  plugin::{Plugin, PluginHookContext},
  HashSet,
};
use farmfe_plugin_partial_bundling::module_group_graph_from_entries;
use farmfe_testing_helpers::construct_test_module_graph;

use crate::{
  generate::partial_bundling::generate_resource_pot_map,
  update::{
    diff_and_patch_module_graph::{diff_module_graph, patch_module_graph},
    patch_module_group_graph,
    regenerate_resources::generate_and_diff_resource_pots::handle_dynamic_entry_resource_pots,
  },
};

#[test]
fn test_handle_dynamic_entry_resource_pots() {
  let mut module_graph = construct_test_module_graph();
  let mut update_module_graph = construct_test_module_graph();
  update_module_graph.remove_module(&"C".into());
  update_module_graph
    .remove_edge(&"F".into(), &"A".into())
    .unwrap();
  update_module_graph.add_module({
    let mut m = Module::new("H".into());

    m.module_type = ModuleType::Js;

    m
  });
  update_module_graph
    .add_edge(
      &"B".into(),
      &"H".into(),
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        kind: farmfe_core::plugin::ResolveKind::DynamicEntry {
          name: "BH".to_string(),
          output_filename: None,
        },
        ..Default::default()
      }]),
    )
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
  let group_id_h = ModuleGroupId::new(&"H".into(), &ModuleGroupType::DynamicEntry);

  assert_eq!(
    affected_groups,
    HashSet::from_iter([
      group_id_a,
      group_id_b,
      group_id_f,
      group_id_d,
      group_id_h.clone()
    ])
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
    HashSet::from_iter([
      "A".into(),
      "B".into(),
      "C".into(),
      "D".into(),
      "E".into(),
      "F".into(),
      "H".into()
    ])
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

  let dynamic_entry_resource_pots = handle_dynamic_entry_resource_pots(&affected_groups, &context);

  assert_eq!(dynamic_entry_resource_pots.len(), 1);
  assert_eq!(
    dynamic_entry_resource_pots,
    HashSet::from_iter(["BH__dynamic_entry_js".to_string()])
  );
  let resource_pot_map = context.resource_pot_map.read();
  let dynamic_resource_pot = resource_pot_map
    .resource_pot(&"BH__dynamic_entry_js".into())
    .unwrap();
  assert_eq!(
    dynamic_resource_pot.modules,
    HashSet::from_iter(["F".into(), "H".into()])
  );

  let module_graph = context.module_graph.read();
  let module_h = module_graph.module(&"H".into()).unwrap();
  assert_eq!(module_h.module_groups, HashSet::from_iter([group_id_h]));
}
