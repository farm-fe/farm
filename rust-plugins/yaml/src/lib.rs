#![deny(clippy::all)]

use farmfe_core::{
  config::Config,
  module::ModuleType,
  plugin::{
    Plugin, PluginLoadHookParam, PluginLoadHookResult, PluginTransformHookParam,
    PluginTransformHookResult,
  },
  serde_json,
};
use farmfe_macro_plugin::farm_plugin;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use std::fs::read_to_string;

lazy_static! {
  static ref YAML_MODULE_TYPE: String = String::from("yaml");
}

fn is_yaml_file(file_name: &String) -> bool {
  file_name.ends_with(".yaml") || file_name.ends_with(".yml")
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum DocumentMode {
  /// 单文档模式
  Single,
  /// 多文档模式
  Multi,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FarmPluginYamlOptions {
  document_mode: Option<DocumentMode>,
  include: Option<String>,
  exclude: Option<String>,
}

#[farm_plugin]
pub struct FarmPluginYaml {
  document_mode: DocumentMode,
  include: String,
  exclude: String,
}

impl FarmPluginYaml {
  fn new(_config: &Config, options: String) -> Self {
    let yaml_options: FarmPluginYamlOptions =
      serde_json::from_str(&options).expect("Failed to parse YAML plugin options");
    let include: String = yaml_options.include.unwrap_or_default();
    let exclude: String = yaml_options.exclude.unwrap_or_default();
    Self {
      document_mode: yaml_options.document_mode.unwrap_or(DocumentMode::Single),
      include,
      exclude,
    }
  }

  fn should_process_path(&self, path: &str) -> bool {
    if !self.include.is_empty() {
      let inc_reg = match Regex::new(&self.include) {
        Ok(reg) => reg,
        Err(_) => return false,
      };
      if inc_reg.find(path).is_none() {
        return false;
      }
    }

    if !self.exclude.is_empty() {
      let exc_reg = match Regex::new(&self.exclude) {
        Ok(reg) => reg,
        Err(_) => return true,
      };
      if exc_reg.find(path).is_some() {
        return false;
      }
    }

    true
  }

  fn yaml_to_js(&self, content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let result = match self.document_mode {
      DocumentMode::Single | DocumentMode::Multi => {
        serde_yaml::from_str::<serde_json::Value>(content)?
      }
    };

    let mut export_val = String::new();
    if let serde_json::Value::Object(object) = result.clone() {
      for (key, val) in object {
        export_val.push_str(&format!("export var {} = {};\n", key, val));
      }
    }

    Ok(format!("export default {};\n\n{}", result, export_val))
  }
}

impl Plugin for FarmPluginYaml {
  fn name(&self) -> &str {
    "FarmPluginYaml"
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    if is_yaml_file(&param.module_id) {
      let content = read_to_string(param.resolved_path).unwrap();
      return Ok(Some(PluginLoadHookResult {
        content,
        source_map: None,
        module_type: ModuleType::Custom(YAML_MODULE_TYPE.to_string()),
      }));
    }
    Ok(None)
  }

  fn transform(
    &self,
    param: &PluginTransformHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<PluginTransformHookResult>> {
    if param.module_type != ModuleType::Custom(YAML_MODULE_TYPE.to_string()) {
      return Ok(None);
    }

    if !self.should_process_path(param.resolved_path) {
      return Ok(None);
    }

    let code = match self.yaml_to_js(&param.content) {
      Ok(code) => code,
      Err(e) => panic!("Failed to parse YAML: {}", e),
    };

    Ok(Some(PluginTransformHookResult {
      content: code,
      module_type: Some(ModuleType::Js),
      source_map: None,
      ignore_previous_source_map: false,
    }))
  }
}
