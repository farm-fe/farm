use std::{collections::HashMap, path::Path, sync::Arc};

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::ModuleType,
  plugin::{Plugin, PluginLoadHookParam, PluginTransformHookParam},
  serde_json::{self, json},
};
use farmfe_pulgin_strip::Options;
use farmfe_testing_helpers::fixture;
use farmfe_toolkit::fs;

fn generate_transform_fn(
  file: &Path,
) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
  let config = Config::default();
  let options = json!({
    "debugger": true,
    "labels":["unittest"],
    "functions":["assert.*","console.*"],
    "include": [],
    "exclude": [],
  });
  let stri_options = serde_json::from_value::<Options>(options).unwrap();
  let strip_plugin = farmfe_pulgin_strip::FarmPulginStrip::new(&config, stri_options);
  let id = file.to_string_lossy().to_string();

  let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());

  let param = PluginLoadHookParam {
    resolved_path: &id,
    query: vec![],
    meta: HashMap::new(),
    module_id: id.clone(),
  };

  let content = fs::read_file_utf8(param.resolved_path)?;
  
  let transform_param = PluginTransformHookParam {
    module_id: id.clone(),
    content: content,
    module_type: ModuleType::Js,
    resolved_path: &id,
    query: vec![],
    meta: HashMap::new(),
    source_map_chain: vec![],
  };
  strip_plugin.transform(&transform_param, &context)
}

#[test]
fn transform_js() {
  fixture!("tests/fixtures/debugger/input.js", |file, _| {
    let result = generate_transform_fn(&file).unwrap().unwrap();
    assert_eq!(false, result.content.contains("debugger"))
  });

  fixture!("tests/fixtures/assert/input.js", |file, _| {
    let result = generate_transform_fn(&file).unwrap().unwrap();
    assert_eq!(false, result.content.contains("assert"))
  });

  fixture!("tests/fixtures/console/input.js", |file, _| {
    let result = generate_transform_fn(&file).unwrap().unwrap();
    assert_eq!(false, result.content.contains("console"))
  });

  fixture!("tests/fixtures/console-custom/input.js", |file, _| {
    let result = generate_transform_fn(&file).unwrap().unwrap();
    assert_eq!(false, result.content.contains("console"))
  });

  fixture!("tests/fixtures/label/input.js", |file, _| {
    let result = generate_transform_fn(&file).unwrap().unwrap();
    assert_eq!(false, result.content.contains("unittest"))
  });
}
