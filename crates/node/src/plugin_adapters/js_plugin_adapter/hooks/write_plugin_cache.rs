use std::sync::Arc;

use farmfe_core::{context::CompilationContext, plugin::EmptyPluginHookParam};
use napi::bindgen_prelude::{Function, JsObjectValue, Object};

use crate::plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook;

pub struct JsPluginWritePluginCacheHook {
  tsfn: ThreadSafeJsPluginHook,
}

impl JsPluginWritePluginCacheHook {
  pub fn new(env: &napi::Env, obj: Object) -> Self {
    let func = obj
      .get_named_property::<Function>("executor")
      .expect("executor should be checked in js side");

    Self {
      tsfn: ThreadSafeJsPluginHook::new::<EmptyPluginHookParam, Vec<u8>>(env, func),
    }
  }

  pub fn call(&self, ctx: Arc<CompilationContext>) -> farmfe_core::error::Result<Option<Vec<u8>>> {
    self
      .tsfn
      .call::<EmptyPluginHookParam, Vec<u8>>(EmptyPluginHookParam {}, ctx, None)
  }
}
