use std::{collections::HashMap, sync::Arc};

use farmfe_core::{
  config::{Config, ResolveConfig},
  context::CompilationContext,
  module::{ModuleId, ModuleType},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginAnalyzeDepsHookResultEntry, PluginHookContext,
    PluginLoadHookParam, PluginParseHookParam, ResolveKind,
  },
};
use farmfe_plugin_css::FarmPluginCss;
use farmfe_testing_helpers::fixture;
use farmfe_toolkit::css::codegen_css_stylesheet;

#[test]
fn analyze_deps() {
  fixture!("tests/fixtures/analyze_deps/basic.css", |file, cwd| {
    let config = Config {
      resolve: Box::new(ResolveConfig {
        alias: HashMap::from([
          ("/@".to_string(), cwd.to_string_lossy().to_string()),
          ("@".to_string(), cwd.to_string_lossy().to_string()),
        ]),
        ..Default::default()
      }),
      ..Default::default()
    };
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    let css_plugin = FarmPluginCss::new(&context.config);
    let load_result = css_plugin
      .load(
        &PluginLoadHookParam {
          resolved_path: &file.to_string_lossy(),
          query: vec![],
          meta: HashMap::new(),
          module_id: file.to_string_lossy().to_string(),
        },
        &context,
        &PluginHookContext::default(),
      )
      .unwrap();

    assert!(load_result.is_some());
    let load_result = load_result.unwrap();
    assert_eq!(load_result.module_type, ModuleType::Css);

    let parse_result = css_plugin
      .parse(
        &PluginParseHookParam {
          module_id: ModuleId::new(
            file.to_str().unwrap(),
            "",
            file.parent().unwrap().to_str().unwrap(),
          ),
          resolved_path: file.to_string_lossy().to_string(),
          query: vec![],
          module_type: load_result.module_type.clone(),
          content: Arc::new(load_result.content),
        },
        &context,
        &PluginHookContext::default(),
      )
      .unwrap()
      .unwrap();

    let mut css_module = farmfe_core::module::Module::new(ModuleId::new(
      file.to_str().unwrap(),
      "",
      file.parent().unwrap().to_str().unwrap(),
    ));
    css_module.module_type = load_result.module_type;
    css_module.meta = Box::new(parse_result);

    let mut params = PluginAnalyzeDepsHookParam {
      module: &css_module,
      deps: vec![],
    };

    css_plugin.analyze_deps(&mut params, &context).unwrap();

    assert_eq!(
      params.deps,
      vec![
        PluginAnalyzeDepsHookResultEntry {
          source: "./base.css".to_string(),
          kind: ResolveKind::CssAtImport
        },
        PluginAnalyzeDepsHookResultEntry {
          source: "./index.css".to_string(),
          kind: ResolveKind::CssAtImport
        },
        PluginAnalyzeDepsHookResultEntry {
          source: "./extension.css".to_string(),
          kind: ResolveKind::CssAtImport
        },
        PluginAnalyzeDepsHookResultEntry {
          source: "./background.png".to_string(),
          kind: ResolveKind::CssUrl
        },
        PluginAnalyzeDepsHookResultEntry {
          source: "./img/home.png".to_string(),
          kind: ResolveKind::CssUrl
        },
        PluginAnalyzeDepsHookResultEntry {
          source: "/@/img/logo.png".to_string(),
          kind: ResolveKind::CssUrl
        },
        PluginAnalyzeDepsHookResultEntry {
          source: "@/img/logo.png".to_string(),
          kind: ResolveKind::CssUrl
        },
      ]
    );

    let stylesheet = &css_module.meta.as_css().ast;
    let (css_code, _) = codegen_css_stylesheet(stylesheet, None, false, false);

    println!("{}", css_code);

    assert_eq!(
      css_code,
      r#"@import './base.css';
@import url(./index.css);
@import url("./extension.css");
@import '/public.css';
@import url(https://remote.css);
body {
  background: url('./background.png');
}
.home {
  background: url('./img/home.png') no-repeat;
}
div {
  background: url('/@/img/logo.png');
}
p {
  background: url('@/img/logo.png');
  top: -8px/2 + 1;
  --: 10px;}
.home {
  filter: progid:DXImageTransform.Microsoft.Alpha(opacity=20);
}"#
    );
  });
}
