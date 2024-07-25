//! Merge module pots to resource pots in the same ModuleGroup.
//! See https://github.com/farm-fe/rfcs/blob/main/rfcs/003-partial-bundling/rfc.md#merge-module-pots-into-resource-pot

use std::collections::{HashMap, HashSet};
use std::{cmp::Ordering, usize};

use farmfe_core::config::Config;
use farmfe_core::{
  module::{module_graph::ModuleGraph, module_group::ModuleGroupId, ModuleId, ModuleType},
  resource::resource_pot::{ResourcePot, ResourcePotType},
};

use crate::{module_pot::ModulePot, utils::hash_module_ids};

#[derive(Debug, Clone)]
pub struct ModuleGroupModulePots {
  pub module_group_id: ModuleGroupId,
  /// module bucket name -> module pots
  pub module_pots: HashMap<String, Vec<ModulePot>>,
}

impl ModuleGroupModulePots {
  pub fn new(module_group_id: ModuleGroupId) -> Self {
    Self {
      module_group_id,
      module_pots: HashMap::new(),
    }
  }

  pub fn add_module_pots(&mut self, module_bucket_name: String, module_pot: Vec<ModulePot>) {
    self.module_pots.insert(module_bucket_name, module_pot);
  }

  // pub fn get_size(&self) -> usize {
  //   self.get_immutable_size() + self.get_mutable_size()
  // }

  pub fn get_mutable_size(&self) -> usize {
    self
      .module_pots
      .values()
      .map(|module_pots| {
        module_pots
          .iter()
          .filter(|module_pot| !module_pot.immutable)
          .map(|module_pot| module_pot.size)
          .sum::<usize>()
      })
      .sum()
  }

  pub fn get_immutable_size(&self) -> usize {
    self
      .module_pots
      .values()
      .map(|module_pots| {
        module_pots
          .iter()
          .filter(|module_pot| module_pot.immutable)
          .map(|module_pot| module_pot.size)
          .sum::<usize>()
      })
      .sum()
  }
}

struct CurrentGeneration<'a> {
  pub size: usize,
  pub module_pots: Vec<&'a ModulePot>,
}

impl<'a> CurrentGeneration<'a> {
  pub fn new() -> Self {
    Self {
      size: 0,
      module_pots: vec![],
    }
  }

  pub fn add_module_pot(&mut self, module_pot: &'a ModulePot) {
    self.size += module_pot.size;
    self.module_pots.push(module_pot);
  }

  pub fn modules(&self) -> HashSet<ModuleId> {
    self
      .module_pots
      .iter()
      .map(|module_pot| module_pot.modules())
      .flatten()
      .cloned()
      .collect()
  }

  pub fn module_type(&self) -> ModuleType {
    self.module_pots[0].module_type.clone()
  }

  pub fn immutable(&self) -> bool {
    self.module_pots[0].immutable
  }

  pub fn name(&self) -> Option<String> {
    let mut set = self.module_pots.iter().collect::<Vec<_>>();

    set.sort_by(|a, b| a.id.cmp(&b.id));

    set.iter().find_map(|i| i.name.clone())
  }
}

fn create_resource_by_meta<M: Into<ResourcePotType>>(
  id: String,
  modules: &HashSet<ModuleId>,
  module_type: M,
  is_with_hash: bool,
  immutable: bool,
  name: String,
) -> ResourcePot {
  let id = format!("{}_{}", id, hash_module_ids(modules));
  let resource_name = if is_with_hash {
    format!("{}_{}", name, hash_module_ids(modules))
  } else {
    name
  };

  let resource_pot_type = module_type.into();
  let mut resource_pot = ResourcePot::new(resource_name, resource_pot_type);

  resource_pot.set_resource_pot_id(id);
  resource_pot.immutable = immutable;

  for module_id in modules {
    resource_pot.add_module(module_id.clone());
  }

  resource_pot
}

