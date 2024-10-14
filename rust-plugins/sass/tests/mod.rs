use std::{collections::HashMap, fs, io::Write, path::PathBuf, sync::Arc};

use farmfe_compiler::Compiler;
use farmfe_core::{
  config::{
    bool_or_obj::BoolOrObj, preset_env::PresetEnvConfig, AliasItem, Config, ResolveConfig, RuntimeConfig, SourcemapConfig, StringOrRegex
  },
  context::CompilationContext,
  module::ModuleType,
  plugin::{Plugin, PluginTransformHookParam},
};
use farmfe_plugin_sass::FarmPluginSass;
use farmfe_testing_helpers::{fixture, is_update_snapshot_from_env};
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

fn normalize_css(css: &str) -> String {
  css.replace("\r\n", "\n")
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
      runtime: Box::new(RuntimeConfig {
        path: runtime_path,
        ..Default::default()
      }),
      mode: farmfe_core::config::Mode::Production,
      sourcemap: Box::new(SourcemapConfig::Bool(false)),
      preset_env: Box::new(PresetEnvConfig::Bool(false)),
      minify: Box::new(BoolOrObj::from(false)),
      tree_shaking: Box::new(BoolOrObj::Bool(false)),
      progress: false,
      resolve: Box::new(ResolveConfig {
        alias: vec![
          AliasItem::Complex {
            find: StringOrRegex::String("@".to_string()),
            replacement: cwd.to_string_lossy().to_string(),
          },
        ],
        ..Default::default()
      }),
      ..Default::default()
    };

    let config_filename = PathBuf::from_iter([cwd.to_str().unwrap(), "config.json"]);
    let plugin_sass = FarmPluginSass::new(
      &config,
      if let Ok(content) = fs::read_to_string(config_filename) {
        content
      } else {
        r#"
      {
        "sourceMap": true,
        "style":"expanded"
      }
    "#
        .to_string()
      },
    );
    let compiler = Compiler::new(config, vec![Arc::new(plugin_sass) as _]).unwrap();
    compiler.compile().unwrap();

    let resources_map = compiler.context().resources_map.lock();
    let css = resources_map.get("index.css").unwrap();
    let css_code = normalize_css(&String::from_utf8(css.bytes.clone()).unwrap());

    let output_filename = PathBuf::from_iter(vec![cwd.to_str().unwrap(), "output.css".into()]);

    if is_update_snapshot_from_env() || !output_filename.exists() {
      let mut output_file = std::fs::File::create(output_filename).unwrap();
      output_file.write_all(css_code.as_bytes()).unwrap();
    } else {
      let expected = normalize_css(&read_file_utf8(&output_filename.to_str().unwrap()).unwrap());
      assert_eq!(css_code, expected);
    }
  });
}
