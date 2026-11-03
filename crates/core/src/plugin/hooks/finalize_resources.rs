use crate::{HashMap, config::Config, resource::Resource};

pub struct PluginFinalizeResourcesHookParam<'a> {
  pub resources_map: &'a mut HashMap<String, Resource>,
  pub config: &'a Config,
}
