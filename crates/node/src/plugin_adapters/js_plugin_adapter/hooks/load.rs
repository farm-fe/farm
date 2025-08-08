use farmfe_core::{
  config::config_regex::ConfigRegex,
  context::CompilationContext,
  error::Result,
  plugin::{PluginHookContext, PluginLoadHookParam, PluginLoadHookResult},
};
use napi::bindgen_prelude::FromNapiValue;
use std::sync::Arc;

use crate::{
  new_js_plugin_hook,
  plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook,
};

pub struct JsPluginLoadHook {
  tsfn: ThreadSafeJsPluginHook,
  filters: PluginLoadHookFilters,
}

impl JsPluginLoadHook {
  new_js_plugin_hook!(
    PluginLoadHookFilters,
    JsPluginLoadHookFilters,
    PluginLoadHookParam,
    PluginLoadHookResult
  );

  pub fn call(
    &self,
    param: PluginLoadHookParam,
    ctx: Arc<CompilationContext>,
    hook_context: PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    if self
      .filters
      .resolved_paths
      .iter()
      .any(|f| f.is_match(param.module_id.as_str()))
    {
      self.tsfn.call::<PluginLoadHookParam, PluginLoadHookResult>(
        param.clone(),
        ctx,
        Some(hook_context),
      )
    } else {
      Ok(None)
    }
  }
}

#[napi(object)]
pub struct JsPluginLoadHookFilters {
  pub resolved_paths: Vec<String>,
}

#[derive(Debug)]
pub struct PluginLoadHookFilters {
  pub resolved_paths: Vec<ConfigRegex>,
}

impl From<JsPluginLoadHookFilters> for PluginLoadHookFilters {
  fn from(f: JsPluginLoadHookFilters) -> Self {
    Self {
      resolved_paths: f
        .resolved_paths
        .into_iter()
        .map(|f| ConfigRegex::new(&f))
        .collect(),
    }
  }
}
