use farmfe_core::{
  hashbrown::{HashMap, HashSet},
  module::ModuleId,
  resource::resource_pot::{ResourcePot, ResourcePotId, ResourcePotType},
};
use farmfe_toolkit::hash::sha256;
use farmfe_toolkit::rand::{self, distributions::Alphanumeric, Rng};

pub fn is_subset<T: PartialEq>(v1: &[T], v2: &[T]) -> bool {
  v1.iter().all(|item| v2.contains(item))
}

pub fn resource_pot_ids_to_string<'a, I: Iterator<Item = &'a K>, K>(resources: I) -> String
where
  K: ToString + 'a,
{
  let mut module_group_ids = resources.map(|id| id.to_string()).collect::<Vec<String>>();

  module_group_ids.sort();

  module_group_ids
    .clone()
    .into_iter()
    .collect::<Vec<String>>()
    .join("_")
}

#[derive(Debug)]
pub struct ResourceUnitGroup {
  pub resource_unit: ResourceUnit,
  pub groups: HashSet<ResourceUnitId>,
}

impl From<ResourceUnit> for ResourceUnitGroup {
  fn from(value: ResourceUnit) -> Self {
    ResourceUnitGroup {
      resource_unit: value,
      groups: HashSet::new(),
    }
  }
}

impl ResourceUnitGroup {
  pub fn add_group(&mut self, resource_pot_id: &ResourceUnitId) {
    self.groups.insert(resource_pot_id.clone());
  }

  pub fn remove_module(&mut self, module_id: &ModuleId) -> bool {
    self.resource_unit.remove_module(module_id)
  }

  pub fn add_module(&mut self, module_id: &ModuleId) {
    self.resource_unit.add_module(module_id.clone());
  }
}

#[derive(Default, Debug, Clone)]
pub struct ResourceUnit {
  modules: HashSet<ModuleId>,
  pub resource_pot_type: Option<ResourcePotType>,
  pub immutable: bool,
  pub name: String,
  pub entry_module: Option<ModuleId>,
  pub id: ResourceUnitId,
}

pub type ResourceUnitId = String;

impl ResourceUnit {
  pub fn new(name: String) -> Self {
    Self {
      name,
      id: rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect::<String>(),
      ..Default::default()
    }
  }

  pub fn add_module(&mut self, module_id: ModuleId) {
    self.modules.insert(module_id);
  }

  pub fn remove_module(&mut self, module_id: &ModuleId) -> bool {
    self.modules.remove(module_id)
  }

  pub fn modules(&self) -> &HashSet<ModuleId> {
    &self.modules
  }

  pub fn replace_modules(&mut self, modules: HashSet<ModuleId>) {
    self.modules = modules;
  }

  pub fn take_modules(&mut self) -> HashSet<ModuleId> {
    self.modules.drain().collect()
  }
}

impl Into<ResourcePot> for ResourceUnit {
  fn into(self) -> ResourcePot {
    let resource_pot_type = self.resource_pot_type.unwrap_or(ResourcePotType::Js);

    let resource_pot_id = format!(
      "{}-{}",
      self.name,
      sha256(
        format!(
          "{}-{:?}-{}",
          self.immutable,
          resource_pot_type,
          resource_pot_ids_to_string(self.modules.iter())
        )
        .as_bytes(),
        8
      )
    );

    let mut resource_pot = ResourcePot::new(ResourcePotId::new(resource_pot_id), resource_pot_type);

    resource_pot.immutable = self.immutable;
    resource_pot.entry_module = self.entry_module;
    resource_pot.replace_module(self.modules);

    resource_pot
  }
}

#[derive(Debug)]
pub struct ResourceGroup {
  pub resource_pot_group_map: HashMap<ResourceUnitId, ResourceUnitGroup>,
}

impl ResourceGroup {
  pub fn add_resource_pot(&mut self, resource_unit: ResourceUnit) {
    let resource_unit_id = resource_unit.id.clone();

    self.resource_pot_group_map.insert(
      resource_unit_id.clone(),
      ResourceUnitGroup {
        resource_unit,
        groups: HashSet::from([resource_unit_id]),
      },
    );
  }

  pub fn remove_resource_pot(&mut self, resource_unit_id: &ResourceUnitId) {
    self.resource_pot_group_map.remove(resource_unit_id);
  }

  // resource_unit depend on resource_unit
  pub fn deps(&self, resource_unit_id: &ResourceUnitId) -> HashSet<ResourceUnitId> {
    self
      .resource_pot_group_map
      .values()
      .filter_map(|resource_unit| {
        if resource_unit.groups.contains(resource_unit_id) {
          return Some(resource_unit.resource_unit.id.clone());
        }

        None
      })
      .collect()
  }

  pub fn to_resources(self) -> Vec<ResourcePot> {
    self
      .resource_pot_group_map
      .into_values()
      .map(|group| group.resource_unit.into())
      .collect()
  }

  pub fn resource_pot(&self, resource_unit_id: &ResourceUnitId) -> Option<&ResourceUnit> {
    self
      .resource_pot_group_map
      .get(resource_unit_id)
      .as_ref()
      .map(|group| &group.resource_unit)
  }

  pub fn resource_pot_mut(
    &mut self,
    resource_unit_id: &ResourceUnitId,
  ) -> Option<&mut ResourceUnit> {
    self
      .resource_pot_group_map
      .get_mut(resource_unit_id)
      .map(|group| &mut group.resource_unit)
  }

  pub fn group_mut(&mut self, resource_unit_id: &ResourceUnitId) -> Option<&mut ResourceUnitGroup> {
    self.resource_pot_group_map.get_mut(resource_unit_id)
  }

  pub fn clean_empty_resources(&mut self) {
    let mut empty_resources = HashSet::new();
    for (key, group) in self.resource_pot_group_map.iter() {
      if group.resource_unit.entry_module.is_some() {
        continue;
      }

      if group.resource_unit.modules().is_empty() {
        empty_resources.insert(key.clone());
      }
    }

    empty_resources.iter().for_each(|resource| {
      self.remove_resource_pot(resource);
    });
  }
}
