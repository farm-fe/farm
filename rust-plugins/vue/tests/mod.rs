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
    PluginTransformHookResult, PluginUpdateModulesHookParam, UpdateType,
  },
};

use farmfe_plugin_vue::FarmPluginVue;

#[derive(Debug)]
struct ScssToCssPlugin;

impl Plugin for ScssToCssPlugin {
  fn name(&self) -> &str {
    "ScssToCssPlugin"
  }

  fn transform(
    &self,
    param: &PluginTransformHookParam,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<PluginTransformHookResult>> {
    if matches!(&param.module_type, ModuleType::Custom(module_type) if module_type == "scss") {
      return Ok(Some(PluginTransformHookResult {
        content: ".card{background:#40916c}.card button{color:white}".to_string(),
        module_type: Some(ModuleType::Css),
        source_map: None,
        ignore_previous_source_map: false,
      }));
    }

    Ok(None)
  }
}

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

fn write_temp_vue(name: &str, content: &str) -> String {
  let dir = std::env::temp_dir().join(format!("farm-plugin-vue-{}-{name}", std::process::id()));
  std::fs::create_dir_all(&dir).expect("temp dir created");
  let path = dir.join("Component.vue");
  std::fs::write(&path, content).expect("temp vue fixture written");
  path.to_string_lossy().into_owned()
}

fn transform_content(
  plugin: &FarmPluginVue,
  context: &Arc<CompilationContext>,
  path: &str,
  content: &str,
) -> String {
  let transform_param = PluginTransformHookParam {
    module_id: path.to_string(),
    content: content.to_string(),
    module_type: ModuleType::Custom("vue".to_string()),
    resolved_path: path,
    query: vec![],
    meta: Default::default(),
    source_map_chain: vec![],
  };
  plugin
    .transform(&transform_param, context)
    .expect("transform returns Ok")
    .expect("transform returns Some")
    .content
}

fn assert_single_updated_path(params: &PluginUpdateModulesHookParam, expected_path: &str) {
  assert_eq!(params.paths.len(), 1);
  assert_eq!(params.paths[0].0, expected_path);
  assert!(matches!(params.paths[0].1, UpdateType::Updated));
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
fn compiles_deferred_import_evaluation_script() {
  let (context, plugin) = make_plugin(
    r#"{"script":{"babelParserPlugins":["deferredImportEvaluation"]}}"#,
  );
  let path = write_temp_vue(
    "defer",
    r#"<script setup lang="ts">
import defer * as deferredFeature from "./deferred-feature";

const message = deferredFeature.message;
</script>

<template>
  <div>{{ message }}</div>
</template>
"#,
  );

  let code = transform_content(
    &plugin,
    &context,
    &path,
    &std::fs::read_to_string(&path).expect("temp vue fixture is readable"),
  );

  assert!(
    code.contains(r#"import defer * as deferredFeature from "./deferred-feature""#),
    "expected compiled output to preserve deferred import, got:\n{code}"
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
  let descriptor = plugin
    .cached_descriptor_for_test(&path)
    .expect("descriptor cached after transform");
  let virtual_id = descriptor.styles[0].virtual_id.clone();
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
  let descriptor = plugin
    .cached_descriptor_for_test(&path)
    .expect("descriptor cached after transform");
  let expected_id = descriptor.styles[0].virtual_id.clone();
  assert!(
    code.contains("?vue&type=style&idx=0&lang=scss&scoped=true&scopeId=data-v-"),
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
  let descriptor = plugin
    .cached_descriptor_for_test(&path)
    .expect("descriptor cached after transform");

  let virtual_id = descriptor.styles[0].virtual_id.clone();
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
    .expect("scss style virtual module should be registered");
  assert!(
    matches!(&res.module_type, ModuleType::Custom(t) if t == "scss"),
    "scss style block should be tagged as Custom(\"scss\"), got {:?}",
    res.module_type
  );
}

#[test]
fn descriptor_cache_threads_owner_and_scope_metadata_for_preprocessed_styles() {
  let (context, plugin) = make_plugin("");
  let path = fixture_path("scss-style.vue");
  transform_fixture(&plugin, &context, "scss-style.vue");

  let descriptor = plugin
    .cached_descriptor_for_test(&path)
    .expect("descriptor cached after transform");
  let style = descriptor.styles.first().expect("scss style descriptor");

  assert_eq!(style.lang, "scss");
  assert!(style.scoped);
  assert_eq!(style.owner_module_id, path.replace('\\', "/"));
  assert_eq!(style.owner_resolved_path, path.replace('\\', "/"));
  assert_eq!(style.scope_id, format!("data-v-{}", descriptor.source_hash));
}

#[test]
fn scoped_preprocessor_styles_keep_preprocessor_module_type_with_scope_metadata() {
  let (context, plugin) = make_plugin("");
  let path = fixture_path("scss-style.vue");
  transform_fixture(&plugin, &context, "scss-style.vue");
  let descriptor = plugin
    .cached_descriptor_for_test(&path)
    .expect("descriptor cached after transform");

  let virtual_id = descriptor.styles[0].virtual_id.clone();
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
    .expect("scss style virtual module should be registered");

  assert!(matches!(&res.module_type, ModuleType::Custom(t) if t == "scss"));
  assert!(res.content.contains("$primary"));
  assert!(!res.content.contains("data-v-"));
  assert!(virtual_id.contains("&scopeId=data-v-"));
}

#[test]
fn scoped_preprocessor_styles_publish_component_scope_id() {
  let (context, plugin) = make_plugin("");
  let path = fixture_path("scss-style.vue");
  let code = transform_fixture(&plugin, &context, "scss-style.vue");
  let descriptor = plugin
    .cached_descriptor_for_test(&path)
    .expect("descriptor cached after transform");

  assert!(code.contains("__farm_vue_sfc.__scopeId"));
  assert!(code.contains(&format!("\"{}\"", descriptor.styles[0].scope_id)));
}

#[test]
fn dev_sfc_modules_self_accept_with_vue_hmr_runtime() {
  let (context, plugin) = make_plugin("");
  let code = transform_fixture(&plugin, &context, "basic.vue");

  assert!(code.contains("__farm_vue_sfc.__hmrId"));
  assert!(code.contains("window.__VUE_HMR_RUNTIME__.createRecord"));
  assert!(code.contains("import.meta.hot.accept"));
  assert!(code.contains("window.__VUE_HMR_RUNTIME__.reload"));
}

#[test]
fn scoped_preprocessor_styles_are_transformed_then_scoped_by_vue() {
  let config = Config::default();
  let plugin = FarmPluginVue::new(&config, "".to_string());
  let context = Arc::new(
    CompilationContext::new(config, vec![Arc::new(ScssToCssPlugin) as _]).expect("context created"),
  );
  let path = fixture_path("scss-style.vue");
  transform_fixture(&plugin, &context, "scss-style.vue");
  let descriptor = plugin
    .cached_descriptor_for_test(&path)
    .expect("descriptor cached after transform");

  let virtual_id = descriptor.styles[0].virtual_id.clone();
  let load_param = PluginLoadHookParam {
    module_id: virtual_id.clone(),
    resolved_path: &virtual_id,
    query: vec![
      ("vue".to_string(), "".to_string()),
      ("type".to_string(), "style".to_string()),
      ("idx".to_string(), "0".to_string()),
      ("lang".to_string(), "scss".to_string()),
      ("scoped".to_string(), "true".to_string()),
      ("scopeId".to_string(), descriptor.styles[0].scope_id.clone()),
    ],
    meta: Default::default(),
  };
  let hook_ctx = PluginHookContext {
    caller: None,
    meta: Default::default(),
  };
  let res = plugin
    .load(&load_param, &context, &hook_ctx)
    .unwrap()
    .expect("scss style virtual module should be transformed");

  assert_eq!(res.module_type, ModuleType::Css);
  assert!(res
    .content
    .contains(&format!(".card[{}]", descriptor.styles[0].scope_id)));
  assert!(res
    .content
    .contains(&format!("button[{}]", descriptor.styles[0].scope_id)));
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
fn update_modules_invalidates_unscoped_native_style_virtual_module_for_style_only_changes() {
  let (context, plugin) = make_plugin("");
  let first = r#"<template><p class="one">same</p></template><style>.one{color:red}</style>"#;
  let second = r#"<template><p class="one">same</p></template><style>.one{color:blue}</style>"#;
  let path = write_temp_vue("style-only", first);
  transform_content(&plugin, &context, &path, first);
  std::fs::write(&path, second).expect("updated temp vue fixture written");

  let mut params = PluginUpdateModulesHookParam {
    paths: vec![(path.clone(), UpdateType::Updated)],
  };
  plugin
    .update_modules(&mut params, &context)
    .expect("update_modules ok");

  let expected_id = format!(
    "{}?vue&type=style&idx=0&lang=css&scoped=false",
    path.replace('\\', "/")
  );
  assert_single_updated_path(&params, &expected_id);

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
  let res = plugin
    .load(&load_param, &context, &hook_ctx)
    .unwrap()
    .expect("updated style virtual module should be registered");
  assert!(res.content.contains("blue"));
}

#[test]
fn load_virtual_style_rehydrates_registry_when_descriptor_cache_is_empty() {
  let (context, plugin) = make_plugin("");
  let content = r#"<template><p class="one">cached</p></template><style>.one{color:red}</style>"#;
  let path = write_temp_vue("rehydrate-load", content);
  let virtual_id = format!(
    "{}?vue&type=style&idx=0&lang=css&scoped=false",
    path.replace('\\', "/")
  );
  let load_param = PluginLoadHookParam {
    module_id: virtual_id,
    resolved_path: &path,
    query: vec![
      ("vue".to_string(), "".to_string()),
      ("type".to_string(), "style".to_string()),
      ("idx".to_string(), "0".to_string()),
      ("lang".to_string(), "css".to_string()),
      ("scoped".to_string(), "false".to_string()),
    ],
    meta: Default::default(),
  };
  let hook_ctx = PluginHookContext {
    caller: None,
    meta: Default::default(),
  };

  let res = plugin
    .load(&load_param, &context, &hook_ctx)
    .unwrap()
    .expect("virtual style should be rehydrated from the SFC file");

  assert_eq!(res.module_type, ModuleType::Css);
  assert!(res.content.contains("color:red"));
}

#[test]
fn update_modules_rehydrates_descriptor_when_cache_is_empty() {
  let (context, plugin) = make_plugin("");
  let content = r#"<template><p class="one">cached</p></template><style>.one{color:red}</style>"#;
  let path = write_temp_vue("rehydrate-update", content);
  let mut params = PluginUpdateModulesHookParam {
    paths: vec![(path.clone(), UpdateType::Updated)],
  };

  plugin
    .update_modules(&mut params, &context)
    .expect("update_modules ok");

  assert_single_updated_path(&params, &path.replace('\\', "/"));
  let descriptor = plugin
    .cached_descriptor_for_test(&path.replace('\\', "/"))
    .expect("descriptor should be rehydrated during update_modules");
  assert_eq!(descriptor.styles.len(), 1);
}

#[test]
fn update_modules_falls_back_to_main_sfc_for_scoped_style_changes() {
  let (context, plugin) = make_plugin("");
  let first =
    r#"<template><p class="one">same</p></template><style scoped>.one{color:red}</style>"#;
  let second =
    r#"<template><p class="one">same</p></template><style scoped>.one{color:blue}</style>"#;
  let path = write_temp_vue("scoped-style-only", first);
  transform_content(&plugin, &context, &path, first);
  std::fs::write(&path, second).expect("updated temp vue fixture written");

  let mut params = PluginUpdateModulesHookParam {
    paths: vec![(path.clone(), UpdateType::Updated)],
  };
  plugin
    .update_modules(&mut params, &context)
    .expect("update_modules ok");

  assert_single_updated_path(&params, &path.replace('\\', "/"));
}

#[test]
fn update_modules_falls_back_to_main_sfc_for_template_changes() {
  let (context, plugin) = make_plugin("");
  let first = r#"<template><p class="one">one</p></template><style scoped>.one{color:red}</style>"#;
  let second =
    r#"<template><p class="one">two</p></template><style scoped>.one{color:red}</style>"#;
  let path = write_temp_vue("template-change", first);
  transform_content(&plugin, &context, &path, first);
  std::fs::write(&path, second).expect("updated temp vue fixture written");

  let mut params = PluginUpdateModulesHookParam {
    paths: vec![(path.clone(), UpdateType::Updated)],
  };
  plugin
    .update_modules(&mut params, &context)
    .expect("update_modules ok");

  assert_single_updated_path(&params, &path.replace('\\', "/"));
}

#[test]
fn update_modules_fallback_uses_resolved_path_for_relative_update_paths() {
  let (context, plugin) = make_plugin("");
  let first = r#"<template><p class="one">one</p></template><style scoped>.one{color:red}</style>"#;
  let second =
    r#"<template><p class="one">two</p></template><style scoped>.one{color:red}</style>"#;
  let path = write_temp_vue("relative-template-change", first);
  let module_id = "src/views/HomeView.vue";

  let transform_param = PluginTransformHookParam {
    module_id: module_id.to_string(),
    content: first.to_string(),
    module_type: ModuleType::Custom("vue".to_string()),
    resolved_path: &path,
    query: vec![],
    meta: Default::default(),
    source_map_chain: vec![],
  };
  plugin
    .transform(&transform_param, &context)
    .expect("transform ok")
    .expect("transform returns Some");
  std::fs::write(&path, second).expect("updated temp vue fixture written");

  let mut params = PluginUpdateModulesHookParam {
    paths: vec![(module_id.to_string(), UpdateType::Updated)],
  };
  plugin
    .update_modules(&mut params, &context)
    .expect("update_modules ok");

  assert_single_updated_path(&params, &path.replace('\\', "/"));
}

#[test]
fn update_modules_invalidates_custom_block_virtual_module_for_custom_block_only_changes() {
  let (context, plugin) = make_plugin("");
  let first = r#"<template><p>same</p></template><i18n>{"hello":"world"}</i18n>"#;
  let second = r#"<template><p>same</p></template><i18n>{"hello":"farm"}</i18n>"#;
  let path = write_temp_vue("custom-only", first);
  transform_content(&plugin, &context, &path, first);
  std::fs::write(&path, second).expect("updated temp vue fixture written");

  let mut params = PluginUpdateModulesHookParam {
    paths: vec![(path.clone(), UpdateType::Updated)],
  };
  plugin
    .update_modules(&mut params, &context)
    .expect("update_modules ok");

  let expected_id = format!(
    "{}?vue&type=custom&idx=0&block=i18n&lang=i18n",
    path.replace('\\', "/")
  );
  assert_single_updated_path(&params, &expected_id);
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
fn normalizes_windows_paths_for_virtual_style_imports() {
  let (context, plugin) = make_plugin("");
  let path = r"D:\a\farm\farm\examples\vue\src\App.vue".to_string();
  let transform_param = PluginTransformHookParam {
    module_id: path.clone(),
    content: r#"<template><div>Hello</div></template><style scoped>.hello{color:red}</style>"#
      .to_string(),
    module_type: ModuleType::Custom("vue".to_string()),
    resolved_path: &path,
    query: vec![],
    meta: Default::default(),
    source_map_chain: vec![],
  };

  let code = plugin
    .transform(&transform_param, &context)
    .expect("transform returns Ok")
    .expect("transform returns Some")
    .content;

  assert!(
    code.contains(r#"import "D:/a/farm/farm/examples/vue/src/App.vue?vue&type=style"#),
    "expected import path to use forward slashes, got:
{code}"
  );

  let descriptor = plugin
    .cached_descriptor_for_test(&path)
    .expect("descriptor cached after transform");
  let virtual_id = descriptor.styles[0].virtual_id.as_str();
  let load_param = PluginLoadHookParam {
    module_id: virtual_id.to_string(),
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