/// Merge module pots to resource pots in the same ModuleGroup.
/// See https://github.com/farm-fe/rfcs/blob/main/rfcs/003-partial-bundling/rfc.md#merge-module-pots-into-resource-pot
pub fn merge_module_pots(
  module_group_module_pots: ModuleGroupModulePots,
  config: &Config,
  base_resource_pot_name: &str,
  module_graph: &ModuleGraph,
) -> Vec<ResourcePot> {
  let is_with_hash = !(config.output.filename.contains("[hash]")
    || config.output.filename.contains("[contentHash]"));
  let config = &config.partial_bundling;
  // target_concurrent_requests = 0 means no limit
  let target_concurrent_requests = if config.target_concurrent_requests == 0 {
    usize::MAX
  } else {
    config.target_concurrent_requests
  };

  // at least one request
  let immutable_request_numbers = std::cmp::max(
    (target_concurrent_requests as f32 * config.immutable_modules_weight).round() as usize,
    1,
  );
  let mutable_request_numbers =
    std::cmp::max(target_concurrent_requests - immutable_request_numbers, 1);

  let mutable_target_size = std::cmp::max(
    module_group_module_pots.get_mutable_size() / mutable_request_numbers,
    config.target_min_size,
  );
  let immutable_target_size = std::cmp::max(
    module_group_module_pots.get_immutable_size() / immutable_request_numbers,
    config.target_min_size,
  );

  let mut resource_pots = merge_resource_pots_by_buckets(
    &module_group_module_pots.module_pots,
    mutable_target_size,
    immutable_target_size,
    base_resource_pot_name,
    is_with_hash,
  );

  let mut resource_pots_size_mp = HashMap::new();

  if config.enforce_target_concurrent_requests || config.enforce_target_min_size {
    for resource_pot in &resource_pots {
      let size = get_modules_size(resource_pot.modules(), module_graph);
      resource_pots_size_mp.insert(resource_pot.id.clone(), size);
    }
  }

  // Deal with enforce target min size and enforce target concurrent requests.
  if config.enforce_target_min_size {
    resource_pots = handle_enforce_target_min_size(
      resource_pots,
      &resource_pots_size_mp,
      config.target_min_size,
      base_resource_pot_name,
      is_with_hash,
    );
  }

  if config.enforce_target_concurrent_requests {
    resource_pots = handle_enforce_target_concurrent_requests(
      resource_pots,
      &resource_pots_size_mp,
      target_concurrent_requests,
      base_resource_pot_name,
      is_with_hash,
    );
  }

  resource_pots
}

