use std::sync::Arc;

use farmfe_core::{
  config::{config_regex::ConfigRegex, custom::get_config_output_ascii_only},
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{ModuleId, ModuleMetaData, ModuleType},
  plugin::PluginProcessModuleHookParam,
  serde::{Deserialize, Serialize},
  swc_common::SourceMap,
  swc_ecma_parser::{EsSyntax, Syntax},
};
use farmfe_toolkit::{
  css::{codegen_css_stylesheet, parse_css_stylesheet, ParseCssModuleResult},
  html::{codegen_html_document, parse_html_document},
  script::{codegen_module, create_codegen_config, parse_module, ParseScriptModuleResult},
};
use napi::{bindgen_prelude::FromNapiValue, NapiRaw};

use crate::{
  new_js_plugin_hook,
  plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook,
};

#[napi(object)]
pub struct JsPluginProcessModuleHookFilters {
  pub module_types: Vec<String>,
  pub resolved_paths: Vec<String>,
}

pub struct PluginProcessModuleHookFilters {
  pub module_types: Vec<ModuleType>,
  pub resolved_paths: Vec<ConfigRegex>,
}

impl From<JsPluginProcessModuleHookFilters> for PluginProcessModuleHookFilters {
  fn from(value: JsPluginProcessModuleHookFilters) -> Self {
    Self {
      module_types: value.module_types.into_iter().map(|ty| ty.into()).collect(),
      resolved_paths: value
        .resolved_paths
        .into_iter()
        .map(|p| ConfigRegex::new(&p))
        .collect(),
    }
  }
}

pub struct JsPluginProcessModuleHook {
  tsfn: ThreadSafeJsPluginHook,
  filters: PluginProcessModuleHookFilters,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "farmfe_core::serde", rename_all = "camelCase")]
pub struct PluginProcessModuleHookResult {
  content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "farmfe_core::serde", rename_all = "camelCase")]
struct CompatiblePluginProcessModuleHookParams {
  module_id: ModuleId,
  module_type: ModuleType,
  content: String,
}

fn format_module_metadata_to_code(
  metadata: &mut ModuleMetaData,
  context: &Arc<CompilationContext>,
) -> Result<Option<String>> {
  Ok(match metadata {
    ModuleMetaData::Script(script_module_meta_data) => {
      let cm = Arc::new(SourceMap::default());
      let code = codegen_module(
        &script_module_meta_data.ast,
        cm,
        None,
        create_codegen_config(context).with_minify(false),
        None,
      )
      .map_err(|err| CompilationError::GenericError(err.to_string()))?;

      Some(String::from_utf8_lossy(&code).to_string())
    }
    ModuleMetaData::Css(css_module_meta_data) => {
      let (code, _) = codegen_css_stylesheet(
        &css_module_meta_data.ast,
        None,
        false,
        get_config_output_ascii_only(&context.config),
      );

      Some(code)
    }
    ModuleMetaData::Html(html_module_meta_data) => {
      Some(codegen_html_document(&html_module_meta_data.ast, false))
    }
    ModuleMetaData::Custom(_) => None,
  })
}

fn convert_code_to_metadata(params: &mut PluginProcessModuleHookParam, code: String) -> Result<()> {
  let filename = params.module_id.to_string();
  match params.meta {
    ModuleMetaData::Script(script_module_meta_data) => {
      let ParseScriptModuleResult { ast, comments } = parse_module(
        &filename,
        &code,
        // TODO: config should from config or process_module custom config
        match params.module_type {
          ModuleType::Js | ModuleType::Ts => Syntax::Es(Default::default()),
          ModuleType::Jsx | ModuleType::Tsx => Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
          }),
          _ => Syntax::Es(Default::default()),
        },
        Default::default(),
      )?;

      script_module_meta_data.ast = ast;
      script_module_meta_data.comments = comments.into()
    }
    ModuleMetaData::Css(css_module_meta_data) => {
      let ParseCssModuleResult { ast, comments } = parse_css_stylesheet(&filename, Arc::new(code))?;

      css_module_meta_data.ast = ast;
      css_module_meta_data.comments = comments.into();
    }
    ModuleMetaData::Html(html_module_meta_data) => {
      let v = parse_html_document(&filename, Arc::new(code))?;

      html_module_meta_data.ast = v;
    }
    ModuleMetaData::Custom(_) => {
      return Ok(());
    }
  }

  Ok(())
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
    if self.filters.module_types.contains(param.module_type)
      || self
        .filters
        .resolved_paths
        .iter()
        .any(|m| m.is_match(param.module_id.to_string().as_str()))
    {
      let Some(result) = format_module_metadata_to_code(param.meta, &ctx)? else {
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
          ctx,
          None,
        )?
      else {
        return Ok(None);
      };

      convert_code_to_metadata(param, result.content)?;

      return Ok(None);
    }

    Ok(None)
  }
}
