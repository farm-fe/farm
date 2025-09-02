//! This hook is only JS plugin hook, it's to be compatible with Vite's `transformIndexHtml` hook.
//! If you are a Farm plugin author, you should not use this hook, use `transform` or `process_module` or other hooks instead, cause farm treat HTML as a basic module.

use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::Result,
  resource::Resource,
  serde::{Deserialize, Serialize},
};
use napi::bindgen_prelude::{Function, JsObjectValue, Object};

use crate::plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook;

pub struct JsPluginTransformHtmlHook {
  tsfn: ThreadSafeJsPluginHook,
  /// pre/post/normal
  pub order: JsPluginTransformHtmlHookOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "farmfe_core::serde", rename_all = "camelCase")]
pub struct JsPluginTransformHtmlHookParams {
  pub html_resource: Resource,
}

#[napi]
#[derive(Debug)]
pub enum JsPluginTransformHtmlHookOrder {
  Pre,
  Normal,
  Post,
}

impl JsPluginTransformHtmlHook {
  pub fn new(env: &napi::Env, obj: Object) -> Self {
    let func = obj
      .get_named_property::<Function>("executor")
      .expect("executor should be checked in js side");
    let order = obj
      .get_named_property::<JsPluginTransformHtmlHookOrder>("order")
      .unwrap_or(JsPluginTransformHtmlHookOrder::Normal);

    Self {
      tsfn: ThreadSafeJsPluginHook::new::<JsPluginTransformHtmlHookParams, Resource>(env, func),
      order,
    }
  }

  pub fn call(
    &self,
    param: JsPluginTransformHtmlHookParams,
    ctx: Arc<CompilationContext>,
  ) -> Result<Option<Resource>> {
    self.tsfn.call(param, ctx, None)
  }
}