/// Abstraction of generating mutable and immutable resource pots.
fn merge_resource_pots_by_buckets(
  module_pots_map: &HashMap<String, Vec<ModulePot>>,
  mutable_target_size: usize,
  immutable_target_size: usize,
  base_resource_pot_name: &str,
  is_with_hash: bool,
) -> Vec<ResourcePot> {
  let mut final_resource_pots = vec![];

  for (_, module_pots) in module_pots_map {
    let mut current_generation_map = HashMap::<(ModuleType, bool), CurrentGeneration>::new();
    let mut resource_pots = vec![];

    if module_pots.is_empty() {
      continue;
    }

    let target_size = if module_pots[0].immutable {
      immutable_target_size
    } else {
      mutable_target_size
    };

    for module_pot in module_pots {
      if module_pot.enforce {
        resource_pots.push(create_resource_by_meta(
          base_resource_pot_name.to_string(),
          &module_pot.modules,
          module_pot.module_type.clone(),
          is_with_hash,
          module_pot.immutable,
          if let Some(ref name) = module_pot.name {
            name.clone()
          } else {
            base_resource_pot_name.to_string()
          },
        ));
        continue;
      }

      let key = (module_pot.module_type.clone(), module_pot.immutable);

      if let Some(current_generation) = current_generation_map.get_mut(&key) {
        current_generation.add_module_pot(module_pot);
      } else {
        let mut current_generation = CurrentGeneration::new();
        current_generation.add_module_pot(module_pot);
        current_generation_map.insert(key.clone(), current_generation);
      }

      // create a new resource pot if the size of the resource pot is too large
      if current_generation_map[&key].size >= target_size {
        let current_generation = current_generation_map.remove(&key).unwrap();
        let modules = current_generation.modules();

        resource_pots.push(create_resource_by_meta(
          base_resource_pot_name.to_string(),
          &modules,
          current_generation.module_type(),
          is_with_hash,
          current_generation.immutable(),
          if let Some(name) = current_generation.name() {
            name
          } else {
            base_resource_pot_name.to_string()
          },
        ));
      }
    }

    // if current_generation_map is not empty, it means that there are some modules that have not been added to the resource pot.
    if !current_generation_map.is_empty() {
      for (_, current_generation) in current_generation_map {
        let modules = current_generation.modules();

        resource_pots.push(create_resource_by_meta(
          base_resource_pot_name.to_string(),
          &modules,
          current_generation.module_type(),
          is_with_hash,
          current_generation.immutable(),
          current_generation
            .name()
            .unwrap_or(base_resource_pot_name.to_string()),
        ));
      }
    }

    final_resource_pots.extend(resource_pots);
  }
  // sort to make the order stable
  final_resource_pots.sort_by(|a, b| a.id.cmp(&b.id));

  final_resource_pots
}

fn get_modules_size(modules: Vec<&ModuleId>, module_graph: &ModuleGraph) -> usize {
  modules
    .into_iter()
    .map(|module_id| module_graph.module(module_id).unwrap().size)
    .sum()
}

