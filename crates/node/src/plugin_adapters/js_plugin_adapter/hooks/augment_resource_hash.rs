use std::sync::Arc;

use farmfe_core::{context::CompilationContext, error::Result, plugin::ChunkResourceInfo};

use crate::plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook;

pub struct JsPluginAugmentResourceHashHook {
  tsfn: ThreadSafeJsPluginHook,
}

impl JsPluginAugmentResourceHashHook {
  pub fn new(env: &napi::Env, obj: napi::JsObject) -> Self {
    let func = obj
      .get_named_property::<napi::JsFunction>("executor")
      .expect("executor should be checked in js side");

    Self {
      tsfn: ThreadSafeJsPluginHook::new::<ChunkResourceInfo, String>(env, func),
    }
  }

  pub fn call(
    &self,
    param: ChunkResourceInfo,
    ctx: Arc<CompilationContext>,
  ) -> Result<Option<String>> {
    self.tsfn.call(param, ctx, None)
  }
}
