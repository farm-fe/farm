use std::collections::HashMap;

use farmfe_core::{config::ResolveConfig, plugin::ResolveKind};
use farmfe_plugin_resolve::resolver::Resolver;
use farmfe_testing_helpers::fixture;

#[test]
fn resolve_relative_specifier_without_extension() {
  fixture(
    "tests/fixtures/resolve-relative-specifier/**/index.*",
    |file, _| {
      let resolver = Resolver::new(ResolveConfig::default());
      let cwd = file.parent().unwrap().to_path_buf();

      let resolved = resolver.resolve("./index", cwd.clone(), &ResolveKind::Entry);
      assert!(resolved.is_ok());
      let resolved = resolved.unwrap().unwrap();
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
      let resolver = Resolver::new(ResolveConfig::default());
      let cwd = file.parent().unwrap().to_path_buf();

      let resolved = resolver.resolve("./index.html", cwd.clone(), &ResolveKind::Entry);
      assert!(resolved.is_err());

      let resolved = resolver.resolve("./index.ts", cwd.clone(), &ResolveKind::Entry);
      let resolved = resolved.unwrap().unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd.join("index.ts").to_string_lossy().to_string()
      );
    },
  );
}

#[test]
fn resolve_node_modules_normal() {
  fixture(
    "tests/fixtures/resolve-node-modules/normal/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new(ResolveConfig::default());

      let resolved = resolver.resolve("pkg-a", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_ok());
      let resolved = resolved.unwrap().unwrap();

      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("pkg-a")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );

      let resolved = resolver.resolve("pkg-b", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_ok());
      let resolved = resolved.unwrap().unwrap();

      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("pkg-b")
          .join("es")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      )
    },
  );
}

#[test]
fn resolve_alias() {
  fixture("tests/fixtures/resolve-alias/index.ts", |file, _| {
    let cwd = file.parent().unwrap().to_path_buf();
    let resolver = Resolver::new(ResolveConfig {
      alias: HashMap::from([("@".to_string(), cwd.to_string_lossy().to_string())]),
      ..Default::default()
    });

    let resolved = resolver.resolve("@/pages/a", cwd.clone(), &ResolveKind::Import);
    assert!(resolved.is_ok());
    let resolved = resolved.unwrap().unwrap();

    assert_eq!(
      resolved.resolved_path,
      cwd
        .join("pages")
        .join("a.tsx")
        .to_string_lossy()
        .to_string()
    );
  });
}
