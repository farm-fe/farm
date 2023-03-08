use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  plugin::{
    Plugin, PluginHookContext, PluginLoadHookParam, PluginLoadHookResult, PluginResolveHookParam,
    PluginResolveHookResult, PluginTransformHookParam, PluginTransformHookResult, DEFAULT_PRIORITY,
  },
};
use napi::{bindgen_prelude::FromNapiValue, Env, JsObject, JsUnknown, NapiRaw};

use self::thread_safe_js_plugin_hook::{
  JsPluginLoadHook, JsPluginResolveHook, JsPluginTransformHook,
};

pub mod context;
mod thread_safe_js_plugin_hook;

pub struct JsPluginAdapter {
  name: String,
  priority: i32,
  js_resolve_hook: Option<JsPluginResolveHook>,
  js_load_hook: Option<JsPluginLoadHook>,
  js_transform_hook: Option<JsPluginTransformHook>,
}

impl JsPluginAdapter {
  pub fn new(env: &Env, js_plugin_object: JsObject) -> Result<Self> {
    let name = get_named_property(env, &js_plugin_object, "name")?;
    let priority =
      get_named_property::<i32>(env, &js_plugin_object, "priority").unwrap_or(DEFAULT_PRIORITY);

    let resolve_hook_obj = get_named_property::<JsObject>(env, &js_plugin_object, "resolve").ok();
    let load_hook_obj = get_named_property::<JsObject>(env, &js_plugin_object, "load").ok();
    let transform_hook_obj =
      get_named_property::<JsObject>(env, &js_plugin_object, "transform").ok();

    // TODO calculating hooks should execute
    Ok(Self {
      name,
      priority,
      js_resolve_hook: resolve_hook_obj.map(|obj| JsPluginResolveHook::new(env, obj)),
      js_load_hook: load_hook_obj.map(|obj| JsPluginLoadHook::new(env, obj)),
      js_transform_hook: transform_hook_obj.map(|obj| JsPluginTransformHook::new(env, obj)),
    })
  }
}

impl Plugin for JsPluginAdapter {
  fn name(&self) -> &str {
    &self.name
  }

  fn priority(&self) -> i32 {
    self.priority
  }

  fn resolve(
    &self,
    param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> Result<Option<PluginResolveHookResult>> {
    if let Some(js_resolve_hook) = &self.js_resolve_hook {
      let cp = param.clone();
      js_resolve_hook.call(cp, context.clone(), hook_context.clone())
    } else {
      Ok(None)
    }
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    if let Some(js_load_hook) = &self.js_load_hook {
      let cp = param.clone();
      js_load_hook.call(cp, context.clone(), hook_context.clone())
    } else {
      Ok(None)
    }
  }

  fn transform(
    &self,
    param: &PluginTransformHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<PluginTransformHookResult>> {
    if let Some(js_transform_hook) = &self.js_transform_hook {
      let cp = param.clone();
      js_transform_hook.call(cp, context.clone())
    } else {
      Ok(None)
    }
  }
}

pub fn get_named_property<T: FromNapiValue>(env: &Env, obj: &JsObject, field: &str) -> Result<T> {
  if obj.has_named_property(field).map_err(|e| {
    CompilationError::NAPIError(format!(
      "Get field {} of config object failed. {:?}",
      field, e
    ))
  })? {
    unsafe {
      T::from_napi_value(
        env.raw(),
        obj
          .get_named_property::<JsUnknown>(field)
          .map_err(|e| {
            CompilationError::NAPIError(format!(
              "Get field {} of config object failed. {:?}",
              field, e
            ))
          })?
          .raw(),
      )
      .map_err(|e| {
        CompilationError::NAPIError(format!("Transform config field {} failed. {:?}", field, e))
      })
    }
  } else {
    Err(CompilationError::NAPIError(format!(
      "Invalid Config: the config object does not have field {}",
      field
    )))
  }
}
