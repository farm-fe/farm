use std::sync::Arc;

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::{meta_data::script::ScriptModuleMetaData, Module},
  plugin::{Plugin, PluginFinalizeModuleHookParam, PluginLoadHookResult},
};
use farmfe_testing_helpers::fixture;
use farmfe_toolkit::{
  fs::read_file_utf8,
  script::{parse_module, syntax_from_module_type},
};

#[test]
fn hmr_accepted() {
  fixture!("tests/fixtures/hmr_accepted/*.ts", |file, _| {
    let config = Config::default();
    let plugin_script = farmfe_plugin_runtime::FarmPluginRuntime::new(&config);
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    let id = file.to_string_lossy().to_string();

    let loaded = PluginLoadHookResult {
      content: read_file_utf8(id.as_str()).unwrap(),
      module_type: farmfe_core::module::ModuleType::Tsx,
      source_map: None,
    };

    let ast = parse_module(
      &"any".into(),
      Arc::new(loaded.content),
      syntax_from_module_type(&loaded.module_type, context.config.script.parser.clone()).unwrap(),
      farmfe_core::swc_ecma_ast::EsVersion::EsNext,
    )
    .unwrap();

    let mut module = Module::new("any".into());
    module.meta = Box::new(farmfe_core::module::ModuleMetaData::Script(Box::new(
      ScriptModuleMetaData {
        ast: ast.ast,
        comments: ast.comments.into(),
        ..Default::default()
      },
    )));
    context
      .meta
      .set_module_source_map(&"any".into(), ast.source_map);
    module.module_type = loaded.module_type;

    assert!(!module.meta.as_script().hmr_self_accepted);
    plugin_script
      .finalize_module(
        &mut PluginFinalizeModuleHookParam {
          module: &mut module,
          deps: &mut vec![],
        },
        &context,
      )
      .unwrap();
    assert!(module.meta.as_script().hmr_self_accepted);
  });
}
