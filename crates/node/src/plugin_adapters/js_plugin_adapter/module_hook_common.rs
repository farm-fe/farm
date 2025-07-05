use std::sync::Arc;

use farmfe_core::{
  config::config_regex::ConfigRegex,
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{
    meta_data::script::CommentsMetaData, module_graph::ModuleGraphEdge, ModuleId, ModuleMetaData,
    ModuleType,
  },
  serde::{Deserialize, Serialize},
  swc_common::{comments::SingleThreadedComments, Globals, SourceMap},
  swc_css_ast::Stylesheet,
  swc_ecma_ast::{EsVersion, Module as SwcModule},
  swc_ecma_parser::{EsSyntax, Syntax},
};
use farmfe_toolkit::{
  css::{codegen_css_stylesheet, parse_css_stylesheet, ParseCssModuleResult},
  html::{codegen_html_document, parse_html_document},
  script::{
    codegen_module, parse_module, swc_try_with::resolve_module_mark, CodeGenCommentsConfig,
    ParseScriptModuleResult,
  },
};

pub struct ModuleHookFilters {
  pub module_types: Vec<ModuleType>,
  pub resolved_paths: Vec<ConfigRegex>,
}

#[napi(object)]
pub struct JsModuleHookFilters {
  pub module_types: Option<Vec<String>>,
  pub resolved_paths: Option<Vec<String>>,
}

impl From<JsModuleHookFilters> for ModuleHookFilters {
  fn from(value: JsModuleHookFilters) -> Self {
    let module_types = value.module_types.unwrap_or_default();
    let resolved_paths = value.resolved_paths.unwrap_or_default();

    Self {
      module_types: module_types.into_iter().map(|ty| ty.into()).collect(),
      resolved_paths: resolved_paths
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
  pub ignore_previous_source_map: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "farmfe_core::serde", rename_all = "camelCase")]
pub struct ModuleHookParams {
  pub module_id: ModuleId,
  pub module_type: ModuleType,
  pub content: String,
  pub source_map_chain: Vec<Arc<String>>,
  pub resolved_deps: Option<Vec<(ModuleId, ModuleGraphEdge)>>,
}

#[macro_export]
macro_rules! check_module_filters {
    ($plugin_name:expr, $hook_name:expr, $filters:expr) => {
      if $filters.module_types.is_empty() && $filters.resolved_paths.is_empty() {
        panic!("[Farm warn] filters.resolvedPaths or filters.moduleTypes of {}.{} must be set to control which module will be processed. If you want to process all modules, please set them to ['.*']", $plugin_name, $hook_name);
      }
    };
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

pub fn js_codegen(
  ast: &SwcModule,
  comments: &mut CommentsMetaData,
  cm: Arc<SourceMap>,
  context: &Arc<CompilationContext>,
) -> Result<(String, Option<String>)> {
  let source_map_enabled = !context.config.sourcemap.is_false();
  let taken_comments = std::mem::take(comments);
  let single_threaded_comments = SingleThreadedComments::from(taken_comments);
  let mut src_map = vec![];

  let code = codegen_module(
    ast,
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

  *comments = single_threaded_comments.into();

  let mut source_map = None;

  // append source map
  if source_map_enabled {
    let map = cm.build_source_map(&src_map);
    let mut src_map = vec![];
    map.to_writer(&mut src_map).map_err(|err| {
      CompilationError::GenericError(format!("failed to write source map: {err:?}"))
    })?;
    source_map = Some(String::from_utf8(src_map).unwrap());
  }

  Ok((String::from_utf8_lossy(&code).to_string(), source_map))
}

pub fn css_codegen(
  ast: &Stylesheet,
  cm: Arc<SourceMap>,
  context: &Arc<CompilationContext>,
) -> Result<(String, Option<String>)> {
  let source_map_enabled = !context.config.sourcemap.is_false();
  let (code, map) = codegen_css_stylesheet(
    &ast,
    false,
    if source_map_enabled { Some(cm) } else { None },
  );

  Ok((code, map))
}

pub fn format_module_metadata_to_code(
  meta: &mut ModuleMetaData,
  module_id: &ModuleId,
  source_map_chain: &mut Vec<Arc<String>>,
  context: &Arc<CompilationContext>,
) -> Result<Option<String>> {
  Ok(match meta {
    ModuleMetaData::Script(script_module_meta_data) => {
      let cm = context.meta.get_module_source_map(module_id);
      let (code, source_map) = js_codegen(
        &script_module_meta_data.ast,
        &mut script_module_meta_data.comments,
        cm,
        context,
      )?;

      if let Some(source_map) = source_map {
        source_map_chain.push(Arc::new(source_map));
      }

      Some(code)
    }
    ModuleMetaData::Css(css_module_meta_data) => {
      let cm = context.meta.get_module_source_map(module_id);
      let (code, source_map) = css_codegen(&css_module_meta_data.ast, cm, context)?;

      if let Some(source_map) = source_map {
        source_map_chain.push(Arc::new(source_map));
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
  result: ModuleHookResult,
  source_map_chain: &mut Vec<Arc<String>>,
  context: &Arc<CompilationContext>,
) -> Result<()> {
  if let Some(source_map) = result.source_map {
    if result.ignore_previous_source_map.unwrap_or(false) {
      *source_map_chain = vec![Arc::new(source_map)];
    } else {
      source_map_chain.push(Arc::new(source_map));
    }
  }

  let filename = module_id.to_string();
  let content = Arc::new(result.content);

  match meta {
    ModuleMetaData::Script(script_module_meta_data) => {
      let ParseScriptModuleResult {
        mut ast,
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

      let globals = Globals::new();
      let (unresolved_mark, top_level_mark) =
        resolve_module_mark(&mut ast, module_type.is_typescript(), &globals);
      script_module_meta_data.unresolved_mark = unresolved_mark.as_u32();
      script_module_meta_data.top_level_mark = top_level_mark.as_u32();

      context.meta.set_module_source_map(module_id, source_map);

      script_module_meta_data.ast = ast;
      script_module_meta_data.comments = comments.into();
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
