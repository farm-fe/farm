use std::sync::Arc;

use crate::{
  new_js_plugin_hook,
  plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook,
};
use farmfe_core::{
  config::config_regex::ConfigRegex, context::CompilationContext, error::Result,
  plugin::ChunkResourceInfo, resource::resource_pot::ResourcePotType,
};
use napi::{bindgen_prelude::FromNapiValue, NapiRaw};

pub struct JsPluginAugmentResourceHashHook {
  tsfn: ThreadSafeJsPluginHook,
  filters: PluginAugmentResourceHashHookFilters,
}

impl JsPluginAugmentResourceHashHook {
  new_js_plugin_hook!(
    PluginAugmentResourceHashHookFilters,
    JsPluginAugmentResourceHashHookFilters,
    ChunkResourceInfo,
    String
  );

  pub fn call(
    &self,
    param: ChunkResourceInfo,
    ctx: Arc<CompilationContext>,
  ) -> Result<Option<String>> {
    if self
      .filters
      .resource_pot_types
      .contains(&param.resource_pot_type)
      && self
        .filters
        .paths
        .iter()
        .any(|p| param.module_ids.iter().any(|m| p.is_match(&m.to_string())))
    {
      self.tsfn.call(param, ctx, None)
    } else {
      Ok(None)
    }
  }
}

#[napi(object)]
pub struct JsPluginAugmentResourceHashHookFilters {
  pub resource_pot_types: Vec<String>,
  pub paths: Vec<String>,
}

#[derive(Debug)]
pub struct PluginAugmentResourceHashHookFilters {
  pub resource_pot_types: Vec<ResourcePotType>,
  pub paths: Vec<ConfigRegex>,
}

impl From<JsPluginAugmentResourceHashHookFilters> for PluginAugmentResourceHashHookFilters {
  fn from(f: JsPluginAugmentResourceHashHookFilters) -> Self {
    Self {
      resource_pot_types: f
        .resource_pot_types
        .into_iter()
        .map(|ty| ty.into())
        .collect(),
      paths: f
        .paths
        .into_iter()
        .map(|p| (ConfigRegex::new(&p)))
        .collect(),
    }
  }
}
