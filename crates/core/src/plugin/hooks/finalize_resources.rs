use crate::{config::Config, resource::Resource, HashMap};

pub struct PluginFinalizeResourcesHookParam<'a> {
  pub resources_map: &'a mut HashMap<String, Resource>,
  pub config: &'a Config,
}
