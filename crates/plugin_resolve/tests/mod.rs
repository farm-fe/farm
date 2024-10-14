use std::{collections::HashMap, sync::Arc};

use farmfe_core::{
  config::{AliasItem, Config, ResolveConfig, StringOrRegex},
  context::CompilationContext,
  plugin::ResolveKind, regex::Regex,
};
use farmfe_plugin_resolve::resolver::{ResolveOptions, Resolver};
use farmfe_testing_helpers::fixture;

#[test]
fn resolve_relative_specifier_without_extension() {
  fixture(
    "tests/fixtures/resolve-relative-specifier/**/index.*",
    |file, _| {
      let resolver = Resolver::new();
      let cwd = file.parent().unwrap().to_path_buf();

      let resolved = resolver.resolve(
        "./index",
        cwd.clone(),
        &ResolveKind::Entry(String::new()),
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd.join("index.ts").to_string_lossy().to_string()
      );
    },
  );
}

#[test]
fn resolve_relative_specifier_with_extension() {
  fixture(
    "tests/fixtures/resolve-relative-specifier/**/index.*",
    |file, _| {
      let resolver = Resolver::new();
      let cwd = file.parent().unwrap().to_path_buf();

      let resolved = resolver.resolve(
        "./index.html",
        cwd.clone(),
        &ResolveKind::Entry(String::new()),
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_none());

      let resolved = resolver.resolve(
        "./index.ts",
        cwd.clone(),
        &ResolveKind::Entry(String::new()),
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd.join("index.ts").to_string_lossy().to_string()
      );
    },
  );
}

#[test]
fn resolve_node_modules_normal() {
  farmfe_testing_helpers::fixture!(
    "tests/fixtures/resolve-node-modules/normal/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "pkg-a",
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
          .join("pkg-a")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
      assert!(!resolved.external);
      assert!(!resolved.side_effects);

      let resolved = resolver.resolve(
        "pkg-a/index.js",
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
          .join("pkg-a")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
      assert!(!resolved.external);
      assert!(!resolved.side_effects);

      let resolved = resolver.resolve(
        "pkg-a/lib",
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
          .join("pkg-a")
          .join("lib")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
      assert!(!resolved.external);
      assert!(resolved.side_effects);

      let resolved = resolver.resolve(
        "pkg-b",
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
          .join("pkg-b")
          .join("es")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
      assert!(!resolved.external);
      assert!(resolved.side_effects);

      let resolved = resolver.resolve(
        "dir-main",
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
          .join("dir-main")
          .join("lib")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
      assert!(!resolved.side_effects);
    }
  );
}

#[test]
fn resolve_alias() {
  fixture("tests/fixtures/resolve-alias/index.ts", |file, _| {
    let cwd = file.parent().unwrap().to_path_buf();
    let resolver = Resolver::new();
    let context = Arc::new(
      CompilationContext::new(
        Config {
          resolve: Box::new(ResolveConfig {
            alias: vec![
              AliasItem::Complex {
                find: StringOrRegex::String("@".to_string()),
                replacement: cwd.to_string_lossy().to_string(),
              },
              AliasItem::Complex {
                find: StringOrRegex::String("/@".to_string()),
                replacement: cwd.to_string_lossy().to_string(),
              },
              AliasItem::Complex {
                find: StringOrRegex::String("@/components".to_string()),
                replacement: cwd.join("components").to_string_lossy().to_string(),
              },
              AliasItem::Complex {
                find: StringOrRegex::Regex(Regex::new("$__farm_regex:^/(utils)$").unwrap()),
                replacement: cwd.join("$1").to_string_lossy().to_string(),
              },
            ],
            ..Default::default()
          }),
          ..Default::default()
        },
        vec![],
      )
      .unwrap(),
    );

    let resolved = resolver.resolve(
      "@/pages/a",
      cwd.clone(),
      &ResolveKind::Import,
      &ResolveOptions::default(),
      &context,
    );
    assert!(resolved.is_some());
    let resolved = resolved.unwrap();

    assert_eq!(
      resolved.resolved_path,
      cwd
        .join("pages")
        .join("a.tsx")
        .to_string_lossy()
        .to_string()
    );

    let resolved = resolver.resolve(
      "/@/pages/a",
      cwd.clone(),
      &ResolveKind::Import,
      &ResolveOptions::default(),
      &context,
    );
    assert!(resolved.is_some());
    let resolved = resolved.unwrap();

    assert_eq!(
      resolved.resolved_path,
      cwd
        .join("pages")
        .join("a.tsx")
        .to_string_lossy()
        .to_string()
    );

    let resolved = resolver.resolve(
      "@/components/button",
      cwd.clone(),
      &ResolveKind::Import,
      &ResolveOptions::default(),
      &context,
    );

    assert!(resolved.is_some());
    let resolved = resolved.unwrap();

    assert_eq!(
      resolved.resolved_path,
      cwd
        .join("components")
        .join("button.tsx")
        .to_string_lossy()
        .to_string()
    );

    let resolved = resolver.resolve(
      "/utils",
      cwd.clone(),
      &ResolveKind::Import,
      &ResolveOptions::default(),
      &context,
    );

    assert!(resolved.is_some());
    let resolved = resolved.unwrap();

    assert_eq!(
      resolved.resolved_path,
      cwd
        .join("utils")
        .join("index.ts")
        .to_string_lossy()
        .to_string()
    );
  });
}

#[test]
fn resolve_dot() {
  fixture!("tests/fixtures/resolve-dot/index.ts", |file, _| {
    let cwd = file.parent().unwrap().to_path_buf();
    let resolver = Resolver::new();

    let resolved = resolver.resolve(
      ".",
      cwd.clone(),
      &ResolveKind::Import,
      &ResolveOptions::default(),
      &Arc::new(CompilationContext::default()),
    );
    assert!(resolved.is_some());
    let resolved = resolved.unwrap();

    assert_eq!(
      resolved.resolved_path,
      cwd.join("index.ts").to_string_lossy().to_string()
    );
  });
}

#[test]
fn resolve_double_dot() {
  fixture!(
    "tests/fixtures/resolve-double-dot/lib/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "..",
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
          .join("index.ts")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_absolute_specifier() {
  fixture!(
    "tests/fixtures/resolve-absolute-specifier/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        file.to_str().unwrap(),
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();

      assert_eq!(resolved.resolved_path, file.to_string_lossy().to_string());

      let resolved = resolver.resolve(
        cwd.join("lib").to_str().unwrap(),
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
          .join("lib")
          .join("index.ts")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_package_dir() {
  fixture!(
    "tests/fixtures/resolve-node-modules/package_dir/entry.js",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        cwd.to_str().unwrap(),
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());

      let resolved = resolved.unwrap();

      assert_eq!(resolved.resolved_path, file.to_string_lossy().to_string());
    }
  );
}

#[test]
fn resolve_package_end_with_js() {
  fixture!(
    "tests/fixtures/resolve-node-modules/issue-983/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "bn.js",
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
          .join("bn.js")
          .join("lib")
          .join("bn.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_package_subpath() {
  fixture!(
    "tests/fixtures/resolve-node-modules/issue-1402/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "bn.js/lib",
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
          .join("bn.js")
          .join("lib")
          .join("index.esm.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}
