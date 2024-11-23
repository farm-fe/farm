use farmfe_core::{
  config::partial_bundling::PartialBundlingGroupConfigResourceType,
  module::{
    module_graph::ModuleGraph,
    module_group::{ModuleGroupGraph, ModuleGroupId},
    ModuleId,
  },
  HashMap,
};

use crate::module_bucket::ModuleBucket;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
  Initial,
  Async,
}

impl ResourceType {
  pub fn is_match(&self, ty: PartialBundlingGroupConfigResourceType) -> bool {
    match ty {
      PartialBundlingGroupConfigResourceType::All => true,
      PartialBundlingGroupConfigResourceType::Initial => matches!(self, Self::Initial),
      PartialBundlingGroupConfigResourceType::Async => matches!(self, Self::Async),
    }
  }
}

#[derive(Debug)]
pub struct ModuleGroupBuckets {
  pub module_group_id: ModuleGroupId,
  pub resource_type: ResourceType,
  pub buckets: Vec<String>,
}

impl ModuleGroupBuckets {
  pub fn new(
    module_group_id: ModuleGroupId,
    resource_type: ResourceType,
    buckets: Vec<String>,
  ) -> Self {
    Self {
      module_group_id,
      resource_type,
      buckets,
    }
  }
}

/// Generate module buckets from modules.
pub fn generate_module_buckets_map(
  modules: &Vec<ModuleId>,
  module_graph: &ModuleGraph,
) -> HashMap<String, ModuleBucket> {
  let mut module_buckets_map = HashMap::<String, ModuleBucket>::default();

  for module_id in modules {
    let module = module_graph.module(module_id).unwrap();
    // ignore external module
    if module.external {
      continue;
    }

    let key = ModuleBucket::id(module);

    if let Some(module_bucket) = module_buckets_map.get_mut(&key) {
      module_bucket.add_module(module);
    } else {
      let module_bucket = ModuleBucket::new(key.clone(), module);
      module_buckets_map.insert(key, module_bucket);
    }
  }

  module_buckets_map
}

/// Group and sort the module buckets by module group.
pub fn group_module_buckets_by_module_group(
  module_buckets: &HashMap<String, ModuleBucket>,
  module_group_graph: &ModuleGroupGraph,
  module_graph: &ModuleGraph,
) -> Vec<ModuleGroupBuckets> {
  let mut module_group_buckets_map = HashMap::<ModuleGroupId, Vec<String>>::default();
  let mut entries = module_graph
    .entries
    .iter()
    .map(|m| m.0.clone())
    .collect::<Vec<_>>();
  entries.sort();
  // get the topo order of module groups
  let sorted_module_group_ids = module_group_graph.toposort(entries);

  for module_bucket in module_buckets.values() {
    for module_group_id in module_bucket.module_groups() {
      if let Some(module_group_buckets) = module_group_buckets_map.get_mut(module_group_id) {
        module_group_buckets.push(module_bucket.id.clone());
      } else {
        module_group_buckets_map.insert(module_group_id.clone(), vec![module_bucket.id.clone()]);
      }
    }
  }

  let mut module_group_buckets = vec![];

  // Make sure the order of module group buckets is topo sorted
  for module_group_id in sorted_module_group_ids {
    if module_group_buckets_map.contains_key(&module_group_id) {
      let buckets = module_group_buckets_map.remove(&module_group_id).unwrap();
      let resource_type = if module_graph.entries.contains_key(&module_group_id) {
        ResourceType::Initial
      } else {
        ResourceType::Async
      };
      module_group_buckets.push(ModuleGroupBuckets::new(
        module_group_id,
        resource_type,
        buckets,
      ));
    }
  }

  module_group_buckets
}

#[cfg(test)]
mod tests {
  use farmfe_testing_helpers::construct_test_module_graph_complex;
  use std::collections::HashSet;

  use crate::module_group_graph_from_entries;

  use super::*;

  fn construct_test_module_buckets_map(
    module_graph: &ModuleGraph,
  ) -> HashMap<String, ModuleBucket> {
    let mut modules = module_graph
      .modules()
      .iter()
      .map(|m| m.id.clone())
      .collect::<Vec<_>>();
    modules.sort();

    generate_module_buckets_map(&modules, module_graph)
  }

