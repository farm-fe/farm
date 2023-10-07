use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  hashbrown::HashSet,
  plugin::{
    EmptyPluginHookParam, Plugin, PluginHookContext, PluginLoadHookParam, PluginLoadHookResult,
    PluginResolveHookParam, PluginResolveHookResult, PluginTransformHookParam,
    PluginTransformHookResult, UpdateType, DEFAULT_PRIORITY,
  },
};
use napi::{bindgen_prelude::FromNapiValue, Env, JsObject, JsUnknown, NapiRaw};

use self::thread_safe_js_plugin_hook::{
  JsPluginBuildEndHook, JsPluginBuildStartHook, JsPluginFinishHook, JsPluginLoadHook,
  JsPluginResolveHook, JsPluginTransformHook, JsPluginUpdateModulesHook,
};

pub mod context;
mod thread_safe_js_plugin_hook;

pub struct JsPluginAdapter {
  name: String,
  priority: i32,
  js_build_start_hook: Option<JsPluginBuildStartHook>,
  js_resolve_hook: Option<JsPluginResolveHook>,
  js_load_hook: Option<JsPluginLoadHook>,
  js_transform_hook: Option<JsPluginTransformHook>,
  js_build_end_hook: Option<JsPluginBuildEndHook>,
  js_finish_hook: Option<JsPluginFinishHook>,
  js_update_modules_hook: Option<JsPluginUpdateModulesHook>,
}

impl JsPluginAdapter {
  pub fn new(env: &Env, js_plugin_object: JsObject) -> Result<Self> {
    let name = get_named_property(env, &js_plugin_object, "name")?;
    let priority =
      get_named_property::<i32>(env, &js_plugin_object, "priority").unwrap_or(DEFAULT_PRIORITY);

    let build_start_hook_obj =
      get_named_property::<JsObject>(env, &js_plugin_object, "buildStart").ok();
    let resolve_hook_obj = get_named_property::<JsObject>(env, &js_plugin_object, "resolve").ok();
    let load_hook_obj = get_named_property::<JsObject>(env, &js_plugin_object, "load").ok();
    let transform_hook_obj =
      get_named_property::<JsObject>(env, &js_plugin_object, "transform").ok();
    let build_end_hook_obj =
      get_named_property::<JsObject>(env, &js_plugin_object, "buildEnd").ok();
    let finish_hook_obj = get_named_property::<JsObject>(env, &js_plugin_object, "finish").ok();
    let update_modules_hook_obj =
      get_named_property::<JsObject>(env, &js_plugin_object, "updateModules").ok();

    Ok(Self {
      name,
      priority,
      js_build_start_hook: build_start_hook_obj.map(|obj| JsPluginBuildStartHook::new(env, obj)),
      js_resolve_hook: resolve_hook_obj.map(|obj| JsPluginResolveHook::new(env, obj)),
      js_load_hook: load_hook_obj.map(|obj| JsPluginLoadHook::new(env, obj)),
      js_transform_hook: transform_hook_obj.map(|obj| JsPluginTransformHook::new(env, obj)),
      js_build_end_hook: build_end_hook_obj.map(|obj| JsPluginBuildEndHook::new(env, obj)),
      js_finish_hook: finish_hook_obj.map(|obj| JsPluginFinishHook::new(env, obj)),
      js_update_modules_hook: update_modules_hook_obj
        .map(|obj| JsPluginUpdateModulesHook::new(env, obj)),
    })
  }
}

impl Plugin for JsPluginAdapter {
  fn name(&self) -> &str {
    &self.name
  }

  fn build_start(&self, context: &Arc<CompilationContext>) -> Result<Option<()>> {
    if let Some(js_build_start_hook) = &self.js_build_start_hook {
      js_build_start_hook.call(EmptyPluginHookParam {}, context.clone())?;
      Ok(Some(()))
    } else {
      Ok(None)
    }
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

  fn build_end(&self, context: &Arc<CompilationContext>) -> Result<Option<()>> {
    if let Some(js_build_end_hook) = &self.js_build_end_hook {
      js_build_end_hook.call(EmptyPluginHookParam {}, context.clone())?;
      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  fn update_modules(
    &self,
    params: &mut farmfe_core::plugin::PluginUpdateModulesHookParams,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if let Some(js_update_modules_hook) = &self.js_update_modules_hook {
      let update_result = js_update_modules_hook.call(params.clone(), context.clone())?;
      let mut updating_modules = params
        .paths
        .iter()
        .map(|p| p.0.to_string())
        .collect::<HashSet<_>>();

      if let Some(result) = update_result {
        for item in result {
          if !updating_modules.contains(&item) {
            params.paths.push((item.clone(), UpdateType::Updated));
            updating_modules.insert(item);
          }
        }
      }

      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  fn finish(
    &self,
    _stat: &farmfe_core::stats::Stats,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if let Some(js_finish_hook) = &self.js_finish_hook {
      js_finish_hook.call(EmptyPluginHookParam {}, context.clone())?;
      Ok(Some(()))
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
