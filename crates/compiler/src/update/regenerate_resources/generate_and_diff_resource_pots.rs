use farmfe_core::{
  context::CompilationContext,
  module::{module_group::ModuleGroupId, Module, ModuleId},
  plugin::PluginHookContext,
  resource::resource_pot::{ResourcePot, ResourcePotId},
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;

use crate::{
  generate::partial_bundling::{
    call_partial_bundling_hook, fill_necessary_fields_for_resource_pot,
    get_enforce_resource_name_for_module, get_resource_pot_id_for_enforce_resources,
    get_resource_pot_id_for_enforce_resources_by_removed_module,
  },
  update::diff_and_patch_module_graph::DiffResult,
};

/// The steps to generate and diff resource pots:
/// 1. get affected modules
/// 2. deal with enforce resource pots. If enforce resource pot is existed, patch it. Otherwise, create a new one.
/// 3. call partial bundling for other modules
/// 4. diff resource pots
///    4.1 for existing resource pot only rerender it when it's modules are changed, added or removed.
///    4.2 alway render new resource pots and remove the old ones
pub fn generate_and_diff_resource_pots(
  module_groups: &FxHashSet<ModuleGroupId>,
  diff_result: &DiffResult,
  updated_module_ids: &Vec<ModuleId>,
  removed_modules: &FxHashMap<ModuleId, Module>,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<Vec<ResourcePotId>> {
  let affected_modules = get_affected_modules(module_groups, context);

  let (enforce_resource_pot_ids, modules) = handle_enforce_resource_pots(
    &affected_modules,
    diff_result,
    updated_module_ids,
    removed_modules,
    context,
  );

  // for enforce resource pots, only rerender it when it's modules are changed, added or removed.
  let mut resources_pots =
    call_partial_bundling_hook(&modules, context, &PluginHookContext::default())?;

  // fill necessary fields for all resource pots
  {
    let mut resource_pot_map = context.resource_pot_map.write();

    enforce_resource_pot_ids.iter().for_each(|id| {
      let resource_pot = resource_pot_map.resource_pot_mut(id).unwrap();
      fill_necessary_fields_for_resource_pot(vec![resource_pot], context);
    });
    resources_pots.iter_mut().for_each(|rp| {
      fill_necessary_fields_for_resource_pot(vec![rp], context);
    });
  }

  let mut new_resource_pot_ids =
    diff_and_patch_resource_pot_map(resources_pots, &enforce_resource_pot_ids, context);

  // alway render affected resource pots resource pots
  enforce_resource_pot_ids.into_iter().for_each(|id| {
    if !new_resource_pot_ids.contains(&id) {
      new_resource_pot_ids.insert(id);
    }
  });

  Ok(new_resource_pot_ids.into_iter().collect())
}

fn get_affected_modules(
  module_groups: &FxHashSet<ModuleGroupId>,
  context: &Arc<CompilationContext>,
) -> Vec<ModuleId> {
  let module_group_graph = context.module_group_graph.read();
  // let mut enforce_resource_pots = HashSet::new();
  module_groups
    .iter()
    .fold(FxHashSet::default(), |mut acc, module_group_id| {
      let module_group = module_group_graph.module_group(module_group_id).unwrap();
      acc.extend(module_group.modules().clone());
      acc
    })
    .into_iter()
    .collect::<Vec<_>>()
}

#[derive(Debug, PartialEq, Eq)]
enum ChangedModuleType {
  Added,
  Removed,
  Updated,
}

/// Handle the enforce resource pots.
/// return (enforce_resource_pot_ids, un_enforced_modules)
fn handle_enforce_resource_pots(
  affected_modules: &Vec<ModuleId>,
  diff_result: &DiffResult,
  updated_module_ids: &Vec<ModuleId>,
  removed_modules: &FxHashMap<ModuleId, Module>,
  context: &Arc<CompilationContext>,
) -> (Vec<ResourcePotId>, Vec<ModuleId>) {
  let module_graph = context.module_graph.read();
  let mut resource_pot_map = context.resource_pot_map.write();
  let mut un_enforced_modules = FxHashSet::default();
  let mut affected_resource_pot_ids = FxHashSet::default();

  let is_module_external = |module_id: &ModuleId| {
    let module = if let Some(module) = removed_modules.get(module_id) {
      module
    } else {
      module_graph.module(module_id).unwrap()
    };

    module.external
  };

  let mut handle_changed_modules = |module_ids: &FxHashSet<ModuleId>, ty: ChangedModuleType| {
    for module_id in module_ids {
      // ignore external module
      if is_module_external(module_id) {
        continue;
      }

      if let Some(name) = get_enforce_resource_name_for_module(
        module_id,
        &context.config.partial_bundling.enforce_resources,
      ) {
        let (resource_pot_type, resource_pot_name, resource_pot_id) =
          if ty != ChangedModuleType::Removed {
            get_resource_pot_id_for_enforce_resources(name, module_id, &module_graph)
          } else {
            let module = removed_modules.get(module_id).unwrap_or_else(|| {
              panic!("can not find module {module_id:?}");
            });
            get_resource_pot_id_for_enforce_resources_by_removed_module(name, module)
          };
        affected_resource_pot_ids.insert(resource_pot_id.clone());

        if let Some(resource_pot) = resource_pot_map.resource_pot_mut(&resource_pot_id) {
          if ty == ChangedModuleType::Added {
            resource_pot.add_module(module_id.clone());
          } else if ty == ChangedModuleType::Removed {
            resource_pot.remove_module(module_id);
          }
        } else if ty != ChangedModuleType::Removed {
          let mut resource_pot = ResourcePot::new(resource_pot_name, resource_pot_type);
          resource_pot.add_module(module_id.clone());
          resource_pot_map.add_resource_pot(resource_pot);
        }
      }
    }
  };

  handle_changed_modules(
    &updated_module_ids
      .clone()
      .into_iter()
      .collect::<FxHashSet<_>>(),
    ChangedModuleType::Updated,
  );
  handle_changed_modules(&diff_result.added_modules, ChangedModuleType::Added);
  handle_changed_modules(&diff_result.removed_modules, ChangedModuleType::Removed);

  // Filter out the modules that are not in any enforce resource pot
  for module_id in affected_modules {
    if is_module_external(module_id) {
      continue;
    }

    if let Some(name) = get_enforce_resource_name_for_module(
      module_id,
      &context.config.partial_bundling.enforce_resources,
    ) {
      let (_, _, resource_pot_id) =
        get_resource_pot_id_for_enforce_resources(name, module_id, &module_graph);

      if !affected_resource_pot_ids.contains(&resource_pot_id) {
        affected_resource_pot_ids.insert(resource_pot_id);
      }
    } else {
      un_enforced_modules.insert(module_id.clone());
    }
  }

  // remove the resource pot if it's modules are empty
  for id in &affected_resource_pot_ids {
    let resource_pot = resource_pot_map.resource_pot_mut(id).unwrap_or_else(|| {
      panic!("resource pot not found: {id:?}");
    });

    if resource_pot.modules().is_empty() {
      resource_pot_map.remove_resource_pot(id);
    }
  }

  let mut modules = un_enforced_modules.into_iter().collect::<Vec<_>>();
  modules.sort();

  (
    affected_resource_pot_ids.into_iter().collect::<Vec<_>>(),
    modules,
  )
}

fn diff_and_patch_resource_pot_map(
  resources_pots: Vec<ResourcePot>,
  enforce_resource_pot_ids: &Vec<String>,
  context: &Arc<CompilationContext>,
) -> FxHashSet<ResourcePotId> {
  let resources_pots_ids = resources_pots
    .iter()
    .map(|rp| rp.id.clone())
    .collect::<Vec<_>>();

  let module_graph = context.module_graph.read();
  let mut resource_pot_map = context.resource_pot_map.write();
  let mut module_group_graph = context.module_group_graph.write();

  let mut new_resource_pot_ids = FxHashSet::default();

  for mut resource_pot in resources_pots {
    let mut module_groups = FxHashSet::default();

    for module_id in resource_pot.modules() {
      let module = module_graph.module(module_id).unwrap();
      module_groups.extend(module.module_groups.clone());
    }

    resource_pot.module_groups = module_groups.clone();

    for module_group_id in module_groups {
      let module_group = module_group_graph
        .module_group_mut(&module_group_id)
        .unwrap();
      let mut resources_pots_to_remove = vec![];

      // Remove the old resource pots
      for resource_pot in module_group.resource_pots() {
        if !resources_pots_ids.contains(resource_pot)
          && !enforce_resource_pot_ids.contains(resource_pot)
        {
          resources_pots_to_remove.push(resource_pot.clone());

          if resource_pot_map.has_resource_pot(resource_pot) {
            let resource_pot = resource_pot_map
              .remove_resource_pot(resource_pot)
              .unwrap_or_else(|| {
                panic!("The resource pot {resource_pot:?} should be in the resource pot map")
              });

            // also remove the related resource
            let mut resource_maps = context.resources_map.lock();

            for resource in resource_pot.resources() {
              resource_maps.remove(resource);
            }
          }
        }
      }

      for resource_pot in resources_pots_to_remove {
        module_group.remove_resource_pot(&resource_pot);
      }

      if module_group.has_resource_pot(&resource_pot.id) {
        // the resource pot is already in the module group
        continue;
      } else {
        new_resource_pot_ids.insert(resource_pot.id.clone());
        module_group.add_resource_pot(resource_pot.id.clone());
      }
    }

    if !resource_pot_map.has_resource_pot(&resource_pot.id) {
      new_resource_pot_ids.insert(resource_pot.id.clone());
      resource_pot_map.add_resource_pot(resource_pot);
    }
  }

  new_resource_pot_ids
}

#[cfg(test)]
mod test_handle_enforce_resource_pots;

#[cfg(test)]
mod test_generate_and_diff_resource_pots;