  #[test]
  fn test_generate_module_buckets() {
    let mut module_graph = construct_test_module_graph_complex();
    let entries = module_graph.entries.clone().into_keys().collect::<Vec<_>>();
    module_group_graph_from_entries(&entries, &mut module_graph);

    let module_buckets_map = construct_test_module_buckets_map(&module_graph);

    // Sort the keys to make sure the order is stable.
    let mut module_buckets = module_buckets_map.into_values().collect::<Vec<_>>();
    module_buckets.sort_by_key(|b| b.id.clone());

    assert_eq!(module_buckets.len(), 6);
    // println!("{:?} \n {:?}", modules, module_buckets);

    assert_eq!(module_buckets[0].modules().len(), 2);
    assert_eq!(
      module_buckets[0].modules(),
      &HashSet::from_iter(["A".into(), "C".into()])
    );
    assert_eq!(module_buckets[0].module_groups().len(), 2);
    assert_eq!(
      module_buckets[0].module_groups(),
      &HashSet::from_iter(["A".into(), "F".into()])
    );

    assert_eq!(module_buckets[1].modules().len(), 2);
    assert_eq!(
      module_buckets[1].modules(),
      &HashSet::from_iter(["B".into(), "E".into()])
    );
    assert_eq!(module_buckets[1].module_groups().len(), 1);
    assert_eq!(
      module_buckets[1].module_groups(),
      &HashSet::from_iter(["B".into()])
    );

    assert_eq!(module_buckets[2].modules().len(), 1);
    assert_eq!(
      module_buckets[2].modules(),
      &HashSet::from_iter(["D".into()])
    );
    assert_eq!(module_buckets[2].module_groups().len(), 2);
    assert_eq!(
      module_buckets[2].module_groups(),
      &HashSet::from_iter(["B".into(), "D".into()])
    );

    assert_eq!(module_buckets[3].modules().len(), 1);
    assert_eq!(
      module_buckets[3].modules(),
      &HashSet::from_iter(["H".into()])
    );
    assert_eq!(module_buckets[3].module_groups().len(), 4);
    assert_eq!(
      module_buckets[3].module_groups(),
      &HashSet::from_iter(["G".into(), "F".into(), "B".into(), "D".into()])
    );

    assert_eq!(module_buckets[4].modules().len(), 1);
    assert_eq!(
      module_buckets[4].modules(),
      &HashSet::from_iter(["F".into()])
    );
    assert_eq!(module_buckets[4].module_groups().len(), 1);
    assert_eq!(
      module_buckets[4].module_groups(),
      &HashSet::from_iter(["F".into()])
    );

    assert_eq!(module_buckets[5].modules().len(), 1);
    assert_eq!(
      module_buckets[5].modules(),
      &HashSet::from_iter(["G".into()])
    );
    assert_eq!(module_buckets[5].module_groups().len(), 1);
    assert_eq!(
      module_buckets[5].module_groups(),
      &HashSet::from_iter(["G".into()])
    );
  }

  #[test]
  pub fn test_group_module_buckets_for_module_group() {
    let mut module_graph = construct_test_module_graph_complex();
    let entries = module_graph.entries.clone().into_keys().collect::<Vec<_>>();
    let module_group_graph = module_group_graph_from_entries(&entries, &mut module_graph);

    let module_buckets_map = construct_test_module_buckets_map(&module_graph);

    let module_group_buckets =
      group_module_buckets_by_module_group(&module_buckets_map, &module_group_graph, &module_graph);

    assert_eq!(module_group_buckets.len(), 5);

    assert_eq!(module_group_buckets[0].module_group_id, "B".into());
    assert_eq!(module_group_buckets[0].resource_type, ResourceType::Initial);
    assert_eq!(module_group_buckets[0].buckets.len(), 3);
    assert_eq!(
      HashSet::<String>::from_iter(module_group_buckets[0].buckets.clone().into_iter()),
      HashSet::from([
        "js_false_B".to_string(),
        "js_false_B_D".to_string(),
        "js_false_B_D_F_G".to_string()
      ])
    );

    assert_eq!(module_group_buckets[1].module_group_id, "G".into());
    assert_eq!(module_group_buckets[1].resource_type, ResourceType::Async);
    assert_eq!(module_group_buckets[1].buckets.len(), 2);
    assert_eq!(
      HashSet::<String>::from_iter(module_group_buckets[1].buckets.clone().into_iter()),
      HashSet::from(["js_false_G".to_string(), "js_false_B_D_F_G".to_string()])
    );

    assert_eq!(module_group_buckets[2].module_group_id, "A".into());
    assert_eq!(module_group_buckets[2].resource_type, ResourceType::Initial);
    assert_eq!(module_group_buckets[2].buckets.len(), 1);
    assert_eq!(
      HashSet::<String>::from_iter(module_group_buckets[2].buckets.clone().into_iter()),
      HashSet::from(["js_false_A_F".to_string()])
    );

    assert_eq!(module_group_buckets[3].module_group_id, "F".into());
    assert_eq!(module_group_buckets[3].resource_type, ResourceType::Async);
    assert_eq!(module_group_buckets[3].buckets.len(), 3);
    assert_eq!(
      HashSet::<String>::from_iter(module_group_buckets[3].buckets.clone().into_iter()),
      HashSet::from([
        "js_false_F".to_string(),
        "js_false_A_F".to_string(),
        "js_false_B_D_F_G".to_string()
      ])
    );

    assert_eq!(module_group_buckets[4].module_group_id, "D".into());
    assert_eq!(module_group_buckets[4].resource_type, ResourceType::Async);
    assert_eq!(module_group_buckets[4].buckets.len(), 2);
    assert_eq!(
      HashSet::<String>::from_iter(module_group_buckets[4].buckets.clone().into_iter()),
      HashSet::from(["js_false_B_D_F_G".to_string(), "js_false_B_D".to_string()])
    );
  }
}
