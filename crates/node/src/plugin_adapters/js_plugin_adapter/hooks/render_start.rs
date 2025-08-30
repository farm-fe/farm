use std::sync::Arc;

use farmfe_core::{config::Config, context::CompilationContext, error::Result};
use napi::bindgen_prelude::{Function, JsObjectValue, Object};

use crate::plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook;

pub struct JsPluginRenderStartHook {
  tsfn: ThreadSafeJsPluginHook,
}

impl JsPluginRenderStartHook {
  pub fn new(env: &napi::Env, obj: Object) -> Self {
    let func = obj
      .get_named_property::<Function>("executor")
      .expect("executor should be checked in js side");

    Self {
      tsfn: ThreadSafeJsPluginHook::new::<Config, Config>(env, func),
    }
  }

  pub fn call(&self, param: Config, ctx: Arc<CompilationContext>) -> Result<Option<Config>> {
    self.tsfn.call(param, ctx, None)
  }
}
