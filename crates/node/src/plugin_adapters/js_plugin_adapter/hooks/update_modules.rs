use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext, plugin::hooks::update_modules::PluginUpdateModulesHookParam,
};
use napi::bindgen_prelude::{Function, JsObjectValue, Object};

use crate::plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook;

pub struct JsPluginUpdateModulesHook {
  tsfn: ThreadSafeJsPluginHook,
}

impl JsPluginUpdateModulesHook {
  pub fn new(env: &napi::Env, obj: Object) -> Self {
    let func = obj
      .get_named_property::<Function>("executor")
      .expect("executor should be checked in js side");

    Self {
      tsfn: ThreadSafeJsPluginHook::new::<PluginUpdateModulesHookParam, Vec<(String, String)>>(
        env, func,
      ),
    }
  }

  pub fn call(
    &self,
    param: PluginUpdateModulesHookParam,
    ctx: Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<Vec<(String, String)>>> {
    self
      .tsfn
      .call::<PluginUpdateModulesHookParam, Vec<(String, String)>>(param, ctx, None)
  }
}
