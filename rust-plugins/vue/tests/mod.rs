//! Unit-level tests for `farmfe_plugin_vue` that exercise the plugin's
//! `transform` and `load` hooks directly with a minimal in-memory Farm
//! [`CompilationContext`]. These tests focus on observable plugin
//! behaviour (which fixtures get compiled, which virtual modules get
//! registered, which module types get tagged) rather than golden-string
//! comparisons against fervid's evolving output.

use std::sync::Arc;

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::ModuleType,
  plugin::{
    Plugin, PluginHookContext, PluginLoadHookParam, PluginTransformHookParam,
    PluginTransformHookResult,
  },
};

use farmfe_plugin_vue::FarmPluginVue;

fn make_plugin(options: &str) -> (Arc<CompilationContext>, FarmPluginVue) {
  let config = Config::default();
  let plugin = FarmPluginVue::new(&config, options.to_string());
  let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
  (context, plugin)
}

fn fixture_path(name: &str) -> String {
  let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("tests/fixtures")
    .join(name);
  p.to_string_lossy().into_owned()
}

fn load_fixture(plugin: &FarmPluginVue, context: &Arc<CompilationContext>, name: &str) -> String {
  let path = fixture_path(name);
  let load_param = PluginLoadHookParam {
    module_id: path.clone(),
    resolved_path: &path,
    query: vec![],
    meta: Default::default(),
  };
  let hook_ctx = PluginHookContext {
    caller: None,
    meta: Default::default(),
  };
  let res = plugin
    .load(&load_param, context, &hook_ctx)
    .expect("load returns Ok")
    .expect("load returns Some for .vue files");
  assert!(
    matches!(&res.module_type, ModuleType::Custom(t) if t == "vue"),
    "expected vue module type, got {:?}",
    res.module_type
  );
  res.content
}

fn transform_fixture(
  plugin: &FarmPluginVue,
  context: &Arc<CompilationContext>,
  name: &str,
) -> String {
  transform_fixture_result(plugin, context, name).content
}

fn transform_fixture_result(
  plugin: &FarmPluginVue,
  context: &Arc<CompilationContext>,
  name: &str,
) -> PluginTransformHookResult {
  let path = fixture_path(name);
  let content = load_fixture(plugin, context, name);
  let transform_param = PluginTransformHookParam {
    module_id: path.clone(),
    content,
    module_type: ModuleType::Custom("vue".to_string()),
    resolved_path: &path,
    query: vec![],
    meta: Default::default(),
    source_map_chain: vec![],
  };
  let res = plugin
    .transform(&transform_param, context)
    .expect("transform returns Ok")
    .expect("transform returns Some for vue files");
  assert_eq!(res.module_type, Some(ModuleType::Ts));
  res
}

#[test]
fn ignores_non_vue_paths() {
  let (context, plugin) = make_plugin("");
  let path = "/tmp/not-a-vue-file.js".to_string();
  let load_param = PluginLoadHookParam {
    module_id: path.clone(),
    resolved_path: &path,
    query: vec![],
    meta: Default::default(),
  };
  let hook_ctx = PluginHookContext {
    caller: None,
    meta: Default::default(),
  };
  assert!(plugin
    .load(&load_param, &context, &hook_ctx)
    .unwrap()
    .is_none());
}

#[test]
fn compiles_basic_sfc() {
  let (context, plugin) = make_plugin("");
  let code = transform_fixture(&plugin, &context, "basic.vue");
  // Sanity: should contain the template string somewhere in fervid's output.
  assert!(
    code.contains("greeting") || code.contains("createElementVNode") || code.contains("_sfc_main"),
    "expected compiled output to reference template or sfc_main, got:\n{code}"
  );
}

