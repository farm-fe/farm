#![deny(clippy::all)]
use farmfe_core::module::ModuleType;
use farmfe_core::serde_json;
use mdxjs::compile;
use mdxjs::JsxRuntime;
use mdxjs::MdxParseOptions;
use mdxjs::Options;
use regex::Regex;
use serde::Deserialize;

use farmfe_core::{config::Config, plugin::Plugin};

use farmfe_core::plugin::PluginLoadHookResult;
use farmfe_macro_plugin::farm_plugin;
use std::fs::read_to_string;

#[farm_plugin]
#[derive(Debug, Default)]
pub struct FarmPluginMdx {
  mdx_options: Options,
  include: Option<String>,
  exclude: Option<String>,
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
}

fn is_mdx_file(file_name: &String) -> bool {
  file_name.ends_with(".md") || file_name.ends_with(".mdx")
}

impl FarmPluginMdx {
  fn new(_: &Config, options: String) -> Self {
    let plugin_options: FarmPluginMdxOptions =
      serde_json::from_str::<FarmPluginMdxOptions>(&options).unwrap();

    Self {
      mdx_options: Options {
        development: plugin_options.development.unwrap_or(false),
        provider_import_source: plugin_options.provider_import_source,
        jsx: plugin_options.jsx.unwrap_or(false),
        jsx_runtime: Some(plugin_options.jsx_runtime.unwrap_or(JsxRuntime::Automatic)),
        pragma: Some(
          plugin_options
            .pragma
            .unwrap_or("React.createElement".into()),
        ),
        pragma_frag: Some(
          plugin_options
            .pragma_frag
            .unwrap_or("React.Fragment".into()),
        ),
        pragma_import_source: Some(
          plugin_options
            .pragma_import_source
            .unwrap_or("react".into()),
        ),
        parse: plugin_options.parse.unwrap_or_default(),
        jsx_import_source: Some(plugin_options.jsx_import_source.unwrap_or("react".into())),
        filepath: plugin_options.filepath,
      },
      include: Some(plugin_options.include.unwrap_or("".into())),
      exclude: Some(plugin_options.exclude.unwrap_or("".into())),
    }
  }
}

impl Plugin for FarmPluginMdx {
  fn name(&self) -> &str {
    "FarmPluginMdx"
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    if is_mdx_file(&param.module_id) {
      let content = read_to_string(param.resolved_path).unwrap();
      return Ok(Some(farmfe_core::plugin::PluginLoadHookResult {
        content,
        source_map: None,
        module_type: ModuleType::Custom("mdx".to_string()),
      }));
    }
    Ok(None)
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    if param.module_type != ModuleType::Custom("mdx".to_string()) {
      return Ok(None);
    }

    if self.include != Some(String::from("")) {
      let inc_reg = Regex::new(&format!("{}", self.include.as_ref().unwrap())).unwrap();
      if let Some(_text) = inc_reg.find(param.resolved_path) {
      } else {
        return Ok(None);
      }
    }

    if self.exclude != Some(String::from("")) {
      let exc_reg = Regex::new(&format!("{}", self.exclude.as_ref().unwrap())).unwrap();
      if let Some(_text) = exc_reg.find(param.resolved_path) {
        return Ok(None);
      }
    }
    if param.module_id.ends_with(".mdx") || param.module_id.ends_with(".md") {
      let code = compile(&param.content, &self.mdx_options);
      let js_code = code.unwrap();
      return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
        content: js_code.to_string(),
        module_type: Some(ModuleType::Jsx),
        source_map: None,
        ignore_previous_source_map: true,
      }));
    }
    return Ok(None);
  }
}
