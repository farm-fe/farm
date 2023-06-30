use std::{collections::HashMap, sync::Arc};

use farmfe_core::{
  config::Config,
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
        },
        &Arc::new(context),
      )
      .unwrap()
      .unwrap();
    let expected =
      "body {\n  color: #000;\n}\nbody .description:hover {\n  background-color: #f8f9fa;\n}\n";
    assert_eq!(transformed.content, expected);
  });
}