/// Merge resource pots that are less than target_min_size to a new ResourcePot or into the first matched resource pot.
fn handle_enforce_target_min_size(
  resource_pots: Vec<ResourcePot>,
  resource_pots_size_mp: &HashMap<String, usize>,
  target_min_size: usize,
  base_resource_pot_name: &str,
  is_with_hash: bool,
) -> Vec<ResourcePot> {
  let mut small_resource_pots_to_merge = vec![];
  let mut resource_pot_map = resource_pots
    .into_iter()
    .map(|resource_pot| (resource_pot.id.clone(), resource_pot))
    .collect::<HashMap<_, _>>();

  for resource_pot in resource_pot_map.values() {
    let size = *resource_pots_size_mp
      .get(&resource_pot.id)
      .expect("resource pot size should be calculated");

    if size < target_min_size {
      small_resource_pots_to_merge.push((size, resource_pot.id.clone()));
    }
  }

  small_resource_pots_to_merge.sort_by(|a, b| a.0.cmp(&b.0));

  let mut merged_resource_pot_map = HashMap::<(ResourcePotType, bool), (usize, Vec<String>)>::new();

  for (size, resource_pot_id) in small_resource_pots_to_merge {
    let resource_pot = resource_pot_map.get(&resource_pot_id).unwrap();
    let mut cur_merged_size = size;

    if let Some((merged_size, merged_resource_pot_ids)) = merged_resource_pot_map.get_mut(&(
      resource_pot.resource_pot_type.clone(),
      resource_pot.immutable,
    )) {
      merged_resource_pot_ids.push(resource_pot_id.clone());
      *merged_size += size;
      cur_merged_size = *merged_size;
    } else {
      merged_resource_pot_map.insert(
        (
          resource_pot.resource_pot_type.clone(),
          resource_pot.immutable,
        ),
        (size, vec![resource_pot_id.clone()]),
      );
    }

    if cur_merged_size >= target_min_size {
      let (_, merged_resource_pot_ids) = merged_resource_pot_map
        .remove(&(
          resource_pot.resource_pot_type.clone(),
          resource_pot.immutable,
        ))
        .unwrap();
      // remove the merged resource pots and add the new merged resource pot
      let merged_resource_pot = create_merged_resource_pot(
        &merged_resource_pot_ids,
        resource_pot.resource_pot_type.clone(),
        resource_pot.immutable,
        base_resource_pot_name,
        &mut resource_pot_map,
        is_with_hash,
      );

      resource_pot_map.insert(merged_resource_pot.id.clone(), merged_resource_pot);
    }
  }

  let mut final_resource_pot_ids = resource_pot_map
    .iter()
    .map(|(id, _)| id)
    .cloned()
    .collect::<Vec<_>>();
  final_resource_pot_ids.sort();

  // merge resource pots left into the first matched resource pot
  if !merged_resource_pot_map.is_empty() {
    for ((ty, immutable), (_, merged_resource_pot_ids)) in merged_resource_pot_map {
      let mut found = false;

      for final_resource_pot_id in &final_resource_pot_ids {
        let (f_resource_pot_type, f_immutable) = {
          // this resource pot has been merged
          if !resource_pot_map.contains_key(final_resource_pot_id) {
            continue;
          }

          let final_resource_pot = resource_pot_map
            .get(final_resource_pot_id)
            .unwrap_or_else(|| panic!("resource pot {:?} does not exist", final_resource_pot_id));
          (
            final_resource_pot.resource_pot_type.clone(),
            final_resource_pot.immutable,
          )
        };

        if f_resource_pot_type == ty
          && f_immutable == immutable
          && !merged_resource_pot_ids.contains(final_resource_pot_id)
        {
          for resource_pot_id in &merged_resource_pot_ids {
            let removed_resource_pot = resource_pot_map.remove(resource_pot_id).unwrap();
            let final_resource_pot = resource_pot_map.get_mut(final_resource_pot_id).unwrap();

            for module_id in removed_resource_pot.modules() {
              final_resource_pot.add_module(module_id.clone());
            }
          }

          found = true;
          break;
        }
      }

      // total size < target_min_size, just create new resource pot
      if !found {
        let merged_resource_pot = create_merged_resource_pot(
          &merged_resource_pot_ids,
          ty,
          immutable,
          base_resource_pot_name,
          &mut resource_pot_map,
          is_with_hash,
        );
        final_resource_pot_ids.push(merged_resource_pot.id.clone());
        resource_pot_map.insert(merged_resource_pot.id.clone(), merged_resource_pot);
      }
    }
  }

  final_resource_pot_ids
    .into_iter()
    .filter_map(|id| resource_pot_map.remove(&id))
    .collect()
}

fn create_merged_resource_pot(
  merged_resource_pot_ids: &Vec<String>,
  resource_pot_type: ResourcePotType,
  immutable: bool,
  base_resource_pot_name: &str,
  resource_pot_map: &mut HashMap<String, ResourcePot>,
  is_with_hash: bool,
) -> ResourcePot {
  let mut modules = HashSet::new();

  let mut name: Option<(String, bool)> = None;
  for resource_pot_id in merged_resource_pot_ids {
    let removed_resource_pot = resource_pot_map.remove(resource_pot_id).unwrap();

    if name.is_none() {
      name = Some((removed_resource_pot.name.clone(), true));
    }

    if let Some((name, is_ready)) = name.as_mut() {
      if *is_ready && *name != removed_resource_pot.name {
        *is_ready = false;
      }
    }

    for module_id in removed_resource_pot.modules() {
      modules.insert(module_id.clone());
    }
  }

  let resource_pot_id = format!(
    "{}_{}",
    base_resource_pot_name,
    hash_module_ids(&modules) // get_sorted_module_ids_str(&module_bucket.modules())
  );

  let resource_pot_name = if let Some((name, _)) = name {
    name
  } else {
    base_resource_pot_name.to_string()
  };

  create_resource_by_meta(
    resource_pot_id.clone(),
    &modules,
    resource_pot_type.clone(),
    is_with_hash,
    immutable,
    resource_pot_name,
  )
}

