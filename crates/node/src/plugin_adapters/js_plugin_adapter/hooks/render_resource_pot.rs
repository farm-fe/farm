use std::sync::Arc;

use farmfe_core::{
  config::config_regex::ConfigRegex,
  context::CompilationContext,
  error::Result,
  plugin::{PluginRenderResourcePotHookParam, PluginRenderResourcePotHookResult},
  resource::resource_pot::ResourcePotType,
};
use napi::{bindgen_prelude::FromNapiValue, NapiRaw};

use crate::{
  new_js_plugin_hook,
  plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook,
};

#[napi(object)]
pub struct JsPluginRenderResourcePotHookFilters {
  pub resource_pot_types: Vec<String>,
  pub module_ids: Vec<String>,
}

#[derive(Debug)]
pub struct PluginRenderResourcePotHookFilters {
  pub resource_pot_types: Vec<ResourcePotType>,
  pub module_ids: Vec<ConfigRegex>,
}

pub struct JsPluginRenderResourcePotHook {
  tsfn: ThreadSafeJsPluginHook,
  filters: PluginRenderResourcePotHookFilters,
}

impl From<JsPluginRenderResourcePotHookFilters> for PluginRenderResourcePotHookFilters {
  fn from(f: JsPluginRenderResourcePotHookFilters) -> Self {
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

impl JsPluginRenderResourcePotHook {
  new_js_plugin_hook!(
    PluginRenderResourcePotHookFilters,
    JsPluginRenderResourcePotHookFilters,
    PluginRenderResourcePotHookParam,
    PluginRenderResourcePotHookResult
  );

  pub fn call(
    &self,
    param: PluginRenderResourcePotHookParam,
    ctx: Arc<CompilationContext>,
  ) -> Result<Option<PluginRenderResourcePotHookResult>> {
    if self
      .filters
      .resource_pot_types
      .contains(&param.resource_pot_info.resource_pot_type)
      || self.filters.module_ids.iter().any(|f| {
        param
          .resource_pot_info
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
