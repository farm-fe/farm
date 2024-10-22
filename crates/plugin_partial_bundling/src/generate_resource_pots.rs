use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use farmfe_core::config::Config;
use farmfe_core::{
  module::{module_graph::ModuleGraph, module_group::ModuleGroupId},
  resource::resource_pot::ResourcePot,
};

use crate::{
  generate_module_buckets::ModuleGroupBuckets,
  generate_module_pots::generate_module_pots,
  merge_module_pots::{merge_module_pots, ModuleGroupModulePots},
  module_bucket::ModuleBucket,
  module_pot::ModulePot,
  utils::try_get_filename,
};

/// Generate resource pots from module group buckets.
/// 1. create module pots from module buckets.
/// 2. merge module pots to resource pots.
pub fn generate_resource_pots(
  module_group_buckets: Vec<ModuleGroupBuckets>,
  mut module_buckets_map: HashMap<String, ModuleBucket>,
  module_graph: &ModuleGraph,
  config: &Config,
  groups_enforce_map: &HashMap<String, bool>,
) -> Vec<ResourcePot> {
  let mut resource_pots = vec![];
  let mut handled_module_group_buckets = HashSet::new();
  let mut used_resource_pot_names = HashSet::new();

  for mut module_group_bucket in module_group_buckets {
    let module_group_id = module_group_bucket.module_group_id;
    let base_resource_pot_name = generate_resource_pot_name(
      module_group_id.clone(),
      &used_resource_pot_names,
      &module_graph,
    );
    used_resource_pot_names.insert(base_resource_pot_name.clone());

    let mut module_group_module_pots = ModuleGroupModulePots::new(module_group_id.clone());

    // Sort the buckets to make sure it is stable.
    module_group_bucket.buckets.sort();

    for module_bucket_id in module_group_bucket.buckets {
      if handled_module_group_buckets.contains(&module_bucket_id) {
        continue;
      }

      let module_bucket = module_buckets_map.get_mut(&module_bucket_id).unwrap();

      let module_pots: Vec<ModulePot> = generate_module_pots(
        &module_bucket.modules(),
        module_graph,
        config,
        module_group_bucket.resource_type.clone(),
        groups_enforce_map,
      );

      module_group_module_pots.add_module_pots(module_bucket_id.clone(), module_pots);
      handled_module_group_buckets.insert(module_bucket_id);
    }

    // sort the module pots by size to make sure bigger module pots are merged first.
    module_group_module_pots
      .module_pots
      .values_mut()
      .for_each(|pots| {
        pots.sort_by(|a, b| b.size.cmp(&a.size));
      });

    let merged_resource_pots = merge_module_pots(
      module_group_module_pots,
      config,
      &base_resource_pot_name,
      module_graph,
    );

    resource_pots.extend(merged_resource_pots);
  }

  resource_pots
}

/// Generate resource pot id from module group id.
/// 1. If module_group_id is entry module group, then the resource pot id is the name defined in config.
/// 2. If module_group_id is not entry module group, then the resource pot id is the module group id's filename(without extension).
///    If the filename is used by other resource pot, try use its parent dir util we find a unique name.
fn generate_resource_pot_name(
  module_group_id: ModuleGroupId,
  used_resource_pot_names: &HashSet<String>,
  module_graph: &ModuleGraph,
) -> String {
  if let Some(name) = module_graph.entries.get(&module_group_id) {
    return name.clone();
  }

  let mut path = PathBuf::from(module_group_id.to_string());
  let mut name = try_get_filename(path.clone());

  if !used_resource_pot_names.contains(&name) {
    return name;
  }

  while path.parent().is_some() {
    path = path.parent().unwrap().to_path_buf();
    // If the path is root, then break.
    if path.parent().is_none() {
      break;
    }

    name = format!("{}_{}", try_get_filename(path.clone()), name);

    if !used_resource_pot_names.contains(&name) {
      return name;
    }
  }

  return name;
}

#[cfg(test)]
mod tests {
  use farmfe_core::module::{module_graph::ModuleGraph, module_group::ModuleGroupId, Module};
  use std::collections::HashSet;

  use crate::generate_resource_pots::generate_resource_pot_name;

  #[test]
  fn test_generate_resource_pot_name() {
    let mut module_graph = ModuleGraph::new();
    let module_a = Module::new("test/src/a.html".into());
    module_graph
      .entries
      .insert(module_a.id.clone(), "a".to_string());
    module_graph.add_module(module_a);

    let mut used_resource_pot_names = HashSet::new();
    assert_eq!(
      generate_resource_pot_name(
        "test/src/a.html".into(),
        &used_resource_pot_names,
        &module_graph
      ),
      "a"
    );

    let group_id: ModuleGroupId = "test/src/api.ts".into();
    assert_eq!(
      generate_resource_pot_name(group_id.clone(), &used_resource_pot_names, &module_graph),
      "api"
    );

    used_resource_pot_names.insert("api".to_string());
    assert_eq!(
      generate_resource_pot_name(group_id.clone(), &used_resource_pot_names, &module_graph),
      "src_api"
    );

    used_resource_pot_names.insert("src_api".to_string());
    used_resource_pot_names.insert("test_src_api".to_string());
    assert_eq!(
      generate_resource_pot_name(group_id, &used_resource_pot_names, &module_graph),
      "test_src_api"
    );
  }
}
