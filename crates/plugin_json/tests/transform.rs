use std::{path::Path, sync::Arc};

use farmfe_core::{
  HashMap,
  config::Config,
  context::CompilationContext,
  module::ModuleType,
  plugin::{Plugin, PluginHookContext, PluginLoadHookParam, PluginTransformHookParam},
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

#[test]
fn transform_preserves_proto_properties() {
  use farmfe_core::config::{ModuleFormat, ModuleFormatConfig, TargetEnv};

  for format in [ModuleFormat::CommonJs, ModuleFormat::EsModule] {
    let mut config = Config::default();
    config.output.target_env = TargetEnv::Library;
    config.output.format = ModuleFormatConfig::Single(format);
    let plugin = farmfe_plugin_json::FarmPluginJson::new(&config);
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());

    for content in [
      r#"{"__proto__":{"x":1},"a":2}"#,
      r#"{"nested":{"__proto__":{"x":1}}}"#,
      r#"[{"__proto__":{"x":1}},[{"__proto__":null}]]"#,
      r#"{"__proto__":null}"#,
      r#"{"__proto__":42}"#,
      r#"{"__pr\u006fto__":{"x":1}}"#,
      r#"{"__proto__":{"__proto__":{"x":1}},"text":"quote\" backslash\\ newline\n"}"#,
      r#"{"constructor":{"prototype":{"x":1}},"text":"__proto__"}"#,
    ] {
      let param = PluginTransformHookParam {
        module_id: "proto.json".into(),
        content: content.into(),
        module_type: ModuleType::Custom("json".into()),
        resolved_path: "proto.json",
        query: vec![],
        meta: HashMap::default(),
        source_map_chain: vec![],
      };
      let result = plugin.transform(&param, &context).unwrap().unwrap();
      assert_eq!(result.module_type, Some(ModuleType::Js));
      let source = if format == ModuleFormat::CommonJs {
        format!(
          "const module = {{ exports: {{}} }}; {}; export default module.exports;",
          result.content
        )
      } else {
        result.content
      };
      let output = std::process::Command::new("node")
        .args([
          "--input-type=module",
          "-e",
          include_str!("fixtures/transform/assert_json_semantics.mjs"),
          &source,
          content,
        ])
        .output()
        .expect("Node.js is required to verify generated JSON module semantics");
      assert!(
        output.status.success(),
        "{format:?}: {content}\n{}",
        String::from_utf8_lossy(&output.stderr)
      );
    }
  }
}
