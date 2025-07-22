use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext, error::Result, plugin::PluginProcessModuleHookParam,
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

pub type PluginProcessModuleHookResult = ModuleHookResult;
pub type CompatiblePluginProcessModuleHookParams = ModuleHookParams;

pub struct JsPluginProcessModuleHook {
  tsfn: ThreadSafeJsPluginHook,
  pub(crate) filters: ModuleHookFilters,
}

impl JsPluginProcessModuleHook {
  new_js_plugin_hook!(
    ModuleHookFilters,
    JsModuleHookFilters,
    CompatiblePluginProcessModuleHookParams,
    PluginProcessModuleHookResult
  );

  pub fn call(
    &self,
    param: &mut PluginProcessModuleHookParam,
    ctx: Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if module_matches_filters(param.module_id, param.module_type, &self.filters) {
      let Some(content) =
        format_module_metadata_to_code(param.meta, param.module_id, param.source_map_chain, &ctx)?
      else {
        return Ok(None);
      };

      let Some(result) = self
        .tsfn
        .call::<CompatiblePluginProcessModuleHookParams, PluginProcessModuleHookResult>(
          CompatiblePluginProcessModuleHookParams {
            module_id: param.module_id.clone(),
            module_type: param.module_type.clone(),
            content,
            source_map_chain: param.source_map_chain.clone(),
            resolved_deps: None,
          },
          ctx.clone(),
          None,
        )?
      else {
        return Ok(None);
      };

      convert_code_to_metadata(
        param.module_id,
        param.module_type,
        param.meta,
        result,
        param.source_map_chain,
        &ctx,
      )?;

      return Ok(None);
    }

    Ok(None)
  }
}
