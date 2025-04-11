use crate::{
  module::{module_graph::ModuleGraph, module_group::ModuleGroupGraph, ModuleId},
  resource::{resource_pot_map::ResourcePotMap, Resource, ResourceType},
  HashMap,
};

pub struct PluginHandleEntryResourceHookParam<'a> {
  pub resource: Resource,
  pub resource_sourcemap: Option<Resource>,

  pub module_graph: &'a ModuleGraph,
  pub module_group_graph: &'a ModuleGroupGraph,
  pub resource_pot_map: &'a ResourcePotMap,
  pub entry_module_id: &'a ModuleId,

  /// Initial resources including entry resource
  pub initial_resources: Vec<(String, ResourceType)>,
  pub dynamic_resources: String,
  pub dynamic_module_resources_map: String,

  pub runtime_code: &'a str,
  pub runtime_resource_name: &'a str,
  /// Set it to true if runtime needs to be emitted as a separate
  pub emit_runtime: bool,

  pub additional_inject_resources: HashMap<String, Resource>,
}
