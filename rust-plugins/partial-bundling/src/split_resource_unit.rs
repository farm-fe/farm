use farmfe_core::{
  hashbrown::HashMap,
  module::{module_graph::ModuleGraph, ModuleId, ModuleType},
};

use crate::{ResourceGroup, ResourceUnitId};

pub fn split_resource_by_module_metadata(
  resource_group: &mut ResourceGroup,
  resource_unit_id: &ResourceUnitId,
  module_graph: &mut ModuleGraph,
) {
  if resource_group
    .resource_unit(resource_unit_id)
    .is_some_and(|resource_unit| resource_unit.modules().is_empty())
  {
    return;
  }

  let resource_unit_group = resource_group.group_mut(resource_unit_id).unwrap();
  let mut resource_map: HashMap<ModuleType, Vec<ModuleId>> = HashMap::new();

  for module_id in resource_unit_group.resource_unit.take_modules() {
    let module = module_graph.module(&module_id).unwrap();

    resource_map
      .entry(module.module_type.clone())
      .or_insert_with(Vec::new)
      .push(module_id.clone());
  }

  if resource_map.len() == 1 {
    let (module_type, modules) = resource_map.into_iter().next().unwrap();
    resource_unit_group
      .resource_unit
      .replace_modules(modules.into_iter().collect());
    resource_unit_group.resource_unit.resource_pot_type = Some(module_type.into());
    return;
  }

  let new_resource_pots = resource_map
    .into_iter()
    .map(|(module_type, modules)| {
      let mut unit_group = resource_unit_group.clone();

      if unit_group.groups.remove(resource_unit_id) {
        unit_group
          .groups
          .insert(unit_group.resource_unit.id.clone());
      };

      unit_group.resource_unit.resource_pot_type = Some(module_type.into());

      modules.into_iter().for_each(|module_id| {
        if module_graph.entries.contains_key(&module_id) {
          unit_group.resource_unit.entry_module = Some(module_id.clone());
        }

        unit_group.resource_unit.add_module(module_id);
      });

      unit_group
    })
    .collect::<Vec<_>>();

  new_resource_pots
    .into_iter()
    .for_each(|resource_unit_group| {
      resource_group.add_unit_group(resource_unit_group);
    });
  resource_group.remove_resource_group(resource_unit_id);
}
