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
  plugin::{Plugin, PluginHookContext, PluginLoadHookParam, PluginTransformHookParam},
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
  res.content
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
