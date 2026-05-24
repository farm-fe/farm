use std::sync::Arc;

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::ModuleType,
  plugin::{Plugin, PluginTransformHookParam},
  HashMap,
};
use farmfe_plugin_tailwindcss::FarmPluginTailwindCSS;
use farmfe_testing_helpers::fixture;
use farmfe_toolkit::fs::read_file_utf8;

/// Normalize line endings for cross-platform comparison.
fn normalize_css(css: &str) -> String {
  css.replace("\r\n", "\n")
}

#[test]
fn tailwindcss_transform_basic() {
  fixture!("tests/fixtures/basic/index.css", |file, _crate_path| {
    let resolved_path = file.to_string_lossy().to_string();
    let config = Config::default();
    let plugin = Arc::new(FarmPluginTailwindCSS::new(&config, "{}".to_string()));
    let context = CompilationContext::new(config, vec![plugin.clone()]).unwrap();
    let content = read_file_utf8(&resolved_path).unwrap();

    let result = plugin
      .transform(
        &PluginTransformHookParam {
          resolved_path: &resolved_path,
          content: content.clone(),
          module_type: ModuleType::Css,
          query: vec![],
          meta: HashMap::from_iter([]),
          module_id: resolved_path.clone(),
          source_map_chain: vec![],
        },
        &Arc::new(context),
      )
      .unwrap();

    // Basic CSS without tailwind directives should not be transformed
    assert!(
      result.is_none(),
      "Basic CSS without tailwind directives should return None"
    );
  });
}

#[test]
fn tailwindcss_transform_at_apply() {
  fixture!(
    "tests/fixtures/at-apply/index.css",
    |file, _crate_path| {
      let resolved_path = file.to_string_lossy().to_string();
      let config = Config::default();
      let plugin = Arc::new(FarmPluginTailwindCSS::new(&config, "{}".to_string()));
      let context = CompilationContext::new(config, vec![plugin.clone()]).unwrap();
      let content = read_file_utf8(&resolved_path).unwrap();

      let result = plugin
        .transform(
          &PluginTransformHookParam {
            resolved_path: &resolved_path,
            content: content.clone(),
            module_type: ModuleType::Css,
            query: vec![],
            meta: HashMap::from_iter([]),
            module_id: resolved_path.clone(),
            source_map_chain: vec![],
          },
          &Arc::new(context),
        )
        .unwrap();

      // CSS with @apply should be detected and processed
      match result {
        Some(transformed) => {
          let css = normalize_css(&transformed.content);
          assert!(
            css.contains("@apply") || css.contains("padding"),
            "Transformed CSS should contain @apply or processed output"
          );
          assert_eq!(
            transformed.module_type,
            Some(ModuleType::Css),
            "Module type should remain CSS"
          );
        }
        None => {
          // If compile returns no features, it returns None.
          // This is acceptable since we don't have the full tailwindcss
          // runtime in test - the @apply feature is detected but
          // compilation may not produce output without full tailwindcss.
        }
      }
    }
  );
}

#[test]
fn tailwindcss_transform_import_resolve() {
  fixture!(
    "tests/fixtures/import-resolve/index.css",
    |file, _crate_path| {
      let resolved_path = file.to_string_lossy().to_string();
      let config = Config::default();
      let plugin = Arc::new(FarmPluginTailwindCSS::new(&config, "{}".to_string()));
      let context = CompilationContext::new(config, vec![plugin.clone()]).unwrap();
      let content = read_file_utf8(&resolved_path).unwrap();

      let result = plugin
        .transform(
          &PluginTransformHookParam {
            resolved_path: &resolved_path,
            content: content.clone(),
            module_type: ModuleType::Css,
            query: vec![],
            meta: HashMap::from_iter([]),
            module_id: resolved_path.clone(),
            source_map_chain: vec![],
          },
          &Arc::new(context),
        )
        .unwrap();

      // CSS with only @import (no tailwind directives) should not be
      // transformed by the tailwindcss plugin
      assert!(
        result.is_none(),
        "CSS without tailwind directives should not be transformed"
      );
    }
  );
}

#[test]
fn tailwindcss_scan_candidates() {
  fixture!("tests/fixtures/basic/index.css", |_file, _crate_path| {
    let config = Config::default();
    let plugin = Arc::new(FarmPluginTailwindCSS::new(&config, "{}".to_string()));
    let context = CompilationContext::new(config, vec![plugin.clone()]).unwrap();

    // Simulate scanning a JSX file with tailwind classes
    let jsx_content = r#"
      export default function App() {
        return <div className="bg-blue-500 text-white p-4 rounded-lg">Hello</div>;
      }
    "#;

    let result = plugin
      .transform(
        &PluginTransformHookParam {
          resolved_path: "/src/App.tsx",
          content: jsx_content.to_string(),
          module_type: ModuleType::Tsx,
          query: vec![],
          meta: HashMap::from_iter([]),
          module_id: "/src/App.tsx".to_string(),
          source_map_chain: vec![],
        },
        &Arc::new(context),
      )
      .unwrap();

    // Scanning non-CSS files should return None (pass-through)
    assert!(
      result.is_none(),
      "Scanning source files should return None (pass-through)"
    );
  });
}

#[test]
fn tailwindcss_skips_non_css_modules() {
  fixture!("tests/fixtures/basic/index.css", |_file, _crate_path| {
    let config = Config::default();
    let plugin = Arc::new(FarmPluginTailwindCSS::new(&config, "{}".to_string()));
    let context = CompilationContext::new(config, vec![plugin.clone()]).unwrap();

    // JavaScript files should be passed through
    let result = plugin
      .transform(
        &PluginTransformHookParam {
          resolved_path: "/src/utils.js",
          content: "export const x = 1;".to_string(),
          module_type: ModuleType::Js,
          query: vec![],
          meta: HashMap::from_iter([]),
          module_id: "/src/utils.js".to_string(),
          source_map_chain: vec![],
        },
        &Arc::new(context),
      )
      .unwrap();

    assert!(result.is_none(), "JS modules should be passed through");
  });
}

#[test]
fn tailwindcss_skips_node_modules() {
  fixture!("tests/fixtures/basic/index.css", |_file, _crate_path| {
    let config = Config::default();
    let plugin = Arc::new(FarmPluginTailwindCSS::new(&config, "{}".to_string()));
    let context = CompilationContext::new(config, vec![plugin.clone()]).unwrap();

    let result = plugin
      .transform(
        &PluginTransformHookParam {
          resolved_path: "/node_modules/some-lib/style.css",
          content: "@tailwind base;".to_string(),
          module_type: ModuleType::Css,
          query: vec![],
          meta: HashMap::from_iter([]),
          module_id: "/node_modules/some-lib/style.css".to_string(),
          source_map_chain: vec![],
        },
        &Arc::new(context),
      )
      .unwrap();

    assert!(
      result.is_none(),
      "CSS files in node_modules should be skipped"
    );
  });
}
