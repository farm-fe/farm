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
  source_map: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "farmfe_core::serde", rename_all = "camelCase")]
struct CompatiblePluginProcessModuleHookParams {
  module_id: ModuleId,
  module_type: ModuleType,
  content: String,
}

fn format_module_metadata_to_code(
  param: &mut PluginProcessModuleHookParam,
  context: &Arc<CompilationContext>,
) -> Result<Option<String>> {
  let source_map_enabled = !context.config.sourcemap.is_false();

  Ok(match param.meta {
    ModuleMetaData::Script(script_module_meta_data) => {
      let cm = context.meta.get_module_source_map(param.module_id);
      let mut src_map = vec![];
      let comments = std::mem::take(&mut script_module_meta_data.comments);
      let single_threaded_comments = SingleThreadedComments::from(comments);

      let code = codegen_module(
        &script_module_meta_data.ast,
        EsVersion::latest(),
        cm.clone(),
        if source_map_enabled {
          Some(&mut src_map)
        } else {
          None
        },
        false,
        Some(CodeGenCommentsConfig {
          comments: &single_threaded_comments,
          config: &context.config.comments,
        }),
      )
      .map_err(|err| CompilationError::GenericError(err.to_string()))?;
      // write back the comments
      script_module_meta_data.comments = single_threaded_comments.into();
      // append source map
      if source_map_enabled {
        let map = cm.build_source_map(&src_map);
        let mut src_map = vec![];
        map.to_writer(&mut src_map).map_err(|err| {
          CompilationError::GenericError(format!("failed to write source map: {err:?}"))
        })?;
        param
          .source_map_chain
          .push(Arc::new(String::from_utf8(src_map).unwrap()));
      }

      Some(String::from_utf8_lossy(&code).to_string())
    }
    ModuleMetaData::Css(css_module_meta_data) => {
      let cm = context.meta.get_module_source_map(param.module_id);
      let (code, map) = codegen_css_stylesheet(
        &css_module_meta_data.ast,
        false,
        if source_map_enabled {
          Some(cm.clone())
        } else {
          None
        },
      );

      if let Some(map) = map {
        param.source_map_chain.push(Arc::new(map));
      }

      Some(code)
    }
    ModuleMetaData::Html(html_module_meta_data) => {
      Some(codegen_html_document(&html_module_meta_data.ast, false))
    }
    ModuleMetaData::Custom(_) => None,
  })
}

fn convert_code_to_metadata(
  params: &mut PluginProcessModuleHookParam,
  code: Arc<String>,
  source_map: Option<String>,
  context: &Arc<CompilationContext>,
) -> Result<()> {
  *params.content = code.clone();

  if let Some(source_map) = source_map {
    params.source_map_chain.push(Arc::new(source_map));
  }

  let filename = params.module_id.to_string();

  match params.meta {
    ModuleMetaData::Script(script_module_meta_data) => {
      let ParseScriptModuleResult {
        ast,
        comments,
        source_map,
      } = parse_module(
        params.module_id,
        code,
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

      context
        .meta
        .set_module_source_map(params.module_id, source_map);

      script_module_meta_data.ast = ast;
      script_module_meta_data.comments = comments.into()
    }
    ModuleMetaData::Css(css_module_meta_data) => {
      let ParseCssModuleResult {
        ast,
        comments,
        source_map,
      } = parse_css_stylesheet(&filename, code)?;

      context
        .meta
        .set_module_source_map(params.module_id, source_map);

      css_module_meta_data.ast = ast;
      css_module_meta_data.comments = comments.into();
    }
    ModuleMetaData::Html(html_module_meta_data) => {
      let v = parse_html_document(&filename, code)?;

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
      let Some(result) = format_module_metadata_to_code(param, &ctx)? else {
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

      convert_code_to_metadata(param, Arc::new(result.content), result.source_map, &ctx)?;

      return Ok(None);
    }

    Ok(None)
  }
}
