use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext, error::Result, plugin::PluginFreezeModuleHookParam,
};
use napi::{bindgen_prelude::FromNapiValue, NapiRaw};

use crate::{
  new_js_plugin_hook,
  plugin_adapters::js_plugin_adapter::module_hook_common::{
    convert_code_to_metadata, format_module_metadata_to_code, module_matches_filters,
    JsModuleHookFilters, ModuleHookFilters, ModuleHookParams, ModuleHookResult,
  },
  plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook,
};

pub type JsPluginFreezeModuleHookFilters = JsModuleHookFilters;
pub type PluginFreezeModuleHookFilters = ModuleHookFilters;
pub type PluginFreezeModuleHookResult = ModuleHookResult;
pub type CompatiblePluginFreezeModuleHookParams = ModuleHookParams;

pub struct JsPluginFreezeModuleHook {
  tsfn: ThreadSafeJsPluginHook,
  filters: PluginFreezeModuleHookFilters,
}

impl JsPluginFreezeModuleHook {
  new_js_plugin_hook!(
    PluginFreezeModuleHookFilters,
    JsPluginFreezeModuleHookFilters,
    CompatiblePluginFreezeModuleHookParams,
    PluginFreezeModuleHookResult
  );

  pub fn call(
    &self,
    param: &mut PluginFreezeModuleHookParam,
    ctx: Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if module_matches_filters(&param.module.id, &param.module.module_type, &self.filters) {
      let Some(result) = format_module_metadata_to_code(
        &mut param.module.meta,
        &param.module.id,
        &mut param.module.source_map_chain,
        &ctx,
      )?
      else {
        return Ok(None);
      };

      let Some(result) = self
        .tsfn
        .call::<CompatiblePluginFreezeModuleHookParams, PluginFreezeModuleHookResult>(
          CompatiblePluginFreezeModuleHookParams {
            module_id: param.module.id.clone(),
            module_type: param.module.module_type.clone(),
            content: result,
          },
          ctx.clone(),
          None,
        )?
      else {
        return Ok(None);
      };

      convert_code_to_metadata(
        &param.module.id,
        &param.module.module_type,
        &mut param.module.meta,
        Arc::new(result.content),
        result.source_map,
        &mut param.module.source_map_chain,
        &ctx,
      )?;

      return Ok(None);
    }

    Ok(None)
  }
}
