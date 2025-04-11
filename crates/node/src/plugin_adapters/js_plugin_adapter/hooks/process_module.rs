use std::sync::Arc;

use farmfe_core::{
  config::config_regex::ConfigRegex,
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{ModuleId, ModuleMetaData, ModuleType},
  plugin::PluginProcessModuleHookParam,
  serde::{Deserialize, Serialize},
  swc_common::comments::SingleThreadedComments,
  swc_ecma_ast::EsVersion,
  swc_ecma_parser::{EsSyntax, Syntax},
};
use farmfe_toolkit::{
  css::{codegen_css_stylesheet, parse_css_stylesheet, ParseCssModuleResult},
  html::{codegen_html_document, parse_html_document},
  script::{codegen_module, parse_module, CodeGenCommentsConfig, ParseScriptModuleResult},
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

pub type JsPluginProcessModuleHookFilters = JsModuleHookFilters;
pub type PluginProcessModuleHookFilters = ModuleHookFilters;
pub type PluginProcessModuleHookResult = ModuleHookResult;
pub type CompatiblePluginProcessModuleHookParams = ModuleHookParams;

pub struct JsPluginProcessModuleHook {
  tsfn: ThreadSafeJsPluginHook,
  filters: PluginProcessModuleHookFilters,
}

impl JsPluginProcessModuleHook {
  new_js_plugin_hook!(
    PluginProcessModuleHookFilters,
    JsPluginProcessModuleHookFilters,
    CompatiblePluginProcessModuleHookParams,
    PluginProcessModuleHookResult
  );

  pub fn call(
    &self,
    param: &mut PluginProcessModuleHookParam,
    ctx: Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if module_matches_filters(param.module_id, param.module_type, &self.filters) {
      let Some(result) =
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
            content: result,
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
        Arc::new(result.content),
        result.source_map,
        param.source_map_chain,
        &ctx,
      )?;

      return Ok(None);
    }

    Ok(None)
  }
}
