#![feature(is_some_and)]

use std::{collections::VecDeque, fs, path::PathBuf, sync::Arc};

use farmfe_core::{
  config::{config_regex::ConfigRegex, Config, PartialBundlingModuleBucketsConfig},
  context::CompilationContext,
  farm_profile_function, farm_profile_scope,
  hashbrown::{HashMap, HashSet},
  module::{
    module_graph::ModuleGraph,
    module_group::{ModuleGroup, ModuleGroupGraph},
    ModuleId, ModuleType,
  },
  plugin::{Plugin, PluginHookContext},
  resource::resource_pot::ResourcePot,
};
use module_bucket::{ModuleBucket, ModuleBucketId};

mod module_bucket;
mod resource_group;
mod split_resource_unit;

use resource_group::{ids_to_string, is_subset, ResourceGroup, ResourceUnit, ResourceUnitId};

fn try_get_filename(module_id: &ModuleId) -> String {
  PathBuf::from(module_id.to_string())
    .file_stem()
    .map(|name| name.to_string_lossy().to_string())
    .unwrap_or(module_id.to_string())
}

pub struct FarmPluginPartialBundling {}

impl Plugin for FarmPluginPartialBundling {
  fn name(&self) -> &str {
    "FarmPluginPartialBundling"
  }

  fn priority(&self) -> i32 {
    99
  }

  fn config(&self, config: &mut Config) -> farmfe_core::error::Result<Option<()>> {
    let module_buckets = &mut config.partial_bundling.module_buckets;

    module_buckets.push(PartialBundlingModuleBucketsConfig {
      name: "i-vendor".into(),
      test: config.partial_bundling.immutable_modules.clone(),
      weight: 1000000000,
      ..Default::default()
    });

    if module_buckets.iter().any(|bucket| bucket.name != "vendor") {
      module_buckets.push(PartialBundlingModuleBucketsConfig {
        name: "vendor".into(),
        min_size: Some(1024 * 20),
        test: vec![ConfigRegex::new("[\\/]node_modules[\\/]")],
        max_concurrent_requests: Some(500),
        weight: -20,
        ..Default::default()
      });
    }

    if module_buckets.iter().any(|bucket| bucket.name != "common") {
      module_buckets.push(PartialBundlingModuleBucketsConfig {
        name: "common".into(),
        min_size: Some(1024 * 20),
        max_concurrent_requests: Some(5),
        weight: -20,
        ..Default::default()
      });
    }

    Ok(None)
  }

