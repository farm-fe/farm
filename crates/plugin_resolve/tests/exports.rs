use farmfe_core::{config::ResolveConfig, plugin::ResolveKind};
use farmfe_plugin_resolve::resolver::Resolver;
use farmfe_testing_helpers::fixture;

#[test]
fn resolve_exports_basic() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      println!("解析的config {:?}", ResolveConfig::default());
      let resolver = Resolver::new(ResolveConfig::default());
      println!("cwd: {:?}", cwd.clone());
      // Parsing packages in node_modules
      let resolved = resolver.resolve("basic", cwd.clone(), &ResolveKind::Import);
      // println!("resolve path, {:?}", resolved);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      // println!("resolve path, {}", resolved.resolved_path);
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("basic")
          .join("lib")
          .join("basic-exports.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_exports_replace() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      println!("解析的config {:?}", ResolveConfig::default());
      let resolver = Resolver::new(ResolveConfig::default());
      println!("cwd: {:?}", cwd.clone());
      // Parsing packages in node_modules
      let resolved = resolver.resolve("basic", cwd.clone(), &ResolveKind::Import);
    }
  );
}

#[test]
fn it_works() {
  println!("Hello, world!, {}", 2 + 2);
  assert_eq!(2 + 2, 4);
}
