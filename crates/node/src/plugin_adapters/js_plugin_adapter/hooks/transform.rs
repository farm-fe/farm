use farmfe_core::{
  context::CompilationContext,
  error::Result,
  module::ModuleId,
  plugin::{PluginTransformHookParam, PluginTransformHookResult},
};
use napi::bindgen_prelude::FromNapiValue;
use std::sync::Arc;

use crate::plugin_adapters::js_plugin_adapter::module_hook_common::{
  JsModuleHookFilters, ModuleHookFilters,
};
use crate::{
  new_js_plugin_hook,
  plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook,
};

pub struct JsPluginTransformHook {
  tsfn: ThreadSafeJsPluginHook,
  pub(crate) filters: ModuleHookFilters,
}

impl JsPluginTransformHook {
  new_js_plugin_hook!(
    ModuleHookFilters,
    JsModuleHookFilters,
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