#[test]
fn registers_style_virtual_modules() {
  let (context, plugin) = make_plugin("");
  let path = fixture_path("scoped-style.vue");
  let code = transform_fixture(&plugin, &context, "scoped-style.vue");
  // The main output must prepend a virtual style import.
  assert!(
    code.contains("?vue&type=style"),
    "expected virtual style import, got:\n{code}"
  );

  // The style virtual module id should be served by `load`.
  let virtual_id = format!("{path}?vue&type=style&idx=0&lang=css&scoped=true");
  let load_param = PluginLoadHookParam {
    module_id: virtual_id.clone(),
    resolved_path: &path,
    query: vec![("vue".to_string(), "".to_string())],
    meta: Default::default(),
  };
  let hook_ctx = PluginHookContext {
    caller: None,
    meta: Default::default(),
  };
  let res = plugin
    .load(&load_param, &context, &hook_ctx)
    .unwrap()
    .expect("style virtual module should be registered");
  assert!(matches!(res.module_type, ModuleType::Css));
  assert!(res.content.contains("color"));
}

#[test]
fn style_virtual_module_id_format_is_stable() {
  let (context, plugin) = make_plugin("");
  let path = fixture_path("scss-style.vue");
  let code = transform_fixture(&plugin, &context, "scss-style.vue");
  let expected_id = format!("{path}?vue&type=style&idx=0&lang=scss&scoped=true");
  assert!(
    code.contains("?vue&type=style&idx=0&lang=scss&scoped=true"),
    "expected main output to import {expected_id}, got:\n{code}"
  );

  let load_param = PluginLoadHookParam {
    module_id: expected_id,
    resolved_path: &path,
    query: vec![("vue".to_string(), "".to_string())],
    meta: Default::default(),
  };
  let hook_ctx = PluginHookContext {
    caller: None,
    meta: Default::default(),
  };
  assert!(plugin
    .load(&load_param, &context, &hook_ctx)
    .unwrap()
    .is_some());
}

#[test]
fn scss_styles_get_custom_module_type() {
  let (context, plugin) = make_plugin("");
  let path = fixture_path("scss-style.vue");
  transform_fixture(&plugin, &context, "scss-style.vue");

  let virtual_id = format!("{path}?vue&type=style&idx=0&lang=scss&scoped=true");
  let load_param = PluginLoadHookParam {
    module_id: virtual_id,
    resolved_path: &path,
    query: vec![("vue".to_string(), "".to_string())],
    meta: Default::default(),
  };
  let hook_ctx = PluginHookContext {
    caller: None,
    meta: Default::default(),
  };
  let res = plugin
    .load(&load_param, &context, &hook_ctx)
    .unwrap()
    .expect("scss style virtual module should be registered");
  assert!(
    matches!(&res.module_type, ModuleType::Custom(t) if t == "scss"),
    "scss style block should be tagged as Custom(\"scss\"), got {:?}",
    res.module_type
  );
}

#[test]
fn descriptor_cache_is_replaced_on_repeated_transform() {
  let (context, plugin) = make_plugin("");
  let path = fixture_path("scoped-style.vue");

  let first_param = PluginTransformHookParam {
    module_id: path.clone(),
    content: r#"<template><p class="one">one</p></template><style scoped>.one{color:red}</style>"#
      .to_string(),
    module_type: ModuleType::Custom("vue".to_string()),
    resolved_path: &path,
    query: vec![],
    meta: Default::default(),
    source_map_chain: vec![],
  };
  plugin
    .transform(&first_param, &context)
    .expect("first transform ok")
    .expect("first transform returns Some");
  let first_descriptor = plugin
    .cached_descriptor_for_test(&path)
    .expect("descriptor cached after first transform");

  let second_param = PluginTransformHookParam {
    content: r#"<template><p class="one">two</p></template><style scoped>.one{color:blue}</style>"#
      .to_string(),
    ..first_param
  };
  plugin
    .transform(&second_param, &context)
    .expect("second transform ok")
    .expect("second transform returns Some");
  let second_descriptor = plugin
    .cached_descriptor_for_test(&path)
    .expect("descriptor cached after second transform");

  assert_eq!(second_descriptor.styles.len(), 1);
  assert_ne!(first_descriptor.source_hash, second_descriptor.source_hash);
  assert_ne!(
    first_descriptor.styles[0].content_hash,
    second_descriptor.styles[0].content_hash
  );
}

