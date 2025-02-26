use std::sync::Arc;

use farmfe_compiler::{DYNAMIC_VIRTUAL_SUFFIX, FARM_CSS_MODULES_SUFFIX, RUNTIME_INPUT_SCOPE};
use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::ModuleType,
  plugin::{
    EmptyPluginHookParam, Plugin, PluginFinalizeResourcesHookParam, PluginHookContext,
    PluginLoadHookParam, PluginLoadHookResult, PluginResolveHookParam, PluginResolveHookResult,
    PluginTransformHookParam, PluginTransformHookResult, UpdateType, DEFAULT_PRIORITY,
  },
  resource::{Resource, ResourceOrigin, ResourceType},
  HashSet,
};
use napi::{bindgen_prelude::FromNapiValue, Env, JsObject, JsUnknown, NapiRaw};

use self::hooks::{
  // augment_resource_hash::JsPluginAugmentResourceHashHook,
  build_end::JsPluginBuildEndHook,
  build_start::JsPluginBuildStartHook,
  finalize_resources::JsPluginFinalizeResourcesHook,
  finish::JsPluginFinishHook,
  load::JsPluginLoadHook,
  plugin_cache_loaded::JsPluginPluginCacheLoadedHook,
  // render_resource_pot::JsPluginRenderResourcePotHook,
  process_module::JsPluginProcessModuleHook,
  freeze_module::JsPluginFreezeModuleHook,
  render_start::JsPluginRenderStartHook,
  resolve::JsPluginResolveHook,
  transform::JsPluginTransformHook,
  transform_html::{
    JsPluginTransformHtmlHook, JsPluginTransformHtmlHookOrder, JsPluginTransformHtmlHookParams,
  },
  update_finished::JsPluginUpdateFinishedHook,
  update_modules::JsPluginUpdateModulesHook,
  write_plugin_cache::JsPluginWritePluginCacheHook,
};

