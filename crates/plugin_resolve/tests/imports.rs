use std::sync::Arc;

use farmfe_core::{
  config::{Config, OutputConfig, TargetEnv},
  context::CompilationContext,
  plugin::ResolveKind,
};
use farmfe_plugin_resolve::resolver::{ResolveOptions, Resolver};
use farmfe_testing_helpers::fixture;

#[test]
fn resolve_imports_basic() {
  fixture!(
    // TODO node environment
    "tests/fixtures/resolve-node-modules/imports/node_modules/chalk/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      // Parsing packages in node_modules
      let resolved = resolver.resolve(
        "#ansi-styles",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("source")
          .join("vendor")
          .join("ansi-styles")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_imports_deep() {
  fixture!(
    "tests/fixtures/resolve-node-modules/imports/node_modules/chalk/source/index.js",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      // Parsing packages in node_modules
      let resolved = resolver.resolve(
        "#ansi-styles",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("vendor")
          .join("ansi-styles")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_imports_replace_object() {
  fixture!(
    // TODO node environment
    "tests/fixtures/resolve-node-modules/imports/node_modules/chalk/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      // Parsing packages in node_modules
      let resolved = resolver.resolve(
        "#supports-color",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("source")
          .join("vendor")
          .join("supports-color")
          .join("browser.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_imports_replace_deps() {
  fixture!(
    // TODO node environment
    "tests/fixtures/resolve-node-modules/imports/node_modules/chalk/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      // import resolve other deps like `"#ansi-styles-execa": "execa"`
      let resolved = resolver.resolve(
        "#ansi-styles-execa",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .parent()
          .unwrap()
          .to_path_buf()
          .join("execa")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_imports_target_browser() {
  fixture!(
    // TODO node environment
    "tests/fixtures/resolve-node-modules/imports/node_modules/chalk/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let context = CompilationContext::new(
        Config {
          output: Box::new(OutputConfig {
            target_env: TargetEnv::Browser,
            ..Default::default()
          }),
          ..Default::default()
        },
        vec![],
      )
      .unwrap();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "#supports-color",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(context),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("source")
          .join("vendor")
          .join("supports-color")
          .join("browser.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_imports_target_node() {
  fixture!(
    // TODO node environment
    "tests/fixtures/resolve-node-modules/imports/node_modules/chalk/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();
      let context = CompilationContext::new(
        Config {
          output: Box::new(OutputConfig {
            target_env: TargetEnv::Node,
            ..Default::default()
          }),
          ..Default::default()
        },
        vec![],
      )
      .unwrap();

      let resolved = resolver.resolve(
        "#supports-color",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(context),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("source")
          .join("vendor")
          .join("supports-color")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}
