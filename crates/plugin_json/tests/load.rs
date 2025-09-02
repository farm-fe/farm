use std::sync::Arc;

use farmfe_core::{
  config::Config, context::CompilationContext, plugin::Plugin, plugin::PluginHookContext, HashMap,
};
use farmfe_testing_helpers::fixture;

#[test]
fn load_json() {
  fixture!("tests/fixtures/load/load.json", |file, _| {
    let config = Config::default();
    let plugin_json = farmfe_plugin_json::FarmPluginJson::new(&config);
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());

    let id = file.to_string_lossy().to_string();

    let hook_context = PluginHookContext {
      caller: None,
      meta: HashMap::default(),
    };

    let loaded = plugin_json
      .load(
        &farmfe_core::plugin::PluginLoadHookParam {
          resolved_path: &id,
          query: vec![],
          meta: HashMap::default(),
          module_id: id.clone(),
        },
        &context,
        &hook_context,
      )
      .unwrap()
      .unwrap();

    assert_eq!(
      loaded.module_type,
      farmfe_core::module::ModuleType::Custom("json".into())
    );

    assert!(loaded.content.contains("\"hello\""));
  });
}
