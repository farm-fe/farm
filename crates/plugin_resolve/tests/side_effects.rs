use std::sync::Arc;

use farmfe_core::{context::CompilationContext, plugin::ResolveKind};
use farmfe_plugin_resolve::resolver::{ResolveOptions, Resolver};
use farmfe_testing_helpers::fixture;

#[test]
fn resolve_side_effects_entry() {
  fixture("tests/fixtures/side_effects/index.ts", |file, _| {
    let resolver = Resolver::new();
    let cwd = file.parent().unwrap().to_path_buf();

    let resolved = resolver.resolve(
      "array",
      cwd.clone(),
      &ResolveKind::Entry(String::new()),
      &ResolveOptions::default(),
      &Arc::new(CompilationContext::default()),
    );
    assert!(resolved.is_some());
    let resolved = resolved.unwrap();
    assert_eq!(
      resolved.resolved_path,
      cwd
        .join("node_modules")
        .join("array")
        .join("index.js")
        .to_string_lossy()
        .to_string()
    );
    assert_eq!(resolved.side_effects, false);
  });
}

#[test]
fn resolve_side_effects_subpath() {
  fixture!("tests/fixtures/side_effects/index.ts", |file, _| {
    let resolver = Resolver::new();
    let cwd = file.parent().unwrap().to_path_buf();

    let resolved = resolver.resolve(
      "array/index.css",
      cwd.clone(),
      &ResolveKind::Entry(String::new()),
      &ResolveOptions::default(),
      &Arc::new(CompilationContext::default()),
    );
    assert!(resolved.is_some());
    let resolved = resolved.unwrap();
    assert_eq!(
      resolved.resolved_path,
      cwd
        .join("node_modules")
        .join("array")
        .join("index.css")
        .to_string_lossy()
        .to_string()
    );
    assert_eq!(resolved.side_effects, true);
  });
}

#[test]
fn resolve_side_effects_bool() {
  fixture!("tests/fixtures/side_effects/index.ts", |file, _| {
    let resolver = Resolver::new();
    let cwd = file.parent().unwrap().to_path_buf();

    let resolved = resolver.resolve(
      "bool",
      cwd.clone(),
      &ResolveKind::Entry(String::new()),
      &ResolveOptions::default(),
      &Arc::new(CompilationContext::default()),
    );
    assert!(resolved.is_some());
    let resolved = resolved.unwrap();
    assert_eq!(
      resolved.resolved_path,
      cwd
        .join("node_modules")
        .join("bool")
        .join("index.js")
        .to_string_lossy()
        .to_string()
    );
    assert_eq!(resolved.side_effects, true);
  });
}
