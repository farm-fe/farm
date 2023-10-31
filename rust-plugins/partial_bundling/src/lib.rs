#![feature(is_some_and)]

use std::{collections::VecDeque, path::PathBuf, sync::Arc};

use config::{PartialBundlingConfig, PartialBundlingModuleBucketsConfig};
use farmfe_core::{
  config::{config_regex::ConfigRegex, Config},
  context::CompilationContext,
  farm_profile_function, farm_profile_scope,
  hashbrown::{HashMap, HashSet},
  module::{
    module_graph::ModuleGraph,
    module_group::{ModuleGroup, ModuleGroupGraph},
    ModuleId, ModuleType,
  },
  plugin::{Plugin, PluginHookContext},
  resource::resource_pot::{ResourcePot, ResourcePotType},
  serde_json,
};
use module_bucket::{ModuleBucket, ModuleBucketId};

mod config;
mod module_bucket;
mod resource_group;
mod split_resource_unit;

use resource_group::{ids_to_string, is_subset, ResourceGroup, ResourceUnit, ResourceUnitId};

fn try_get_filename(module_id: &ModuleId, named_map: &HashMap<ModuleId, String>) -> String {
  let mut result = Vec::new();

  let mut filename = PathBuf::from(module_id.to_string());

  if filename.extension().is_some() {
    filename.set_extension("");
  }

  loop {
    if let Some(name) = filename.file_name() {
      result.insert(0, name.to_string_lossy().to_string());
      filename.pop();
      if !named_map
        .values()
        .any(|name| name == &result.join("_").replace("/", "_"))
      {
        break;
      }
    } else {
      break;
    };
  }

  return result.join("_").replace("/", "_");
}

#[derive(Debug, Clone)]
struct ModuleChain {
  has_dynamic: bool,
  chains: Vec<(bool, ModuleId)>,
  last: ModuleId,
  visite: HashSet<String>,
}

fn find_module_importer_chains(
  module_id: &ModuleId,
  module_graph: &ModuleGraph,
) -> Vec<ModuleChain> {
  let mut queen = VecDeque::from([ModuleChain {
    has_dynamic: false,
    chains: Vec::from([(false, module_id.clone())]),
    last: module_id.clone(),
    visite: HashSet::new(),
  }]);
  let mut result: Vec<ModuleChain> = Vec::new();

  let mut module_groups = module_graph
    .module(module_id)
    .map(|item| item.module_groups.clone())
    .unwrap();

  'out: while !queen.is_empty() {
    let mut module_chain = queen.pop_back().unwrap();

    let last_import = &module_chain.last;

    let last_import_modules_groups = &module_graph.module(last_import).unwrap().module_groups;
    if !module_groups
      .iter()
      .any(|item| last_import_modules_groups.contains(item))
    {
      continue;
    }

    let dynamic_importer = module_graph
      .module_importer(last_import)
      .into_iter()
      .filter(|item| {
        module_graph
          .edge_info(&item, last_import)
          .is_some_and(|item| item.is_dynamic())
      })
      .collect::<HashSet<_>>();

    for dynamic_importer_id in dynamic_importer {
      let mut p = module_chain.clone();

      p.has_dynamic = true;

      p.chains.push((true, dynamic_importer_id.clone()));

      p.last = dynamic_importer_id;

      queen.push_back(p);
    }

    for module_groups_id in last_import_modules_groups {
      let importer_module_groups = &module_graph
        .module(&module_groups_id)
        .unwrap()
        .module_groups;

      let hash = module_groups_id.hash();

      if !module_groups
        .iter()
        .any(|item| importer_module_groups.contains(item))
        || module_chain.visite.contains(&hash)
      {
        continue;
      }

      module_chain.visite.insert(hash);

      let mut new_module_chain = module_chain.clone();

      new_module_chain.last = module_groups_id.clone();

      if module_graph.entries.contains_key(&module_groups_id) {
        new_module_chain
          .chains
          .push((false, module_groups_id.clone()));
        result.push(new_module_chain.clone());
        module_groups.remove(&module_groups_id);

        if module_groups.is_empty() {
          break 'out;
        }

        continue;
      }

      queen.push_back(new_module_chain);
    }
  }

  result
}

#[derive(Debug, Default)]
pub struct FarmPluginPartialBundling {
  config: PartialBundlingConfig,
}

