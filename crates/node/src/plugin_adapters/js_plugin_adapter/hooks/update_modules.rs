use std::sync::Arc;

use farmfe_core::{context::CompilationContext, plugin::PluginUpdateModulesHookParams};

use crate::plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook;

pub struct JsPluginUpdateModulesHook {
  tsfn: ThreadSafeJsPluginHook,
}

impl JsPluginUpdateModulesHook {
  pub fn new(env: &napi::Env, obj: napi::JsObject) -> Self {
    let func = obj
      .get_named_property::<napi::JsFunction>("executor")
      .expect("executor should be checked in js side");

    Self {
      tsfn: ThreadSafeJsPluginHook::new::<PluginUpdateModulesHookParams, Vec<String>>(env, func),
    }
  }

  pub fn call(
    &self,
    param: PluginUpdateModulesHookParams,
    ctx: Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<Vec<String>>> {
    self
      .tsfn
      .call::<PluginUpdateModulesHookParams, Vec<String>>(param, ctx, None)
  }
}
