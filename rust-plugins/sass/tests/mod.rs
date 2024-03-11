use std::{collections::HashMap, sync::Arc};

use farmfe_compiler::Compiler;
use farmfe_core::{
  config::{
    bool_or_obj::BoolOrObj, preset_env::PresetEnvConfig, Config, ResolveConfig, RuntimeConfig,
    SourcemapConfig,
  },
  context::CompilationContext,
  module::ModuleType,
  plugin::{Plugin, PluginTransformHookParam},
};
use farmfe_plugin_sass::FarmPluginSass;
use farmfe_testing_helpers::fixture;
use farmfe_toolkit::fs::read_file_utf8;

#[test]
fn test() {
  fixture!("tests/fixtures/index.scss", |file, _cwd| {
    let resolved_path = file.to_string_lossy().to_string();
    let config = Config {
      input: HashMap::from([("button".to_string(), resolved_path.clone())]),
      ..Default::default()
    };
    let plugin = Arc::new(FarmPluginSass::new(
      &config,
      r#"
      {
        "sourceMap": true,
        "style":"expanded"
      }
    "#
      .to_string(),
    ));
    let context = CompilationContext::new(config, vec![plugin.clone()]).unwrap();
    let content = read_file_utf8(&resolved_path).unwrap();
    let transformed = plugin
      .transform(
        &PluginTransformHookParam {
          resolved_path: &resolved_path,
          content,
          module_type: ModuleType::Custom(String::from("sass")),
          query: vec![],
          meta: HashMap::from([]),
          module_id: resolved_path.clone(),
          source_map_chain: vec![],
        },
        &Arc::new(context),
      )
      .unwrap()
      .unwrap();
    let expected =
      "body {\n  color: #000;\n}\nbody .description:hover {\n  background-color: #f8f9fa;\n}";
    assert_eq!(transformed.content, expected);
  });
}

#[test]
fn test_with_compiler() {
  fixture!("tests/fixtures/**/*/index.scss", |file, crate_path| {
    println!("testing: {:?}", file);
    let resolved_path = file.to_string_lossy().to_string();
    let cwd = file.parent().unwrap();
    let runtime_path = crate_path
      .join("tests")
      .join("fixtures")
      .join("_internal")
      .join("runtime")
      .join("index.js")
      .to_string_lossy()
      .to_string();
    let config = Config {
      input: HashMap::from([("index".to_string(), resolved_path.clone())]),
      root: cwd.to_string_lossy().to_string(),
      runtime: RuntimeConfig {
        path: runtime_path,
        ..Default::default()
      },
      mode: farmfe_core::config::Mode::Production,
      sourcemap: SourcemapConfig::Bool(false),
      preset_env: Box::new(PresetEnvConfig::Bool(false)),
      minify: Box::new(BoolOrObj::from(false)),
      tree_shaking: false,
      progress: false,
      resolve: ResolveConfig {
        alias: std::collections::HashMap::from([(
          "@".to_string(),
          cwd.to_string_lossy().to_string(),
        )]),
        ..Default::default()
      },
      ..Default::default()
    };

    let plugin_sass = FarmPluginSass::new(
      &config,
      r#"
      {
        "sourceMap": true,
        "style":"expanded"
      }
    "#
      .to_string(),
    );
    let compiler = Compiler::new(config, vec![Arc::new(plugin_sass) as _]).unwrap();
    compiler.compile().unwrap();

    let resources_map = compiler.context().resources_map.lock();
    let css = resources_map.get("index.css").unwrap();
    let css_code = String::from_utf8(css.bytes.clone()).unwrap();

    let expected = "body {\n  color: red;\n}";
    assert_eq!(css_code, expected);
    let watch_graph = compiler.context().watch_graph.read();
    assert!(watch_graph.modules().len() > 0);
  });
}
