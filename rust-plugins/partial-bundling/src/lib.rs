#![feature(is_some_and)]

use config::{PartialBundlingConfig, PartialBundlingModuleBucketsConfig};
use farmfe_core::{
  config::{config_regex::ConfigRegex, Config},
  context::CompilationContext,
  hashbrown::{HashMap, HashSet},
  module::{module_graph::ModuleGraph, module_group::ModuleGroupGraph, ModuleId},
  plugin::{Plugin, PluginHookContext},
  resource::resource_pot::ResourcePot,
  serde_json,
};
use farmfe_macro_plugin::farm_plugin;
use std::sync::Arc;
use utils::ids_to_string;

mod config;
mod module_bucket;
mod resource_group;
mod split_resource_unit;
mod utils;

use resource_group::{ResourceGroup, ResourceUnit, ResourceUnitId};

use crate::{
  module_bucket::generate_module_buckets,
  utils::{find_module_importer_chains, try_get_filename, ModuleChain},
};

#[farm_plugin]
#[derive(Debug, Default)]
pub struct FarmPluginPartialBundling {
  config: PartialBundlingConfig,
}

impl FarmPluginPartialBundling {
  pub fn new(config: &Config, options: String) -> Self {
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
          min_size: Some(1024 * 20),
          max_concurrent_requests: Some(10),
          weight: -20,
          ..Default::default()
        });
    }

    Self {
      config: partial_bundling_config,
    }
  }
}

type ModduleGroupRedirectResourceMap = HashMap<ModuleId, ResourceUnitId>;

impl Plugin for FarmPluginPartialBundling {
  fn name(&self) -> &str {
    "FarmPluginWebpackPartialBundling"
  }

  fn priority(&self) -> i32 {
    150
  }

  /// The partial bundling algorithm's result should not be related to the order of the module group.
  /// Whatever the order of the module group is, the result should be the same.
  fn partial_bundling(
    &self,
    modules: &Vec<ModuleId>,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<Vec<ResourcePot>>> {
    let mut module_graph = context.module_graph.write();
    let module_group_graph = context.module_group_graph.read();

    let mut resource_group: ResourceGroup = ResourceGroup::new();

    let (module_group_redirect_resource_unit_map, resource_pot_sets, module_group_map) =
      pre_process_module(
        modules,
        &module_graph,
        &module_group_graph,
        &mut resource_group,
      );

    let mut module_bucket_map = module_bucket::generate_module_buckets(
      modules,
      &module_graph,
      &self.config.module_bucket,
      &module_group_redirect_resource_unit_map,
      &module_group_map,
      resource_pot_sets,
    );

    module_bucket::remove_module_bucket_by_size(&module_graph, &mut module_bucket_map);

    module_bucket::gen_resource_unit_by_module_buckets(
      &mut module_graph,
      &mut module_bucket_map,
      &mut resource_group,
    );

    resource_group.clean_empty_resources();

    let resource_units: Vec<_> = resource_group
      .resource_pot_group_map
      .keys()
      .cloned()
      .collect();

    resource_units.iter().for_each(|resource_unit_id| {
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

    Ok(Some(resources))
  }
}

// 1. gen resource pot groups set (len > 1)
// 2. gen module_group resource pot, if not exists
fn pre_process_module(
  modules: &Vec<ModuleId>,
  module_graph: &ModuleGraph,
  module_group_graph: &ModuleGroupGraph,
  resource_group: &mut ResourceGroup,
) -> (
  ModduleGroupRedirectResourceMap,
  Vec<Vec<ResourceUnitId>>,
  HashMap<ModuleId, Vec<ModuleId>>,
) {
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

  let mut module_group_map: HashMap<ModuleId, Vec<ModuleId>> = HashMap::new();
  let mut module_group_redirect_resource_unit_map: ModduleGroupRedirectResourceMap = HashMap::new();
  let mut named_map: HashMap<ModuleId, String> = HashMap::new();
  let mut resource_unit_sets: HashMap<String, Vec<ResourceUnitId>> = Default::default();

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

  (
    module_group_redirect_resource_unit_map,
    resource_unit_sets.into_values().collect(),
    module_group_map,
  )
}
