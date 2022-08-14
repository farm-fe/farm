use std::{collections::HashMap, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginAnalyzeDepsHookResultEntry, PluginHookContext,
    PluginLoadHookParam, PluginParseHookParam, ResolveKind,
  },
  serde_json::Value,
};
use farmfe_plugin_html::FarmPluginHtml;
use farmfe_toolkit::testing_helpers::fixture;

#[test]
fn html_build_stage() {
  fixture("tests/fixtures/**/*.html", |file| {
    let context = Arc::new(CompilationContext::new(Default::default(), vec![]).unwrap());
    let hook_context = PluginHookContext {
      caller: None,
      meta: HashMap::new(),
    };
    let html_plugin = FarmPluginHtml::new(&Default::default());

    let file_content = html_plugin
      .load(
        &PluginLoadHookParam {
          id: file.to_str().unwrap(),
          query: HashMap::new(),
        },
        &context,
        &hook_context,
      )
      .unwrap()
      .unwrap();

    let html_module = html_plugin
      .parse(
        &PluginParseHookParam {
          id: file.to_string_lossy().to_string(),
          content: file_content.content,
          module_type: file_content.module_type,
          query: HashMap::new(),
          source_map_chain: vec![],
          side_effects: false,
          package_json_info: Value::Null,
        },
        &context,
        &hook_context,
      )
      .unwrap()
      .unwrap();

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