pub mod context;
mod context_methods;
mod hooks;
mod module_hook_common;
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
  js_plugin_cache_loaded: Option<JsPluginPluginCacheLoadedHook>,
  js_write_plugin_cache: Option<JsPluginWritePluginCacheHook>,
  // js_render_resource_pot_hook: Option<JsPluginRenderResourcePotHook>,
  js_render_start_hook: Option<JsPluginRenderStartHook>,
  // js_augment_resource_hash_hook: Option<JsPluginAugmentResourceHashHook>,
  js_finalize_resources_hook: Option<JsPluginFinalizeResourcesHook>,
  js_transform_html_hook: Option<JsPluginTransformHtmlHook>,
  js_update_finished_hook: Option<JsPluginUpdateFinishedHook>,
  js_process_module_hook: Option<JsPluginProcessModuleHook>,
  js_freeze_module_hook: Option<JsPluginFreezeModuleHook>,
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
    let plugin_cache_loaded_obj =
      get_named_property::<JsObject>(env, &js_plugin_object, "pluginCacheLoaded").ok();
    let write_plugin_cache_obj =
      get_named_property::<JsObject>(env, &js_plugin_object, "writePluginCache").ok();
    let render_resource_pot_obj =
      get_named_property::<JsObject>(env, &js_plugin_object, "renderResourcePot").ok();
    let render_start_obj =
      get_named_property::<JsObject>(env, &js_plugin_object, "renderStart").ok();
    let augment_resource_hash_obj =
      get_named_property::<JsObject>(env, &js_plugin_object, "augmentResourceHash").ok();
    let finalize_resources_obj =
      get_named_property::<JsObject>(env, &js_plugin_object, "finalizeResources").ok();
    let transform_html_obj =
      get_named_property::<JsObject>(env, &js_plugin_object, "transformHtml").ok();
    let update_finished_obj =
      get_named_property::<JsObject>(env, &js_plugin_object, "updateFinished").ok();
    let process_module_obj =
      get_named_property::<JsObject>(env, &js_plugin_object, "processModule").ok();
    let freeze_module_obj =
      get_named_property::<JsObject>(env, &js_plugin_object, "freezeModule").ok();

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
      js_plugin_cache_loaded: plugin_cache_loaded_obj
        .map(|obj| JsPluginPluginCacheLoadedHook::new(env, obj)),
      js_write_plugin_cache: write_plugin_cache_obj
        .map(|obj| JsPluginWritePluginCacheHook::new(env, obj)),
      // js_render_resource_pot_hook: render_resource_pot_obj
      //   .map(|obj| JsPluginRenderResourcePotHook::new(env, obj)),
      js_render_start_hook: render_start_obj.map(|obj| JsPluginRenderStartHook::new(env, obj)),
      // js_augment_resource_hash_hook: augment_resource_hash_obj
      //   .map(|obj| JsPluginAugmentResourceHashHook::new(env, obj)),
      js_finalize_resources_hook: finalize_resources_obj
        .map(|obj| JsPluginFinalizeResourcesHook::new(env, obj)),
      js_transform_html_hook: transform_html_obj
        .map(|obj| JsPluginTransformHtmlHook::new(env, obj)),
      js_update_finished_hook: update_finished_obj
        .map(|obj| JsPluginUpdateFinishedHook::new(env, obj)),
      js_process_module_hook: process_module_obj
        .map(|obj| JsPluginProcessModuleHook::new(env, obj)),
      js_freeze_module_hook: freeze_module_obj
        .map(|obj| JsPluginFreezeModuleHook::new(env, obj)),
    })
  }

  pub fn is_internal_virtual_module(&self, path: &str) -> bool {
    path.ends_with(DYNAMIC_VIRTUAL_SUFFIX)
      || FARM_CSS_MODULES_SUFFIX.is_match(path)
      || path.ends_with(RUNTIME_INPUT_SCOPE)
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
    if self.is_internal_virtual_module(&param.source) {
      return Ok(None);
    }

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
    if self.is_internal_virtual_module(&param.module_id) {
      return Ok(None);
    }

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
    if self.is_internal_virtual_module(&param.module_id) {
      return Ok(None);
    }

    // call pre transformHtml hook first
    let result = if let Some(js_transform_html_hook) = &self.js_transform_html_hook {
      if matches!(param.module_type, ModuleType::Html)
        && matches!(
          js_transform_html_hook.order,
          JsPluginTransformHtmlHookOrder::Pre
        )
      {
        let params = JsPluginTransformHtmlHookParams {
          html_resource: Resource {
            name: param.module_id.clone(),
            bytes: param.content.clone().into_bytes(),
            resource_type: ResourceType::Html,
            origin: ResourceOrigin::Module(param.module_id.as_str().into()),
            ..Default::default()
          },
        };
        let transformed_html_resource = js_transform_html_hook.call(params, context.clone())?;

        if let Some(transformed_html_resource) = transformed_html_resource {
          Ok(Some(
            String::from_utf8(transformed_html_resource.bytes).unwrap(),
          ))
        } else {
          Ok(None)
        }
      } else {
        Ok(None)
      }
    } else {
      Ok(None)
    }?;

    if let Some(js_transform_hook) = &self.js_transform_hook {
      let cloned_param = param.clone();
      let cp = PluginTransformHookParam {
        content: result.unwrap_or(cloned_param.content),
        ..cloned_param
      };
      js_transform_hook.call(cp, context.clone())
    } else if let Some(result) = result {
      Ok(Some(PluginTransformHookResult {
        content: result,
        ..Default::default()
      }))
    } else {
      Ok(None)
    }
  }

  fn process_module(
    &self,
    param: &mut farmfe_core::plugin::PluginProcessModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if let Some(ref js_process_module_hook) = self.js_process_module_hook {
      return js_process_module_hook.call(param, context.clone());
    }

    Ok(None)
  }

  fn freeze_module(
    &self,
    param: &mut farmfe_core::plugin::PluginFreezeModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if let Some(ref js_freeze_module_hook) = self.js_freeze_module_hook {
      return js_freeze_module_hook.call(param, context.clone());
    }

    Ok(None)
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
    params: &mut farmfe_core::plugin::PluginUpdateModulesHookParam,
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

  fn update_finished(&self, context: &Arc<CompilationContext>) -> Result<Option<()>> {
    if let Some(js_update_finished_hook) = &self.js_update_finished_hook {
      js_update_finished_hook.call(EmptyPluginHookParam {}, context.clone())?;
      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  fn plugin_cache_loaded(
    &self,
    cache: &Vec<u8>,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if let Some(js_plugin_cache_loaded_hook) = &self.js_plugin_cache_loaded {
      js_plugin_cache_loaded_hook.call(cache, context.clone())?;
      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  // fn render_resource_pot(
  //   &self,
  //   param: &farmfe_core::plugin::PluginRenderResourcePotHookParam,
  //   context: &Arc<CompilationContext>,
  // ) -> Result<Option<farmfe_core::plugin::PluginRenderResourcePotHookResult>> {
  //   if let Some(js_plugin_render_resource_pot) = &self.js_render_resource_pot_hook {
  //     js_plugin_render_resource_pot.call(param.clone(), context.clone())
  //   } else {
  //     Ok(None)
  //   }
  // }

  fn render_start(
    &self,
    config: &farmfe_core::config::Config,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if let Some(js_render_start_hook) = &self.js_render_start_hook {
      js_render_start_hook.call(config.clone(), context.clone())?;
      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  // fn augment_resource_hash(
  //   &self,
  //   render_pot_info: &farmfe_core::resource::resource_pot::ResourcePotInfo,
  //   context: &Arc<CompilationContext>,
  // ) -> Result<Option<String>> {
  //   if let Some(js_augment_resource_hash_hook) = &self.js_augment_resource_hash_hook {
  //     js_augment_resource_hash_hook.call(render_pot_info.clone(), context.clone())
  //   } else {
  //     Ok(None)
  //   }
  // }

  fn finalize_resources(
    &self,
    params: &mut PluginFinalizeResourcesHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if let Some(js_finalize_resources_hook) = &self.js_finalize_resources_hook {
      if let Some(result) = js_finalize_resources_hook.call(params.into(), context.clone())? {
        params.resources_map.clear();
        params.resources_map.extend(result);
      };
    }

    if let Some(js_transform_html_hook) = &self.js_transform_html_hook {
      for (_, v) in params.resources_map.iter_mut() {
        if matches!(v.resource_type, ResourceType::Html)
          && matches!(
            js_transform_html_hook.order,
            JsPluginTransformHtmlHookOrder::Normal | JsPluginTransformHtmlHookOrder::Post
          )
        {
          let params = JsPluginTransformHtmlHookParams {
            html_resource: v.clone(),
          };
          let transformed_html_resource = js_transform_html_hook.call(params, context.clone())?;

          if let Some(transformed_html_resource) = transformed_html_resource {
            v.bytes = transformed_html_resource.bytes;
          }
        }
      }
    }

    Ok(Some(()))
  }

  fn write_plugin_cache(&self, context: &Arc<CompilationContext>) -> Result<Option<Vec<u8>>> {
    if let Some(js_write_plugin_cache_hook) = &self.js_write_plugin_cache {
      js_write_plugin_cache_hook.call(context.clone())
    } else {
      Ok(None)
    }
  }
}

pub fn get_named_property<T: FromNapiValue>(env: &Env, obj: &JsObject, field: &str) -> Result<T> {
  // TODO: maybe can prompt for the name of the plugin
  if obj.has_named_property(field).map_err(|e| {
    CompilationError::NAPIError(format!("Get field {field} of config object failed. {e:?}"))
  })? {
    unsafe {
      T::from_napi_value(
        env.raw(),
        obj
          .get_named_property::<JsUnknown>(field)
          .map_err(|e| {
            CompilationError::NAPIError(format!("Get field {field} of config object failed. {e:?}"))
          })?
          .raw(),
      )
      .map_err(|e| {
        CompilationError::NAPIError(format!("Transform config field {field} failed. {e:?}"))
      })
    }
  } else {
    Err(CompilationError::NAPIError(format!(
      "Invalid Config: the config object does not have field {field}"
    )))
  }
}
