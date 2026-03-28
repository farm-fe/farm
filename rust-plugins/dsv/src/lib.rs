#![deny(clippy::all)]

use farmfe_toolkit::plugin_utils::path_filter::PathFilter;
use serde_json::Value;
use std::{error::Error, path::Path};

use csv::{Reader, ReaderBuilder};
use farmfe_core::{
  config::{config_regex::ConfigRegex, Config},
  context::CompilationContext,
  error::Result as HookResult,
  module::ModuleType,
  plugin::{Plugin, PluginTransformHookParam, PluginTransformHookResult},
  serde_json,
};

use farmfe_macro_plugin::farm_plugin;

#[derive(serde::Deserialize, Clone)]
pub struct Options {
  pub include: Option<Vec<ConfigRegex>>,
  pub exclude: Option<Vec<ConfigRegex>>,
}

#[farm_plugin]
pub struct FarmPluginDsv {
  options: Options,
}

impl FarmPluginDsv {
  fn new(_config: &Config, options: String) -> Self {
    let options: Options = serde_json::from_str(&options).unwrap();
    Self { options }
  }
}
struct Param {
  module_id: String,
  content: String,
}

fn get_reader(param: &Param) -> Result<Reader<&[u8]>, Box<dyn Error>> {
  let extname = Path::new(&param.module_id)
    .extension()
    .ok_or("No file extension found")?
    .to_str()
    .ok_or("File extension cannot be converted to string")?;

  let reader = match extname {
    "csv" => ReaderBuilder::new().from_reader(param.content.as_bytes()),
    "tsv" => ReaderBuilder::new()
      .delimiter(b'\t')
      .from_reader(param.content.as_bytes()),
    _ => return Err("Unsupported file type".into()),
  };

  Ok(reader)
}

impl Plugin for FarmPluginDsv {
  fn name(&self) -> &str {
    "FarmPluginDsv"
  }
  fn transform(
    &self,
    param: &PluginTransformHookParam,
    _context: &std::sync::Arc<CompilationContext>,
  ) -> HookResult<Option<PluginTransformHookResult>> {
    let options = self.options.clone();
    let include = options.include.unwrap_or(vec![]);
    let exclude = options.exclude.unwrap_or(vec![]);
    let filter = PathFilter::new(&include, &exclude);
    if !filter.execute(&param.module_id) {
      return Ok(None);
    }

    let binding = Param {
      module_id: param.module_id.clone(),
      content: param.content.clone(),
    };

    let reader = get_reader(&binding);

    if reader.is_err() {
      return Ok(None);
    }

    let mut records = vec![];
    for result in reader.unwrap().records() {
      let record = result.unwrap();
      let json_record: Value = record
        .iter()
        .map(|field| Value::String(field.to_string()))
        .collect();
      records.push(json_record);
    }
    let json_string = serde_json::to_string(&records).unwrap();

    Ok(Some(PluginTransformHookResult {
      content: format!("export default {}", json_string),
      module_type: Some(ModuleType::Custom("json".to_string())),
      source_map: None,
      ignore_previous_source_map: true,
    }))
  }
}