  fn analyze_module_graph(
    &self,
    module_graph: &mut ModuleGraph,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ModuleGroupGraph>> {
    let module_group_graph = module_group_graph_from_entries(
      &module_graph
        .entries
        .clone()
        .into_iter()
        .map(|(entry, _)| entry)
        .collect(),
      module_graph,
    );

    Ok(Some(module_group_graph))
  }

  /// The partial bundling algorithm's result should not be related to the order of the module group.
  /// Whatever the order of the module group is, the result should be the same.
  fn partial_bundling(
    &self,
    modules: &Vec<ModuleId>,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<Vec<ResourcePot>>> {
    farm_profile_function!("partial_bundling");
    let mut module_graph = context.module_graph.write();
    let mut module_bucket_map = HashMap::<ModuleBucketId, ModuleBucket>::new();

    let entries_modules: Vec<ModuleId> = module_graph.entries.keys().cloned().collect();

    let add_module_to_bucket = |module_bucket_id: ModuleBucketId,
                                module_bucket_map: &mut HashMap<ModuleBucketId, ModuleBucket>,
                                bucket_config: PartialBundlingModuleBucketsConfig,
                                resource_units: HashSet<ResourceUnitId>,
                                module_id: &ModuleId,
                                module_type: &ModuleType,
                                size: usize| {
      let module_bucket = module_bucket_map
        .entry(module_bucket_id.clone())
        .or_insert_with(|| {
          ModuleBucket::new(module_bucket_id.clone(), HashSet::new(), bucket_config)
        });

      module_bucket.add_module(module_id.clone(), module_type, size);

      for resource_pot_id in resource_units {
        module_bucket.add_resource_pot(resource_pot_id);
      }
    };

    let mut resource_group: ResourceGroup = ResourceGroup {
      resource_pot_group_map: HashMap::new(),
    };

    type ModduleGroupRedirectResourceMap = HashMap<ModuleId, ResourceUnitId>;
    let mut module_group_redirect_resource_map: ModduleGroupRedirectResourceMap = HashMap::new();

    let mut resource_unit_sets: HashMap<String, Vec<ResourceUnitId>> = Default::default();

    farm_profile_scope!("partial_bundling.gen_resource_unit");
    // 1. gen resource pot groups set (len > 1)
    // 2. gen module_group resource pot, if not exists
    for module_id in modules {
      if !module_graph.has_module(module_id) {
        continue;
      }

      let module = module_graph.module(module_id).unwrap();

      if module.external {
        continue;
      }

      module.module_groups.iter().for_each(|module_group| {
        let name = try_get_filename(module_group);
        if let Some(resource_unit_id) = module_group_redirect_resource_map.get(module_group) {
          let resource_unit = resource_group.resource_pot_mut(resource_unit_id).unwrap();
          resource_unit.add_module(module_id.clone());
        } else {
          let mut resource_unit = ResourceUnit::new(name);

          if entries_modules.contains(module_group) {
            resource_unit.entry_module = Some(module_group.clone());
          }

          resource_unit.add_module(module_id.clone());

          module_group_redirect_resource_map.insert(module_group.clone(), resource_unit.id.clone());

          resource_group.add_resource_pot(resource_unit);
        }
      });

      if module.module_groups.len() <= 1 {
        continue;
      }

      let mut module_relation_resource_pots: Vec<ResourceUnitId> = module
        .module_groups
        .iter()
        .map(|module_id| {
          module_group_redirect_resource_map
            .get(module_id)
            .unwrap()
            .clone()
        })
        .collect();

      module_relation_resource_pots.sort();

      let key = ids_to_string(module_relation_resource_pots.iter());

      resource_unit_sets.insert(key, module_relation_resource_pots);
    }

    let resource_pot_sets: Vec<Vec<ResourceUnitId>> = resource_unit_sets.into_values().collect();

    farm_profile_scope!("partial_bundling.gen_bucket_map");
    // gen bucket map
    for module_id in modules {
      let module = module_graph.module(module_id).unwrap();

      // Skip the external modules
      if module.external {
        continue;
      }

      if module.resource_pot.is_some() {
        panic!(
          "Module {:?} has already been assigned to a resource pot: {:?}.",
          module_id,
          module.resource_pot.as_ref().unwrap()
        );
      }

      let module_relation_resource_pot_ids: Vec<ResourceUnitId> = {
        let mut result: HashSet<ResourceUnitId> = Default::default();

        for module_id in module.module_groups.iter() {
          if let Some(resource_unit) = module_group_redirect_resource_map.get(module_id) {
            result.insert(resource_unit.clone());
          }
        }

        result.into_iter().collect()
      };

      context
        .config
        .partial_bundling
        .module_buckets
        .iter()
        .filter(|bucket_config| {
          let regex = &bucket_config.test;
          regex.is_empty() || regex.iter().any(|r| r.is_match(&module_id.to_string()))
        })
        .for_each(|bucket_config| {
          let mut relation_resource_all_pots = vec![];

          for resource_pot_set in resource_pot_sets.iter() {
            if is_subset(resource_pot_set, &module_relation_resource_pot_ids) {
              let resources: HashSet<ResourceUnitId> = resource_pot_set.iter().cloned().collect();
              relation_resource_all_pots.push(resources);
            };
          }

          for resource_pot_id in module_relation_resource_pot_ids.iter() {
            let sets = HashSet::from([resource_pot_id.clone()]);
            relation_resource_all_pots.push(sets);
          }

          for resource_units in relation_resource_all_pots.into_iter() {
            let module_bucket_id = format!(
              "{}-{}",
              bucket_config.name,
              ids_to_string(resource_units.iter())
            )
            .into();

            add_module_to_bucket(
              module_bucket_id,
              &mut module_bucket_map,
              bucket_config.clone(),
              resource_units,
              module_id,
              &module.module_type,
              module.size,
            );
          }
        });
    }

    farm_profile_scope!("partial_bundling.remove_bucket_by_min_size");
    // remove module bucket by min_size
    let remove_module_buckets = module_bucket_map
      .values_mut()
      .filter_map(|module_bucket| {
        if let Some(min_size) = module_bucket.config.min_size {
          let size = &module_bucket.size;

          let size = size
            .iter()
            .filter(|(_, v)| **v != 0 && **v < min_size)
            .map(|(k, _)| k.clone())
            .collect::<Vec<_>>();

          if size.is_empty() {
            return None;
          }

          let new_modules: HashSet<ModuleId> = module_bucket
            .take_modules()
            .into_iter()
            .filter_map(|module_id| {
              let module_type = module_graph
                .module(&module_id)
                .map(|module| &module.module_type)
                .unwrap();
              if size.contains(module_type) {
                None
              } else {
                Some(module_id)
              }
            })
            .collect();

          module_bucket.replace_modules(new_modules);

          if module_bucket.modules().is_empty() {
            return Some(module_bucket.id.clone());
          }
        }
        None
      })
      .collect::<Vec<_>>();

    remove_module_buckets.iter().for_each(|bucket_id| {
      module_bucket_map.remove(bucket_id);
    });

    farm_profile_scope!("partial_bundling.module_bucket_to_resource_unit");

    let mut module_bucket_ids: HashSet<ModuleBucketId> =
      module_bucket_map.keys().cloned().collect();

    // gen resource unit by module_bucket
    while !module_bucket_ids.is_empty() {
      let cur_process_module_bucket_id =
        module_bucket::find_best_process_bucket(&module_bucket_ids, &module_bucket_map);

      let cur_process_module_bucket_id = module_bucket_ids
        .take(&cur_process_module_bucket_id)
        .unwrap();

      let mut module_bucket = module_bucket_map
        .remove(&cur_process_module_bucket_id)
        .expect("failed read module bucket");

      let reuse_existing_resource_unit = module_bucket.config.reuse_existing_resource_pot;
      let mut reuse_resource_unit_id = None;

      if reuse_existing_resource_unit {
        // TODO: improve find best resource pot
        reuse_resource_unit_id = module_bucket.resource_units.iter().next().cloned();
        module_bucket
          .resource_units
          .remove(reuse_resource_unit_id.as_ref().unwrap());
      }

      let mut bucket_resource_units = module_bucket.resource_units.clone();

      // remove bucket resource unit of groups > max request
      if let Some(max_requests) = module_bucket.config.max_concurrent_requests {
        let mut result = vec![];
        for resource_unit_id in bucket_resource_units.iter() {
          let groups = resource_group.deps(resource_unit_id);
          if groups.len() >= max_requests as usize {
            result.push(resource_unit_id.clone());
          };
        }
        result.iter().for_each(|resource_pot_id| {
          bucket_resource_units.remove(resource_pot_id);
        });
      }

      // transfer to other resource_unit
      if bucket_resource_units.len() < module_bucket.resource_units.len() {
        if reuse_existing_resource_unit {
          bucket_resource_units.insert(reuse_resource_unit_id.clone().unwrap());
        }

        let bucket_config = module_bucket.config.clone();

        if !bucket_resource_units.is_empty() {
          let module_bucket_id: ModuleBucketId = format!(
            "{}-{}",
            bucket_config.name,
            ids_to_string(bucket_resource_units.iter())
          )
          .into();

          for module_id in module_bucket.modules().clone() {
            let module = module_graph.module(&module_id).unwrap();
            module_bucket_ids.insert(module_bucket_id.clone());
            add_module_to_bucket(
              module_bucket_id.clone(),
              &mut module_bucket_map,
              bucket_config.clone(),
              bucket_resource_units.clone(),
              &module_id,
              &module.module_type,
              module.size,
            );
          }
        }

        continue;
      }

      if module_bucket.modules().is_empty() {
        continue;
      }

      let new_name = module_bucket.config.name.clone();

      let resource_pot_group = if reuse_existing_resource_unit {
        resource_group
          .group_mut(&reuse_resource_unit_id.unwrap())
          .unwrap()
      } else {
        let resource_pot = ResourceUnit::new(new_name);
        let id = resource_pot.id.clone();

        resource_group.add_resource_pot(resource_pot);

        resource_group.group_mut(&id).unwrap()
      };

      let resource_unit_id = resource_pot_group.resource_unit.id.clone();

      bucket_resource_units
        .iter()
        .for_each(|resource_pot_id_of_module_group| {
          resource_pot_group.add_group(resource_pot_id_of_module_group)
        });

      for module_id in module_bucket.modules() {
        let module = module_graph.module_mut(module_id).unwrap();

        // delete in bucket module
        for bucket_id in module_bucket_ids.iter() {
          // TODO: this bucket resource pot is out module bucket subset
          let bucket = module_bucket_map.get_mut(bucket_id).unwrap();
          bucket.remove_module(module_id, &module.module_type, module.size);
        }

        // delete in other resource_pot module
        for resource_pot_id in bucket_resource_units.iter() {
          resource_group
            .group_mut(resource_pot_id)
            .unwrap()
            .remove_module(&module.id);
        }

        let resource_pot_group = resource_group.group_mut(&resource_unit_id).unwrap();

        resource_pot_group.add_module(module_id);
      }
    }

    resource_group.clean_empty_resources();

    farm_profile_scope!("partial_bundling.split_resource_unit");
    let keys: Vec<_> = resource_group
      .resource_pot_group_map
      .keys()
      .cloned()
      .collect();
    keys.iter().for_each(|resource_unit_id| {
      split_resource_unit::split_resource_by_module_metadata(
        &mut resource_group,
        resource_unit_id,
        &mut module_graph,
      );
    });

    let resources = resource_group.to_resources();

    resources.iter().for_each(|resource_pot| {
      resource_pot.modules().iter().for_each(|module_id| {
        let module = module_graph.module_mut(module_id).unwrap();
        module.resource_pot = Some(resource_pot.id.clone());
      });
    });

    Ok(Some(resources))
  }
}

impl FarmPluginPartialBundling {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

pub fn module_group_graph_from_entries(
  entries: &Vec<ModuleId>,
  module_graph: &mut ModuleGraph,
) -> ModuleGroupGraph {
  let mut module_group_graph = ModuleGroupGraph::new();
  let mut edges = vec![];
  let mut visited = HashSet::new();

  for entry in entries.clone() {
    let (group, dynamic_dependencies) = module_group_from_entry(&entry, module_graph);
    edges.extend(
      dynamic_dependencies
        .clone()
        .into_iter()
        .map(|dep| (group.id.clone(), dep)),
    );

    module_group_graph.add_module_group(group);

    visited.insert(entry);
    let mut queue = VecDeque::from(dynamic_dependencies);

    while !queue.is_empty() {
      let head = queue.pop_front().unwrap();

      if visited.contains(&head) {
        continue;
      }

      visited.insert(head.clone());

      let (group, dynamic_dependencies) = module_group_from_entry(&head, module_graph);
      edges.extend(
        dynamic_dependencies
          .clone()
          .into_iter()
          .map(|dep| (group.id.clone(), dep)),
      );

      module_group_graph.add_module_group(group);
      queue.extend(dynamic_dependencies);
    }
  }

  for (from, to) in &edges {
    module_group_graph.add_edge(from, to);
  }

  module_group_graph
}

/// get module group start from a entry. return (module group, dynamic dependencies)
/// traverse the module graph using bfs, stop when reach a dynamic dependency
fn module_group_from_entry(
  entry: &ModuleId,
  graph: &mut ModuleGraph,
) -> (ModuleGroup, Vec<ModuleId>) {
  let mut visited = HashSet::new();
  let mut module_group = ModuleGroup::new(entry.clone());
  let mut dynamic_entries = vec![];

  graph
    .module_mut(entry)
    .unwrap()
    .module_groups
    .insert(entry.clone());

  visited.insert(entry.clone());

  let deps = graph
    .dependencies(entry)
    .into_iter()
    .map(|(k, v)| (k, v.is_dynamic()))
    .collect::<Vec<_>>();

  for (dep, is_dynamic) in deps {
    if is_dynamic {
      dynamic_entries.push(dep);
    } else {
      // visited all dep and its dependencies using BFS
      let mut queue = VecDeque::new();
      queue.push_back(dep.clone());

      while !queue.is_empty() {
        let head = queue.pop_front().unwrap();

        if visited.contains(&head) {
          continue;
        }

        visited.insert(head.clone());
        module_group.add_module(head.clone());
        graph
          .module_mut(&head)
          .unwrap()
          .module_groups
          .insert(entry.clone());

        for (dep, edge) in graph.dependencies(&head) {
          if edge.is_dynamic() {
            dynamic_entries.push(dep);
          } else {
            queue.push_back(dep);
          }
        }
      }
    }
  }

  (module_group, dynamic_entries)
}

#[cfg(test)]
mod tests {
  use std::{collections::HashMap, sync::Arc};

  use farmfe_core::{
    context::CompilationContext,
    hashbrown::HashSet,
    parking_lot::RwLock,
    plugin::{Plugin, PluginHookContext},
  };
  #[cfg(test)]
  use farmfe_testing_helpers::construct_test_module_graph;
  use farmfe_testing_helpers::construct_test_module_group_graph;

  use crate::{module_group_from_entry as mgfe, FarmPluginPartialBundling};

  #[test]
  fn analyze_module_graph() {
    let plugin = FarmPluginPartialBundling {};
    let mut context = CompilationContext::new(Default::default(), vec![]).unwrap();
    let graph = construct_test_module_graph();

    let _ = std::mem::replace(&mut context.module_graph, Box::new(RwLock::new(graph)));
    let context = Arc::new(context);
    let mut module_graph = context.module_graph.write();

    let module_group_graph = plugin
      .analyze_module_graph(
        &mut module_graph,
        &context,
        &PluginHookContext {
          caller: None,
          meta: HashMap::new(),
        },
      )
      .unwrap()
      .unwrap();

    assert_eq!(module_group_graph.len(), 5);
    assert!(module_group_graph.has(&"A".into()));
    assert!(module_group_graph.has(&"B".into()));
    assert!(module_group_graph.has(&"D".into()));
    assert!(module_group_graph.has(&"F".into()));
    assert!(module_group_graph.has(&"G".into()));

    let module_group_a = module_group_graph.module_group(&"A".into()).unwrap();
    assert_eq!(module_group_a.id, "A".into());
    assert_eq!(
      module_group_a.modules(),
      &HashSet::from(["A".into(), "C".into()])
    );

    let module_group_b = module_group_graph.module_group(&"B".into()).unwrap();
    assert_eq!(module_group_b.id, "B".into());
    assert_eq!(
      module_group_b.modules(),
      &HashSet::from(["B".into(), "D".into(), "E".into()])
    );

    let module_group_d = module_group_graph.module_group(&"D".into()).unwrap();
    assert_eq!(module_group_d.id, "D".into());
    assert_eq!(module_group_d.modules(), &HashSet::from(["D".into()]));

    let module_group_f = module_group_graph.module_group(&"F".into()).unwrap();
    assert_eq!(module_group_f.id, "F".into());
    assert_eq!(
      module_group_f.modules(),
      &HashSet::from(["F".into(), "A".into(), "C".into()])
    );

    let module_group_g = module_group_graph.module_group(&"G".into()).unwrap();
    assert_eq!(module_group_g.id, "G".into());
    assert_eq!(module_group_g.modules(), &HashSet::from(["G".into()]));

    let test_pairs = vec![(
      "A",
      vec!["A", "F"],
      ("B", vec!["B"]),
      ("C", vec!["A", "F"]),
      ("D", vec!["D", "B"]),
      ("E", vec!["B"]),
      ("F", vec!["F"]),
      ("G", vec!["G"]),
    )];

    for tp in test_pairs {
      let m_a = module_graph.module_mut(&tp.0.into()).unwrap();
      assert_eq!(m_a.module_groups.len(), tp.1.len());

      for g_id in tp.1 {
        assert!(m_a.module_groups.contains(&g_id.into()));
      }
    }
  }

  #[test]
  fn module_group_from_entry() {
    let mut graph = construct_test_module_graph();

    let (module_group, de) = mgfe(&"A".into(), &mut graph);
    assert_eq!(de, vec!["F".into(), "D".into()]);
    assert_eq!(module_group.id, "A".into());
    assert_eq!(
      module_group.modules(),
      &HashSet::from(["A".into(), "C".into()])
    );
    assert!(graph
      .module(&"A".into())
      .unwrap()
      .module_groups
      .contains(&"A".into()));
    assert!(graph
      .module(&"C".into())
      .unwrap()
      .module_groups
      .contains(&"A".into()));
  }

  #[test]
  fn module_group_graph_from_entries() {
    let mut graph = construct_test_module_graph();

    let entries = vec!["A".into(), "B".into()];
    let module_group_graph = crate::module_group_graph_from_entries(&entries, &mut graph);
    let final_module_group_graph = construct_test_module_group_graph();

    assert_eq!(module_group_graph, final_module_group_graph);
  }
}
