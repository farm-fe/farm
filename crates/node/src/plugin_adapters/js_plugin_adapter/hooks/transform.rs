use farmfe_core::{config::config_regex::ConfigRegex, module::ModuleType};
use farmfe_core::{
  context::CompilationContext,
  error::Result,
  module::ModuleId,
  plugin::{PluginTransformHookParam, PluginTransformHookResult},
};
use napi::{bindgen_prelude::FromNapiValue, NapiRaw};
use std::sync::Arc;

use crate::{
  new_js_plugin_hook,
  plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook,
};

pub struct JsPluginTransformHook {
  tsfn: ThreadSafeJsPluginHook,
  filters: PluginTransformHookFilters,
}

impl JsPluginTransformHook {
  new_js_plugin_hook!(
    PluginTransformHookFilters,
    JsPluginTransformHookFilters,
    PluginTransformHookParam,
    PluginTransformHookResult
  );

  pub fn call(
    &self,
    param: PluginTransformHookParam,
    ctx: Arc<CompilationContext>,
  ) -> Result<Option<PluginTransformHookResult>> {
    if self.filters.resolved_paths.iter().any(|f| {
      f.is_match(
        &ModuleId::from(param.module_id.as_str()).resolved_path_with_query(&ctx.config.root),
      )
    }) || self
      .filters
      .module_types
      .iter()
      .any(|ty| &param.module_type == ty)
    {
      self
        .tsfn
        .call::<PluginTransformHookParam, PluginTransformHookResult>(param, ctx, None)
    } else {
      Ok(None)
    }
  }
}

#[napi(object)]
pub struct JsPluginTransformHookFilters {
  pub resolved_paths: Vec<String>,
  pub module_types: Vec<String>,
}

#[derive(Debug)]
pub struct PluginTransformHookFilters {
  pub resolved_paths: Vec<ConfigRegex>,
  pub module_types: Vec<ModuleType>,
}

impl From<JsPluginTransformHookFilters> for PluginTransformHookFilters {
  fn from(f: JsPluginTransformHookFilters) -> Self {
    Self {
      resolved_paths: f
        .resolved_paths
        .into_iter()
        .map(|f| ConfigRegex::new(&f))
        .collect(),
      module_types: f.module_types.into_iter().map(|ty| ty.into()).collect(),
    }
  }
}
