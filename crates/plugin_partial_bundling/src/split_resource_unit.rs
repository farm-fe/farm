use farmfe_core::{
  hashbrown::HashMap,
  module::{module_graph::ModuleGraph, ModuleId, ModuleType},
};

use crate::{ResourceGroup, ResourceUnit, ResourceUnitId};

pub fn split_resource_by_module_metadata(
  resource_group: &mut ResourceGroup,
  resource_unit_id: &ResourceUnitId,
  module_graph: &mut ModuleGraph,
) {
  if resource_group
    .resource_pot(resource_unit_id)
    .is_some_and(|resource_unit| resource_unit.modules().is_empty())
  {
    return;
  }

  let resource_unit = resource_group.resource_pot_mut(resource_unit_id).unwrap();
  let resource_unit_name = resource_unit.name.clone();
  let mut resource_map: HashMap<(ModuleType, bool), Vec<ModuleId>> = HashMap::new();
  let is_entry = resource_unit.entry_module.is_some();

  for module_id in resource_unit.take_modules() {
    let module = module_graph.module(&module_id).unwrap();

    resource_map
      .entry((module.module_type.clone(), module.immutable))
      .or_insert_with(Vec::new)
      .push(module_id.clone());
  }

  if resource_map.len() == 1 {
    let ((module_type, inmutable), modules) = resource_map.into_iter().next().unwrap();
    resource_unit.replace_modules(modules.into_iter().collect());
    resource_unit.resource_pot_type = Some(module_type.into());
    resource_unit.immutable = inmutable;
    return;
  }

  let new_resource_pots = resource_map
    .into_iter()
    .map(|((module_type, immutable), modules)| {
      let resource_unit_name = resource_unit_name.clone();

      let mut new_resource_unit = ResourceUnit::new(resource_unit_name);

      new_resource_unit.resource_pot_type = Some(module_type.into());
      new_resource_unit.immutable = immutable;

      modules.into_iter().for_each(|module_id| {
        if is_entry && module_graph.entries.contains_key(&module_id) {
          new_resource_unit.entry_module = Some(module_id.clone());
        }

        new_resource_unit.add_module(module_id);
      });

      new_resource_unit
    })
    .collect::<Vec<_>>();

  new_resource_pots.into_iter().for_each(|resource_unit| {
    resource_group.add_resource_pot(resource_unit);
  });
  resource_group.remove_resource_pot(resource_unit_id);
}
