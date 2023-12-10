use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::Result,
  plugin::{PluginRenderResourcePotHookParam, PluginRenderResourcePotHookResult},
};

use crate::plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook;

pub struct JsPluginRenderResourcePotHook {
  tsfn: ThreadSafeJsPluginHook,
}

impl JsPluginRenderResourcePotHook {
  pub fn new(env: &napi::Env, obj: napi::JsObject) -> Self {
    let func = obj
      .get_named_property::<napi::JsFunction>("executor")
      .expect("executor should be checked in js side");

    Self {
      tsfn: ThreadSafeJsPluginHook::new::<
        PluginRenderResourcePotHookParam,
        PluginRenderResourcePotHookResult,
      >(env, func),
    }
  }

  pub fn call(
    &self,
    param: PluginRenderResourcePotHookParam,
    ctx: Arc<CompilationContext>,
  ) -> Result<Option<PluginRenderResourcePotHookResult>> {
    self.tsfn.call(param, ctx, None)
  }
}
