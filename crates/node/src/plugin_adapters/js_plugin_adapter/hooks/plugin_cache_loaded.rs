use std::sync::Arc;

use farmfe_core::{context::CompilationContext, plugin::EmptyPluginHookResult};
use napi::bindgen_prelude::{Function, JsObjectValue, Object};

use crate::plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook;

pub struct JsPluginPluginCacheLoadedHook {
  tsfn: ThreadSafeJsPluginHook,
}

impl JsPluginPluginCacheLoadedHook {
  pub fn new(env: &napi::Env, obj: Object) -> Self {
    let func = obj
      .get_named_property::<Function>("executor")
      .expect("executor should be checked in js side");

    Self {
      tsfn: ThreadSafeJsPluginHook::new::<&Vec<u8>, EmptyPluginHookResult>(env, func),
    }
  }

  pub fn call(
    &self,
    param: &Vec<u8>,
    ctx: Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<EmptyPluginHookResult>> {
    self
      .tsfn
      .call::<&Vec<u8>, EmptyPluginHookResult>(param, ctx, None)
  }
}
