use std::{collections::HashMap, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  error::Result,
  plugin::PluginFinalizeResourcesHookParams,
  resource::Resource,
  serde::{Deserialize, Serialize},
};

use crate::plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook;

pub struct JsPluginFinalizeResourcesHook {
  tsfn: ThreadSafeJsPluginHook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "farmfe_core::serde", rename_all = "camelCase")]
pub struct JsPluginFinalizeResourcesHookParams {
  pub resources_map: HashMap<String, Resource>,
  pub config: farmfe_core::config::Config,
}

impl From<&mut PluginFinalizeResourcesHookParams<'_>> for JsPluginFinalizeResourcesHookParams {
  fn from(value: &mut PluginFinalizeResourcesHookParams) -> Self {
    Self {
      resources_map: value.resources_map.clone(),
      config: value.config.clone(),
    }
  }
}

impl JsPluginFinalizeResourcesHook {
  pub fn new(env: &napi::Env, obj: napi::JsObject) -> Self {
    let func = obj
      .get_named_property::<napi::JsFunction>("executor")
      .expect("executor should be checked in js side");

    Self {
      tsfn: ThreadSafeJsPluginHook::new::<
        JsPluginFinalizeResourcesHookParams,
        HashMap<String, Resource>,
      >(env, func),
    }
  }

  pub fn call(
    &self,
    param: JsPluginFinalizeResourcesHookParams,
    ctx: Arc<CompilationContext>,
  ) -> Result<Option<HashMap<String, Resource>>> {
    self.tsfn.call(param, ctx, None)
  }
}
