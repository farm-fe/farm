#![deny(clippy::all)]

use farmfe_toolkit::plugin_utils::path_filter::PathFilter;
use serde_json::Value;
use std::{error::Error, path::Path};

use csv::{Reader, ReaderBuilder};
use farmfe_core::{
  config::{Config, config_regex::ConfigRegex},
  context::CompilationContext,
  error::{CompilationError, Result as HookResult},
  module::ModuleType,
  plugin::{
    Plugin, PluginLoadHookParam, PluginLoadHookResult, PluginTransformHookParam,
    PluginTransformHookResult,
  },
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

  fn load_path(&self, resolved_path: &str) -> HookResult<Option<PluginLoadHookResult>> {
    let extension = Path::new(resolved_path)
      .extension()
      .and_then(|value| value.to_str());
    if !matches!(extension, Some("csv" | "tsv")) {
      return Ok(None);
    }
    let include = self.options.include.clone().unwrap_or_default();
    let exclude = self.options.exclude.clone().unwrap_or_default();
    if !PathFilter::new(&include, &exclude).execute(resolved_path) {
      return Ok(None);
    }
    let content =
      std::fs::read_to_string(resolved_path).map_err(|error| CompilationError::LoadError {
        resolved_path: resolved_path.to_string(),
        source: Some(Box::new(error)),
      })?;
    Ok(Some(PluginLoadHookResult {
      content,
      module_type: ModuleType::Custom("dsv".to_string()),
      source_map: None,
    }))
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
  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &std::sync::Arc<CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> HookResult<Option<PluginLoadHookResult>> {
    self.load_path(param.resolved_path)
  }

  fn transform(
    &self,
    param: &PluginTransformHookParam,
    _context: &std::sync::Arc<CompilationContext>,
  ) -> HookResult<Option<PluginTransformHookResult>> {
    let options = self.options.clone();
    let include = options.include.unwrap_or_default();
    let exclude = options.exclude.unwrap_or_default();
    let filter = PathFilter::new(&include, &exclude);
    if !filter.execute(param.resolved_path) {
      return Ok(None);
    }

    let binding = Param {
      module_id: param.resolved_path.to_string(),
      content: param.content.clone(),
    };

    let reader = get_reader(&binding);

    if reader.is_err() {
      return Ok(None);
    }

    let json_string = records_to_json(reader.unwrap(), param.resolved_path)?;

    Ok(Some(PluginTransformHookResult {
      content: format!("export default {json_string}"),
      module_type: Some(ModuleType::Js),
      source_map: None,
      ignore_previous_source_map: true,
    }))
  }
}

fn records_to_json(mut reader: Reader<&[u8]>, resolved_path: &str) -> HookResult<String> {
  let error = |error: csv::Error| CompilationError::TransformError {
    resolved_path: resolved_path.to_string(),
    msg: error.to_string(),
  };
  let headers = reader.headers().map_err(error)?.clone();
  let mut rows = Vec::new();
  for record in reader.records() {
    let record = record.map_err(error)?;
    let row: serde_json::Map<String, Value> = headers
      .iter()
      .zip(record.iter())
      .map(|(header, value)| (header.to_string(), Value::String(value.to_string())))
      .collect();
    rows.push(Value::Object(row));
  }
  Ok(serde_json::to_string(&rows).unwrap())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::{
    fs::OpenOptions,
    io::Write,
    sync::atomic::{AtomicU64, Ordering},
  };
  static NEXT: AtomicU64 = AtomicU64::new(0);

  struct Input(std::path::PathBuf);
  impl Input {
    fn new(extension: &str, contents: &str) -> Self {
      let path = std::env::temp_dir().join(format!(
        "farm-dsv-{}-{}.{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
        extension
      ));
      let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
      file.write_all(contents.as_bytes()).unwrap();
      Self(path)
    }
    fn path(&self) -> &str {
      self.0.to_str().unwrap()
    }
  }
  impl Drop for Input {
    fn drop(&mut self) {
      let _ = std::fs::remove_file(&self.0);
    }
  }

  #[test]
  fn loads_csv_and_tsv_before_transform() {
    let plugin = FarmPluginDsv::new(&Config::default(), "{}".to_string());
    for (extension, content, expected) in [
      ("csv", "name,value\nfirst,42\n", "42"),
      ("tsv", "name\tvalue\nfirst\t43\n", "43"),
    ] {
      let input = Input::new(extension, content);
      let loaded = plugin.load_path(input.path()).unwrap().unwrap();
      assert_eq!(loaded.content, content);
      assert_eq!(loaded.module_type, ModuleType::Custom("dsv".to_string()));
      let param = Param {
        module_id: input.path().to_string(),
        content: loaded.content,
      };
      let mut reader = get_reader(&param).unwrap();
      assert_eq!(
        reader.records().next().unwrap().unwrap().get(1),
        Some(expected)
      );
    }
  }

  #[test]
  fn respects_filters_and_unsupported_extensions() {
    let plugin = FarmPluginDsv::new(
      &Config::default(),
      r#"{"include":["\\.csv$"],"exclude":["blocked"]}"#.to_string(),
    );
    let csv = Input::new("csv", "name\nfirst\n");
    let tsv = Input::new("tsv", "name\nfirst\n");
    let txt = Input::new("txt", "name\nfirst\n");
    assert!(plugin.load_path(csv.path()).unwrap().is_some());
    assert!(plugin.load_path(tsv.path()).unwrap().is_none());
    assert!(plugin.load_path(txt.path()).unwrap().is_none());
    assert!(plugin.load_path("blocked.csv").unwrap().is_none());
  }

  #[test]
  fn missing_supported_file_returns_load_error() {
    let plugin = FarmPluginDsv::new(&Config::default(), "{}".to_string());
    let input = Input::new("csv", "name\nfirst\n");
    std::fs::remove_file(&input.0).unwrap();
    assert!(matches!(
      plugin.load_path(input.path()),
      Err(CompilationError::LoadError { .. })
    ));
  }
  #[test]
  fn csv_rows_match_documented_header_objects() {
    let param = Param {
      module_id: "fruit.csv".to_string(),
      content: "type,count\napples,7\npears,4\n".to_string(),
    };
    let json = records_to_json(get_reader(&param).unwrap(), &param.module_id).unwrap();
    assert_eq!(
      serde_json::from_str::<Value>(&json).unwrap(),
      serde_json::json!([{"type":"apples","count":"7"},{"type":"pears","count":"4"}])
    );
  }

  #[test]
  fn tsv_rows_keep_named_shipping_fields_and_string_values() {
    let param = Param {
      module_id: "shipping.tsv".to_string(),
      content: "destination\tcents\tfreeAbove\ndomestic\t500\t10000\n".to_string(),
    };
    let json = records_to_json(get_reader(&param).unwrap(), &param.module_id).unwrap();
    assert_eq!(
      serde_json::from_str::<Value>(&json).unwrap(),
      serde_json::json!([{"destination":"domestic","cents":"500","freeAbove":"10000"}])
    );
  }

  #[test]
  fn quoted_csv_fields_keep_commas_and_header_only_input_is_empty() {
    for (content, expected) in [
      (
        "name,value\n\"last, first\",42\n",
        serde_json::json!([{"name":"last, first","value":"42"}]),
      ),
      ("name,value\n", serde_json::json!([])),
    ] {
      let param = Param {
        module_id: "quoted.csv".to_string(),
        content: content.to_string(),
      };
      let json = records_to_json(get_reader(&param).unwrap(), &param.module_id).unwrap();
      assert_eq!(serde_json::from_str::<Value>(&json).unwrap(), expected);
    }
  }

  #[test]
  fn malformed_row_width_is_a_transform_error() {
    let param = Param {
      module_id: "bad.csv".to_string(),
      content: "name,value\nonly-one-field\n".to_string(),
    };
    assert!(matches!(
      records_to_json(get_reader(&param).unwrap(), &param.module_id),
      Err(CompilationError::TransformError { .. })
    ));
  }
}
