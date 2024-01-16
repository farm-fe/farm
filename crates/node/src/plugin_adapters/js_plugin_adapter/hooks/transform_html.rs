//! This hook is only JS plugin hook, it's to be compatible with Vite's `transformIndexHtml` hook in dev mode.
//! It's used to transform the HTML content of the `.html` file.

use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::Result,
  resource::Resource,
  serde::{Deserialize, Serialize},
};

use crate::plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook;

pub struct JsPluginTransformHtmlHook {
  tsfn: ThreadSafeJsPluginHook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "farmfe_core::serde", rename_all = "camelCase")]
pub struct JsPluginTransformHtmlHookParams {
  pub html_resource: Resource,
}

impl JsPluginTransformHtmlHook {
  pub fn new(env: &napi::Env, obj: napi::JsObject) -> Self {
    let func = obj
      .get_named_property::<napi::JsFunction>("executor")
      .expect("executor should be checked in js side");

    Self {
      tsfn: ThreadSafeJsPluginHook::new::<JsPluginTransformHtmlHookParams, Resource>(env, func),
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