impl FarmPluginPartialBundling {
  fn new(config: &Config, options: String) -> Self {
    let mut partial_bundling_config: PartialBundlingConfig = serde_json::from_str(&options)
      .expect("failed parse option, please confirm to your options correct");

    partial_bundling_config
      .module_bucket
      .push(PartialBundlingModuleBucketsConfig {
        name: "i-vendor".into(),
        test: config.partial_bundling.immutable_modules.clone(),
        weight: isize::MAX,
        ..Default::default()
      });

    if partial_bundling_config
      .module_bucket
      .iter()
      .any(|bucket| bucket.name != "vendor")
    {
      partial_bundling_config
        .module_bucket
        .push(PartialBundlingModuleBucketsConfig {
          name: "vendor".into(),
          min_size: Some(1024 * 20),
          test: vec![ConfigRegex::new("[\\/]node_modules[\\/]")],
          max_concurrent_requests: Some(20),
          weight: -20,
          ..Default::default()
        });
    }

    if partial_bundling_config
      .module_bucket
      .iter()
      .any(|bucket| bucket.name != "common")
    {
      partial_bundling_config
        .module_bucket
        .push(PartialBundlingModuleBucketsConfig {
          name: "common".into(),
          min_size: Some(1024 * 100),
          max_concurrent_requests: Some(20),
          weight: -20,
          ..Default::default()
        });
    }

    Self {
      config: partial_bundling_config,
    }
  }
}

impl Plugin for FarmPluginPartialBundling {
  fn name(&self) -> &str {
    "FarmPluginWebpackPartialBundling"
  }

