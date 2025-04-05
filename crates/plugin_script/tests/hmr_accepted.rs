use std::sync::Arc;

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::Module,
  plugin::{
    Plugin, PluginFinalizeModuleHookParam, PluginHookContext, PluginLoadHookParam,
    PluginParseHookParam,
  },
  HashMap,
};
use farmfe_testing_helpers::fixture;

mod common;

#[test]
fn hmr_accepted() {
  fixture!("tests/fixtures/hmr_accepted/*.ts", |file, _| {
    let config = Config::default();
    let plugin_script = farmfe_plugin_script::FarmPluginScript::new(&config);
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    let id = file.to_string_lossy().to_string();
    let hook_context = PluginHookContext {
      caller: None,
      meta: HashMap::default(),
    };
    let loaded = plugin_script
      .load(
        &PluginLoadHookParam {
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

    let module_meta = plugin_script
      .parse(
        &PluginParseHookParam {
          module_id: "any".into(),
          resolved_path: id,
          query: vec![],
          module_type: loaded.module_type.clone(),
          content: Arc::new(loaded.content),
        },
        &context,
        &hook_context,
      )
      .unwrap()
      .unwrap();

    let mut module = Module::new("any".into());
    module.meta = Box::new(module_meta);
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
