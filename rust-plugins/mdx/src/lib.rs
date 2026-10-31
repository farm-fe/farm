#![deny(clippy::all)]
use std::{fs, sync::Arc};

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::{CompilationError, Result},
  module::ModuleType,
  plugin::{
    Plugin, PluginHookContext, PluginLoadHookParam, PluginLoadHookResult, PluginTransformHookParam,
    PluginTransformHookResult,
  },
  serde_json,
};
use farmfe_macro_plugin::farm_plugin;
use mdxjs::{JsxRuntime, MdxParseOptions, Options};
use regex::Regex;
use serde::Deserialize;

#[derive(Debug)]
struct Configuration {
  mdx: Options,
  include: Option<Regex>,
  exclude: Option<Regex>,
  docs: bool,
  format: farmfe_docs_content::ContentFormat,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FarmPluginMdxOptions {
  include: Option<String>,
  exclude: Option<String>,
  development: Option<bool>,
  provider_import_source: Option<String>,
  jsx: Option<bool>,
  jsx_runtime: Option<JsxRuntime>,
  pragma: Option<String>,
  pragma_frag: Option<String>,
  pragma_import_source: Option<String>,
  jsx_import_source: Option<String>,
  filepath: Option<String>,
  parse: Option<MdxParseOptions>,
  #[serde(default)]
  docs: bool,
  #[serde(default)]
  format: farmfe_docs_content::ContentFormat,
}

impl Configuration {
  fn parse(options: &str) -> std::result::Result<Self, String> {
    let value: FarmPluginMdxOptions =
      serde_json::from_str(options).map_err(|error| error.to_string())?;
    let filter = |value: Option<String>| -> std::result::Result<Option<Regex>, String> {
      value
        .filter(|value| !value.is_empty())
        .map(|value| Regex::new(&value).map_err(|error| error.to_string()))
        .transpose()
    };
    Ok(Self {
      include: filter(value.include)?,
      exclude: filter(value.exclude)?,
      docs: value.docs,
      format: value.format,
      mdx: Options {
        development: value.development.unwrap_or(false),
        provider_import_source: value.provider_import_source,
        jsx: value.jsx.unwrap_or(false),
        jsx_runtime: Some(value.jsx_runtime.unwrap_or(JsxRuntime::Automatic)),
        pragma: Some(value.pragma.unwrap_or_else(|| "React.createElement".into())),
        pragma_frag: Some(value.pragma_frag.unwrap_or_else(|| "React.Fragment".into())),
        pragma_import_source: Some(value.pragma_import_source.unwrap_or_else(|| "react".into())),
        parse: value.parse.unwrap_or_default(),
        jsx_import_source: Some(value.jsx_import_source.unwrap_or_else(|| "react".into())),
        filepath: value.filepath,
      },
    })
  }

  fn includes(&self, file: &str) -> bool {
    (file.ends_with(".md") || file.ends_with(".mdx"))
      && self
        .include
        .as_ref()
        .is_none_or(|filter| filter.is_match(file))
      && self
        .exclude
        .as_ref()
        .is_none_or(|filter| !filter.is_match(file))
  }
}

#[farm_plugin]
#[derive(Debug)]
pub struct FarmPluginMdx {
  configuration: std::result::Result<Configuration, String>,
}

impl FarmPluginMdx {
  fn new(_config: &Config, options: String) -> Self {
    Self {
      configuration: Configuration::parse(&options),
    }
  }

  fn configuration(&self) -> Result<&Configuration> {
    self
      .configuration
      .as_ref()
      .map_err(|error| CompilationError::GenericError(format!("[MDX_OPTIONS] {error}")))
  }
}

impl Plugin for FarmPluginMdx {
  fn name(&self) -> &str {
    "FarmPluginMdx"
  }

  fn build_start(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    self.configuration()?;
    Ok(None)
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook: &PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    if !self.configuration()?.includes(param.resolved_path) {
      return Ok(None);
    }
    let content =
      fs::read_to_string(param.resolved_path).map_err(|error| CompilationError::LoadError {
        resolved_path: param.resolved_path.into(),
        source: Some(Box::new(error)),
      })?;
    Ok(Some(PluginLoadHookResult {
      content,
      module_type: ModuleType::Custom("mdx".into()),
      source_map: None,
    }))
  }

  fn transform(
    &self,
    param: &PluginTransformHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<PluginTransformHookResult>> {
    if param.module_type != ModuleType::Custom("mdx".into()) {
      return Ok(None);
    }
    let configuration = self.configuration()?;
    if !configuration.includes(param.resolved_path) {
      return Ok(None);
    }
    let mut options = configuration.mdx.clone();
    if options.filepath.is_none() {
      options.filepath = Some(param.resolved_path.into());
    }
    let (code, source_map) = if configuration.docs {
      let highlight: Option<farmfe_docs_content::HighlightOptions> = context
        .config
        .custom
        .get("farm:syntax-highlight")
        .map(|value| serde_json::from_str(value))
        .transpose()
        .map_err(|error| {
          CompilationError::GenericError(format!("[DOCS_HIGHLIGHT_OPTIONS] {error}"))
        })?;
      farmfe_docs_content::compile_content_with_format(
        &param.content,
        &options,
        context.config.sourcemap.enabled(false),
        highlight.as_ref(),
        configuration.format,
      )
      .map(|result| {
        for diagnostic in result.diagnostics {
          context
            .log_store
            .lock()
            .add_warning(format!("{}: {diagnostic}", param.resolved_path));
        }
        (result.code, result.source_map)
      })
    } else {
      mdxjs::compile(&param.content, &options)
        .map(|code| (code, None))
        .map_err(|error| error.to_string())
    }
    .map_err(|message| CompilationError::TransformError {
      resolved_path: param.resolved_path.into(),
      msg: format!("[MDX_COMPILE] {message}"),
    })?;
    Ok(Some(PluginTransformHookResult {
      content: code,
      module_type: Some(ModuleType::Jsx),
      source_map,
      ignore_previous_source_map: true,
    }))
  }
}