  fn priority(&self) -> i32 {
    99
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
    let module_group_graph = context.module_group_graph.read();

    let entries_modules: Vec<ModuleId> = module_graph.entries.keys().cloned().collect();
    let group_graph_module_groups = module_group_graph
      .module_groups()
      .iter()
      .map(|item| &item.id)
      .collect::<Vec<_>>();
    let dynamic_group_graph = group_graph_module_groups
      .iter()
      .filter(|module_group| !module_graph.entries.contains_key(&module_group))
      .collect::<Vec<_>>();

    let add_module_to_bucket = |module_bucket_id: ModuleBucketId,
                                module_bucket_map: &mut HashMap<ModuleBucketId, ModuleBucket>,
                                bucket_config: PartialBundlingModuleBucketsConfig,
                                resource_units: HashSet<ResourceUnitId>,
                                module_id: &ModuleId,
                                module_type: &ModuleType,
                                size: usize| {
      let module_bucket = module_bucket_map
        .entry(module_bucket_id.clone())
        .or_insert_with(|| ModuleBucket::new(module_bucket_id.clone(), bucket_config));

      module_bucket.add_module(module_id.clone(), module_type, size);

      for resource_pot_id in resource_units {
        module_bucket.add_resource_pot(resource_pot_id);
      }
    };

    let mut resource_group: ResourceGroup = ResourceGroup::new();

    type ModduleGroupRedirectResourceMap = HashMap<ModuleId, ResourceUnitId>;
    let mut module_group_redirect_resource_unit_map: ModduleGroupRedirectResourceMap =
      HashMap::new();
    let mut module_group_map: HashMap<ModuleId, Vec<ModuleId>> = HashMap::new();

    let mut resource_unit_sets: HashMap<String, Vec<ResourceUnitId>> = Default::default();
    let mut named_map: HashMap<ModuleId, String> = HashMap::new();
    println!("entries: {:#?}", entries_modules);
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

      let has_dynamic_module_group = dynamic_group_graph
        .iter()
        .any(|item| module.module_groups.contains(item));

      let importer = if has_dynamic_module_group {
        find_module_importer_chains(module_id, &module_graph)
      } else {
        vec![]
      };

      let module_into_module_group: Vec<ModuleId> = if importer.is_empty() {
        module.module_groups.iter().cloned().collect()
      } else {
        let partiation_map: HashMap<&ModuleId, Vec<&ModuleChain>> =
          importer.iter().fold(HashMap::new(), |mut res, item| {
            res
              .entry(&item.chains.last().unwrap().1)
              .or_insert_with(Vec::new)
              .push(item);
            res
          });

        // partiation_map
        partiation_map
          .values()
          .map(|importer| {
            let module_import_type = importer.iter().fold((false, false), |mut res, item| {
              if res.0 && res.1 {
                return res;
              }

              if item.has_dynamic {
                res.0 = true;
              } else {
                res.1 = true;
              }

              return res;
            });

            match module_import_type {
              (true, true) => importer
                .iter()
                .map(|item| {
                  item
                    .chains
                    .last()
                    .map(|(_, module_id)| module_id.clone())
                    .unwrap()
                })
                .collect::<Vec<_>>(),
              (true, false) => importer
                .iter()
                .map(|item| {
                  item
                    .chains
                    .iter()
                    .find(|path| path.0)
                    .map(|item| item.1.clone())
                    .unwrap()
                })
                .collect::<Vec<_>>(),
              (_, _) => importer
                .iter()
                .map(|item| item.chains.last().unwrap().1.clone())
                .collect::<Vec<_>>(),
            }
          })
          .flatten()
          .collect::<HashSet<_>>()
          .into_iter()
          .collect::<Vec<_>>()
      };

      module_group_map.insert(module_id.clone(), module_into_module_group.clone());

      module_into_module_group.iter().for_each(|module_group| {
        if let Some(resource_unit_id) = module_group_redirect_resource_unit_map.get(module_group) {
          resource_group
            .resource_unit_mut(resource_unit_id)
            .unwrap()
            .add_module(module_id.clone());
        } else {
          if !named_map.contains_key(module_group) {
            named_map.insert(
              module_group.clone(),
              try_get_filename(module_group, &named_map),
            );
          };

          let mut resource_unit = ResourceUnit::new(named_map[module_group].clone());

          if entries_modules.contains(module_group) {
            resource_unit.entry_module = Some(module_group.clone());
          }

          module_group_redirect_resource_unit_map
            .insert(module_group.clone(), resource_unit.id.clone());

          resource_unit.add_module(module_id.clone());
          resource_group.add_resource_pot(resource_unit);
        }
      });

      if module_into_module_group.len() <= 1 {
        continue;
      }

      let mut module_relation_resource_pots: Vec<ResourceUnitId> = module_into_module_group
        .iter()
        .map(|module_group| {
          module_group_redirect_resource_unit_map
            .get(module_group)
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

      if !module.resource_pot.is_empty() {
        panic!(
          "Module {:?} has already been assigned to a resource pot: {:?}.",
          module_id,
          module
            .resource_pot
            .iter()
            .map(|item| item.to_string())
            .collect::<Vec<_>>()
        );
      }

      let module_relation_resource_pot_ids: Vec<ResourceUnitId> = {
        let mut result: HashSet<ResourceUnitId> = Default::default();

        for module_group in module_group_map[module_id].iter() {
          if let Some(resource_unit) = module_group_redirect_resource_unit_map.get(module_group) {
            result.insert(resource_unit.clone());
          }
        }

        result.into_iter().collect()
      };

      self
        .config
        .module_bucket
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
            .filter(|(_, v)| **v < min_size)
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

          module_bucket.replace_modules(&module_graph, new_modules);

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

      let resource_pot_group = if reuse_existing_resource_unit {
        resource_group
          .group_mut(&reuse_resource_unit_id.unwrap())
          .unwrap()
      } else {
        let resource_pot = ResourceUnit::new(module_bucket.config.name.clone());
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
        for bucket_resource_unit in bucket_resource_units.iter() {
          resource_group
            .resource_unit_mut(&bucket_resource_unit)
            .unwrap()
            .remove_module(module_id);
        }

        resource_group
          .resource_unit_mut(&resource_unit_id)
          .unwrap()
          .add_module(module_id.clone());
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

    let resources = resource_group.to_resources(&module_group_map);

    resources.iter().for_each(|resource_pot| {
      resource_pot.modules().iter().for_each(|module_id| {
        let module = module_graph.module_mut(module_id).unwrap();
        module.resource_pot.push(resource_pot.id.clone());
      });
    });
    resources.iter().for_each(|resource| {
      println!("resource pot: {}", resource.id.to_string());
      println!("    entry: {:?}", resource.entry_module);
      println!("    immutable: {}", resource.immutable);
      println!(
        "    module_groups: {:?}",
        resource
          .module_groups
          .iter()
          .map(|item| item.to_string())
          .collect::<Vec<_>>()
      );
      println!("    type: {:?}", resource.resource_pot_type);
      if !resource
        .modules()
        .iter()
        .any(|item| item.to_string().contains("node_modules"))
      {
        println!(
          "    modules: {:?}",
          resource
            .modules()
            .iter()
            .map(|item| item.to_string())
            .collect::<Vec<_>>()
        );
      }
    });

    Ok(Some(resources))
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
    let plugin = FarmPluginPartialBundling::default();
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
