use std::{path::Path, sync::Arc};

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::ModuleType,
  plugin::{Plugin, PluginHookContext, PluginLoadHookParam, PluginTransformHookParam},
  HashMap,
};
use farmfe_testing_helpers::fixture;

fn generate_transform_fn(
  file: &Path,
) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
  let config = Config::default();
  let json_plugin = farmfe_plugin_json::FarmPluginJson::new(&config);
  let id = file.to_string_lossy().to_string();

  let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());

  let param = PluginLoadHookParam {
    resolved_path: &id,
    query: vec![],
    meta: HashMap::default(),
    module_id: id.clone(),
  };
  let hook_context = PluginHookContext {
    caller: None,
    meta: HashMap::default(),
  };

  let loaded = json_plugin
    .load(&param, &context, &hook_context)
    .unwrap()
    .unwrap();

  let transform_param = PluginTransformHookParam {
    module_id: id.clone(),
    content: loaded.content,
    module_type: loaded.module_type,
    resolved_path: &id,
    query: vec![],
    meta: HashMap::default(),
    source_map_chain: vec![],
  };

  json_plugin.transform(&transform_param, &context)
}

#[test]
fn transform_json() {
  fixture!("tests/fixtures/transform/transform.json", |file, _| {
    let result = generate_transform_fn(&file).unwrap().unwrap();

    assert!(result.content.starts_with("module.exports ="));
    assert_eq!(result.module_type.unwrap(), ModuleType::Js);
  });

  fixture!(
    "tests/fixtures/transform/contain_comment.json",
    |file, _| {
      let result = generate_transform_fn(&file);

      assert!(matches!(result, Ok(None)));
    }
  );
}