#[test]
fn registers_custom_block_virtual_modules() {
  let (context, plugin) = make_plugin("");
  let path = fixture_path("custom-block.vue");
  let code = transform_fixture(&plugin, &context, "custom-block.vue");
  assert!(
    code.contains("?vue&type=custom&idx=0&block=i18n&lang=i18n"),
    "expected custom block import, got:\n{code}"
  );

  let virtual_id = format!("{path}?vue&type=custom&idx=0&block=i18n&lang=i18n");
  let load_param = PluginLoadHookParam {
    module_id: virtual_id,
    resolved_path: &path,
    query: vec![("vue".to_string(), "".to_string())],
    meta: Default::default(),
  };
  let hook_ctx = PluginHookContext {
    caller: None,
    meta: Default::default(),
  };
  let res = plugin
    .load(&load_param, &context, &hook_ctx)
    .unwrap()
    .expect("custom block virtual module should be registered");
  assert!(
    matches!(&res.module_type, ModuleType::Custom(t) if t == "i18n"),
    "custom block should be tagged as Custom(\"i18n\"), got {:?}",
    res.module_type
  );
  assert!(res.content.contains("\"hello\""));
}

#[test]
fn source_map_option_requests_fervid_source_map() {
  let (context, plugin) = make_plugin(r#"{"sourceMap":true}"#);
  let result = transform_fixture_result(&plugin, &context, "basic.vue");
  assert!(
    result
      .source_map
      .as_deref()
      .is_some_and(|map| map.contains("basic.vue")),
    "expected source map to mention basic.vue, got {:?}",
    result.source_map
  );
}

#[test]
fn ce_vue_suffix_triggers_custom_element_compilation() {
  // We can't easily peek at fervid's `is_ce` flag from the outside, but we
  // can at least assert the transform succeeds and produces some output.
  let (context, plugin) = make_plugin("");
  let code = transform_fixture(&plugin, &context, "custom-element.ce.vue");
  assert!(!code.is_empty());
}

#[test]
fn respects_exclude_filter() {
  let (context, plugin) = make_plugin(r#"{"exclude":["fixtures/basic\\.vue$"]}"#);
  let path = fixture_path("basic.vue");
  let load_param = PluginLoadHookParam {
    module_id: path.clone(),
    resolved_path: &path,
    query: vec![],
    meta: Default::default(),
  };
  let hook_ctx = PluginHookContext {
    caller: None,
    meta: Default::default(),
  };
  // Excluded -> load should return None and fall through to other plugins.
  assert!(plugin
    .load(&load_param, &context, &hook_ctx)
    .unwrap()
    .is_none());
}

#[test]
fn config_hook_injects_vue_define_flags() {
  let (_context, plugin) = make_plugin("");
  let mut config = Config::default();
  plugin.config(&mut config).expect("config hook ok");
  assert!(config.define.contains_key("__VUE_OPTIONS_API__"));
  assert!(config.define.contains_key("__VUE_PROD_DEVTOOLS__"));
  assert!(config
    .define
    .contains_key("__VUE_PROD_HYDRATION_MISMATCH_DETAILS__"));
  assert!(config.resolve.dedupe.iter().any(|d| d == "vue"));
}

#[test]
fn config_hook_skips_dedupe_for_node_target() {
  use farmfe_core::config::TargetEnv;
  let (_context, plugin) = make_plugin("");
  let mut config = Config::default();
  config.output.target_env = TargetEnv::Node;
  plugin.config(&mut config).expect("config hook ok");
  assert!(
    !config.resolve.dedupe.iter().any(|d| d == "vue"),
    "should not dedupe vue when targeting node"
  );
}

#[test]
fn config_hook_preserves_user_defines() {
  let (_context, plugin) = make_plugin("");
  let mut config = Config::default();
  config.define.insert(
    "__VUE_OPTIONS_API__".to_string(),
    farmfe_core::serde_json::Value::Bool(false),
  );
  plugin.config(&mut config).expect("config hook ok");
  // The user's `false` must survive — plugin only inserts when absent.
  assert_eq!(
    config.define.get("__VUE_OPTIONS_API__"),
    Some(&farmfe_core::serde_json::Value::Bool(false))
  );
}
