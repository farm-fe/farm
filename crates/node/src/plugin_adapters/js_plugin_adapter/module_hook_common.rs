use std::sync::Arc;

use farmfe_core::{
  config::config_regex::ConfigRegex,
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{ModuleId, ModuleMetaData, ModuleType},
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

pub struct ModuleHookFilters {
  pub module_types: Vec<ModuleType>,
  pub resolved_paths: Vec<ConfigRegex>,
}

#[napi(object)]
pub struct JsModuleHookFilters {
  pub module_types: Vec<String>,
  pub resolved_paths: Vec<String>,
}

impl From<JsModuleHookFilters> for ModuleHookFilters {
  fn from(value: JsModuleHookFilters) -> Self {
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "farmfe_core::serde", rename_all = "camelCase")]
pub struct ModuleHookResult {
  pub content: String,
  pub source_map: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "farmfe_core::serde", rename_all = "camelCase")]
pub struct ModuleHookParams {
  pub module_id: ModuleId,
  pub module_type: ModuleType,
  pub content: String,
}

pub fn module_matches_filters(
  module_id: &ModuleId,
  module_type: &ModuleType,
  filters: &ModuleHookFilters,
) -> bool {
  filters.module_types.contains(module_type)
    || filters
      .resolved_paths
      .iter()
      .any(|m| m.is_match(module_id.to_string().as_str()))
}

pub fn format_module_metadata_to_code(
  meta: &mut ModuleMetaData,
  module_id: &ModuleId,
  source_map_chain: &mut Vec<Arc<String>>,
  context: &Arc<CompilationContext>,
) -> Result<Option<String>> {
  let source_map_enabled = !context.config.sourcemap.is_false();

  Ok(match meta {
    ModuleMetaData::Script(script_module_meta_data) => {
      let cm = context.meta.get_module_source_map(module_id);
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
        source_map_chain.push(Arc::new(String::from_utf8(src_map).unwrap()));
      }

      Some(String::from_utf8_lossy(&code).to_string())
    }
    ModuleMetaData::Css(css_module_meta_data) => {
      let cm = context.meta.get_module_source_map(module_id);
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
        source_map_chain.push(Arc::new(map));
      }

      Some(code)
    }
    ModuleMetaData::Html(html_module_meta_data) => {
      Some(codegen_html_document(&html_module_meta_data.ast, false))
    }
    ModuleMetaData::Custom(_) => None,
  })
}

pub fn convert_code_to_metadata(
  module_id: &ModuleId,
  module_type: &ModuleType,
  meta: &mut ModuleMetaData,
  content: Arc<String>,
  source_map: Option<String>,
  source_map_chain: &mut Vec<Arc<String>>,
  context: &Arc<CompilationContext>,
) -> Result<()> {
  if let Some(source_map) = source_map {
    source_map_chain.push(Arc::new(source_map));
  }

  let filename = module_id.to_string();

  match meta {
    ModuleMetaData::Script(script_module_meta_data) => {
      let ParseScriptModuleResult {
        ast,
        comments,
        source_map,
      } = parse_module(
        module_id,
        content,
        match module_type {
          ModuleType::Js | ModuleType::Ts => Syntax::Es(Default::default()),
          ModuleType::Jsx | ModuleType::Tsx => Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
          }),
          _ => Syntax::Es(Default::default()),
        },
        Default::default(),
      )?;

      context.meta.set_module_source_map(module_id, source_map);

      script_module_meta_data.ast = ast;
      script_module_meta_data.comments = comments.into()
    }
    ModuleMetaData::Css(css_module_meta_data) => {
      let ParseCssModuleResult {
        ast,
        comments,
        source_map,
      } = parse_css_stylesheet(&filename, content)?;

      context.meta.set_module_source_map(module_id, source_map);

      css_module_meta_data.ast = ast;
      css_module_meta_data.comments = comments.into();
    }
    ModuleMetaData::Html(html_module_meta_data) => {
      let v = parse_html_document(&filename, content)?;

      html_module_meta_data.ast = v;
    }
    ModuleMetaData::Custom(_) => {
      return Ok(());
    }
  }

  Ok(())
}
