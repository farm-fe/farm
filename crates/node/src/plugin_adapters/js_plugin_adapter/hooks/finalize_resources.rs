use std::{collections::HashMap, sync::Arc};

use farmfe_core::{context::CompilationContext, error::Result, resource::Resource};

use crate::plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook;

pub struct JsPluginFinalizeResourcesHook {
  tsfn: ThreadSafeJsPluginHook,
}

pub type PluginFinalizeResourcesHookValue = HashMap<String, Resource>;

impl JsPluginFinalizeResourcesHook {
  pub fn new(env: &napi::Env, obj: napi::JsObject) -> Self {
    let func = obj
      .get_named_property::<napi::JsFunction>("executor")
      .expect("executor should be checked in js side");

    Self {
      tsfn: ThreadSafeJsPluginHook::new::<
        PluginFinalizeResourcesHookValue,
        PluginFinalizeResourcesHookValue,
      >(env, func),
    }
  }

  pub fn call(
    &self,
    param: PluginFinalizeResourcesHookValue,
    ctx: Arc<CompilationContext>,
  ) -> Result<Option<PluginFinalizeResourcesHookValue>> {
    self.tsfn.call(param, ctx, None)
  }
}
