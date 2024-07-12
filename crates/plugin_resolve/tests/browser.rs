use std::sync::Arc;

use farmfe_core::{
  config::{Config, OutputConfig, TargetEnv},
  context::CompilationContext,
  plugin::ResolveKind,
};
use farmfe_plugin_resolve::resolver::{ResolveOptions, Resolver};
use farmfe_testing_helpers::fixture;

/// See browser field spec (https://github.com/defunctzombie/package-browser-field-spec)

#[test]
fn resolve_browser_basic() {
  fixture!(
    "tests/fixtures/resolve-node-modules/browser/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "basic",
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
          .join("node_modules")
          .join("basic")
          .join("browser.js")
          .to_string_lossy()
          .to_string()
      );

      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "basic",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(
          CompilationContext::new(
            Config {
              output: Box::new(OutputConfig {
                target_env: farmfe_core::config::TargetEnv::Node,
                ..Default::default()
              }),
              ..Default::default()
            },
            vec![],
          )
          .unwrap(),
        ),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();

      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("basic")
          .join("main.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_browser_replace() {
  fixture!(
    "tests/fixtures/resolve-node-modules/browser/node_modules/replace/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "module-a",
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
          .join("shims")
          .join("module-a.js")
          .to_string_lossy()
          .to_string()
      );

      let resolved = resolver.resolve(
        "./only.js",
        cwd.join("server"),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());

      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("shims")
          .join("client-only.js")
          .to_string_lossy()
          .to_string()
      );

      // normal resolve
      let resolved = resolver.resolve(
        "./module-a.js",
        cwd.join("shims"),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();

      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("shims")
          .join("module-a.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_browser_ignore() {
  fixture!(
    "tests/fixtures/resolve-node-modules/browser/node_modules/ignore/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "module-a",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();

      assert_eq!(resolved.resolved_path, "module-a".to_string());
      assert!(resolved.external);

      let resolved = resolver.resolve(
        "./only.js",
        cwd.join("server"),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();

      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("shims")
          .join("server-only.js")
          .to_string_lossy()
          .to_string()
      );
      assert!(!resolved.external);
      assert!(resolved.side_effects);
    }
  );
}

#[test]
fn resolve_browser_target_env_node() {
  fixture!(
    "tests/fixtures/resolve-node-modules/browser/node_modules/replace/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "./only.js",
        cwd.join("server"),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(
          CompilationContext::new(
            Config {
              output: Box::new(OutputConfig {
                target_env: TargetEnv::Node,
                ..Default::default()
              }),
              ..Default::default()
            },
            vec![],
          )
          .unwrap(),
        ),
      );
      assert!(resolved.is_some());

      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("server")
          .join("only.js")
          .to_string_lossy()
          .to_string()
      );

      // normal resolve
      let resolved = resolver.resolve(
        "./module-a.js",
        cwd.join("shims"),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();

      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("shims")
          .join("module-a.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_browser_entry_replace() {
  fixture!(
    "tests/fixtures/resolve-node-modules/browser/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "entry-replace",
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
          .join("node_modules")
          .join("entry-replace")
          .join("lib")
          .join("browser.js")
          .to_string_lossy()
          .to_string()
      );

      let resolved1 = resolver.resolve(
        "priority",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved1.is_some());
      let resolved1 = resolved1.unwrap();

      assert_eq!(
        resolved1.resolved_path,
        cwd
          .join("node_modules")
          .join("priority")
          .join("lib")
          .join("index.mjs")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_browser_issue_941() {
  fixture!(
    "tests/fixtures/resolve-node-modules/browser/node_modules/issue-941/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "indexof",
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
          .join("node_modules")
          .join("component-indexof")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );

      let resolved1 = resolver.resolve(
        "component-indexof",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved1.is_some());
      let resolved1 = resolved1.unwrap();

      assert_eq!(
        resolved1.resolved_path,
        cwd
          .join("node_modules")
          .join("component-indexof")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_browser_issue_1403() {
  fixture!(
    "tests/fixtures/resolve-node-modules/browser/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "exports-pkg",
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
          .join("node_modules")
          .join("exports-pkg")
          .join("standalone.mjs")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}
