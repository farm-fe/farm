use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::{Module, ModuleId},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginAnalyzeDepsHookResultEntry, PluginHookContext,
    PluginLoadHookParam, PluginParseHookParam, ResolveKind,
  },
  HashMap,
};
use farmfe_plugin_html::FarmPluginHtml;
use farmfe_testing_helpers::fixture;

#[test]
fn html_build_stage() {
  fixture("tests/fixtures/**/*.html", |file, _| {
    let context = Arc::new(CompilationContext::new(Default::default(), vec![]).unwrap());
    let hook_context = PluginHookContext {
      caller: None,
      meta: HashMap::default(),
    };
    let html_plugin = FarmPluginHtml::new(&Default::default());

    let file_content = html_plugin
      .load(
        &PluginLoadHookParam {
          resolved_path: file.to_str().unwrap(),
          query: vec![],
          meta: HashMap::default(),
          module_id: file.to_string_lossy().to_string(),
        },
        &context,
        &hook_context,
      )
      .unwrap()
      .unwrap();

    let module_id = ModuleId::new(file.to_str().unwrap(), "", &context.config.root);
    let html_module_meta_data = html_plugin
      .parse(
        &PluginParseHookParam {
          module_id: module_id.clone(),
          resolved_path: file.to_string_lossy().to_string(),
          content: Arc::new(file_content.content),
          module_type: file_content.module_type.clone(),
          query: vec![],
        },
        &context,
        &hook_context,
      )
      .unwrap()
      .unwrap();

    let mut html_module = Module::new(module_id);
    html_module.meta = Box::new(html_module_meta_data);
    html_module.module_type = file_content.module_type;

    let mut analyze_deps_param = PluginAnalyzeDepsHookParam {
      module: &html_module,
      deps: vec![],
    };

    html_plugin
      .analyze_deps(&mut analyze_deps_param, &context)
      .unwrap();

    assert_eq!(
      analyze_deps_param.deps,
      vec![PluginAnalyzeDepsHookResultEntry {
        source: "./src/main.ts".to_string(),
        kind: ResolveKind::ScriptSrc
      }]
    );
  });
}
