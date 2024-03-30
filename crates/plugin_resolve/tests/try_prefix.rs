use std::sync::Arc;

use farmfe_core::{context::CompilationContext, plugin::ResolveKind};
use farmfe_plugin_resolve::resolver::Resolver;
use farmfe_testing_helpers::fixture;

#[test]
fn try_prefix() {
  fixture!("tests/fixtures/try_prefix/prefix/index.ts", |file, _| {
    let resolver = Resolver::new();
    let cwd = file.parent().unwrap().to_path_buf();

    let resolved = resolver.resolve(
      "./a.ts",
      cwd.clone(),
      &ResolveKind::Entry(String::new()),
      &Some("_".to_string()),
      &Arc::new(CompilationContext::default()),
    );
    assert!(resolved.is_some());
    let resolved = resolved.unwrap();

    assert_eq!(
      resolved.resolved_path,
      cwd.join("_a.ts").to_string_lossy().to_string()
    );
  });
}

#[test]
fn try_prefix_priority() {
  fixture!("tests/fixtures/try_prefix/prefix_priority/index.ts", |file, _| {
    let resolver = Resolver::new();
    let cwd = file.parent().unwrap().to_path_buf();

    let resolved = resolver.resolve(
      "./a.ts",
      cwd.clone(),
      &ResolveKind::Entry(String::new()),
      &Some("_".to_string()),
      &Arc::new(CompilationContext::default()),
    );
    assert!(resolved.is_some());
    let resolved = resolved.unwrap();

    assert_eq!(
      resolved.resolved_path,
      cwd.join("a.ts").to_string_lossy().to_string()
    );
  });
}