/// Merge resource pots that are less than target_min_size to a new ResourcePot or into the smallest resource pot.
fn handle_enforce_target_concurrent_requests(
  mut resource_pots: Vec<ResourcePot>,
  resource_pots_size_mp: &HashMap<String, usize>,
  target_concurrent_requests: usize,
  base_resource_pot_name: &str,
  is_with_hash: bool,
) -> Vec<ResourcePot> {
  if resource_pots.len() <= target_concurrent_requests {
    return resource_pots;
  }

  // sort resource pots by size
  resource_pots.sort_by(|a, b| {
    let a_size = *resource_pots_size_mp
      .get(&a.id)
      .expect("resource pot size should be calculated");
    let b_size = *resource_pots_size_mp
      .get(&b.id)
      .expect("resource pot size should be calculated");
    let result = a_size.cmp(&b_size);

    // if size is equal, sort by id to make sure it is stable
    // Note: immutable resource pots are always smaller than mutable resource pots
    if matches!(result, Ordering::Equal) {
      if a.immutable && !b.immutable {
        return Ordering::Less;
      } else if !a.immutable && b.immutable {
        return Ordering::Greater;
      }

      return a.id.cmp(&b.id);
    }

    result
  });

  let len_to_merge = resource_pots.len() - target_concurrent_requests + 1;
  let mut resource_pots_to_merge = HashMap::new();

  for i in 0..len_to_merge {
    let key = (
      resource_pots[i].resource_pot_type.clone(),
      resource_pots[i].immutable,
    );
    let value = resource_pots_to_merge.entry(key).or_insert(vec![]);
    value.push(resource_pots[i].id.clone());
  }

  let resource_pot_ids = resource_pots
    .iter()
    .map(|resource_pot| resource_pot.id.clone())
    .collect::<Vec<_>>();

  let mut resource_pot_map = resource_pots
    .into_iter()
    .map(|resource_pot| (resource_pot.id.clone(), resource_pot))
    .collect::<HashMap<_, _>>();

  // find the first matched resource pot and merge the resource pots left into it
  for i in len_to_merge..resource_pot_ids.len() {
    let resource_pot_id = &resource_pot_ids[i];
    let resource_pot = resource_pot_map.get(resource_pot_id).unwrap();
    let key = (
      resource_pot.resource_pot_type.clone(),
      resource_pot.immutable,
    );

    if let Some(mut resource_pot_ids) = resource_pots_to_merge.remove(&key) {
      resource_pot_ids.push(resource_pot.id.clone());
      let merged_resource_pot = create_merged_resource_pot(
        &resource_pot_ids,
        resource_pot.resource_pot_type.clone(),
        resource_pot.immutable,
        base_resource_pot_name,
        &mut resource_pot_map,
        is_with_hash,
      );
      resource_pot_map.insert(merged_resource_pot.id.clone(), merged_resource_pot);
    }
  }

  // if resource_pots_to_merge is not empty, it means that there are some resource pots that have not been merged.
  if !resource_pots_to_merge.is_empty() {
    for ((ty, immutable), resource_pot_ids) in resource_pots_to_merge {
      let merged_resource_pot = create_merged_resource_pot(
        &resource_pot_ids,
        ty,
        immutable,
        base_resource_pot_name,
        &mut resource_pot_map,
        is_with_hash,
      );
      resource_pot_map.insert(merged_resource_pot.id.clone(), merged_resource_pot);
    }
  }

  let mut resource_pots = resource_pot_map
    .into_iter()
    .map(|(_, v)| v)
    .collect::<Vec<_>>();
  resource_pots.sort_by(|a, b| a.id.cmp(&b.id));

  resource_pots
}

#[cfg(test)]
mod common;
#[cfg(test)]
mod test_boundaries;
#[cfg(test)]
mod test_default;
#[cfg(test)]
mod test_enforce_configs;
