use farmfe_core::{config::ResolveConfig, plugin::ResolveKind};
use farmfe_plugin_resolve::resolver::Resolver;
use farmfe_testing_helpers::fixture;

#[test]
fn resolve_imports_basic() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new(ResolveConfig::default());
      // Parsing packages in node_modules
      let resolved = resolver.resolve("chalk", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      println!("resolved resolved_path: {:?}", resolved.resolved_path);
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("chalk")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}