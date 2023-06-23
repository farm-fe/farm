use std::sync::Arc;

use farmfe_core::{context::CompilationContext, plugin::ResolveKind};
use farmfe_plugin_resolve::resolver::Resolver;
use farmfe_testing_helpers::fixture;

#[test]
fn resolve_exports_basic() {
  fixture!(
    "tests/fixtures/resolve-node-modules/normal/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "test-priority",
        cwd.clone(),
        &ResolveKind::Import,
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();

      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("test-priority")
          .join("lib")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
      assert!(!resolved.external);
      assert!(!resolved.side_effects);

      // let resolved = resolver.resolve(
      //   "test-priority/lib/index.js",
      //   cwd.clone(),
      //   &ResolveKind::Import,
      //   &Arc::new(CompilationContext::default()),
      // );
      // assert!(resolved.is_some());
      // let resolved = resolved.unwrap();

      // assert_eq!(
      //   resolved.resolved_path,
      //   cwd
      //     .join("node_modules")
      //     .join("test-priority")
      //     .join("lib")
      //     .join("index.js")
      //     .to_string_lossy()
      //     .to_string()
      // );
      // assert!(!resolved.external);
      // assert!(!resolved.side_effects);
    }
  );
}
