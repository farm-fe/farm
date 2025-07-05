use std::sync::Arc;

use crate::{
  new_js_plugin_hook,
  plugin_adapters::js_plugin_adapter::{
    hooks::process_rendered_resource_pot::JsResourcePot,
    thread_safe_js_plugin_hook::ThreadSafeJsPluginHook,
  },
};
use farmfe_core::{
  config::config_regex::ConfigRegex, context::CompilationContext, error::Result,
  resource::resource_pot::ResourcePotType,
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
    JsResourcePot,
    String
  );

  pub fn call(&self, param: JsResourcePot, ctx: Arc<CompilationContext>) -> Result<Option<String>> {
    if self
      .filters
      .resource_pot_types
      .contains(&param.resource_pot_type)
      || self.filters.module_ids.iter().any(|f| {
        param
          .module_ids
          .iter()
          .any(|id| f.is_match(&id.to_string()))
      })
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
  pub module_ids: Vec<String>,
}

#[derive(Debug)]
pub struct PluginAugmentResourceHashHookFilters {
  pub resource_pot_types: Vec<ResourcePotType>,
  pub module_ids: Vec<ConfigRegex>,
}

impl From<JsPluginAugmentResourceHashHookFilters> for PluginAugmentResourceHashHookFilters {
  fn from(f: JsPluginAugmentResourceHashHookFilters) -> Self {
    Self {
      resource_pot_types: f
        .resource_pot_types
        .into_iter()
        .map(|ty| ty.into())
        .collect(),
      module_ids: f
        .module_ids
        .into_iter()
        .map(|p| (ConfigRegex::new(&p)))
        .collect(),
    }
  }
}
