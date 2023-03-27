use farmfe_core::{config::ResolveConfig, plugin::ResolveKind};
use farmfe_plugin_resolve::resolver::Resolver;
use farmfe_testing_helpers::fixture;

/// See browser field spec (https://github.com/defunctzombie/package-browser-field-spec)

#[test]
fn resolve_browser_basic() {
  fixture!(
    "tests/fixtures/resolve-node-modules/browser/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new(ResolveConfig::default());

      let resolved = resolver.resolve("basic", cwd.clone(), &ResolveKind::Import);
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
    }
  );
}

#[test]
fn resolve_browser_replace() {
  fixture!(
    "tests/fixtures/resolve-node-modules/browser/node_modules/replace/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new(ResolveConfig::default());

      let resolved = resolver.resolve("./only.js", cwd.join("server"), &ResolveKind::Import);
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

      // let resolved = resolver.resolve("module-a", cwd.clone(), &ResolveKind::Import);
      // assert!(resolved.is_some());
      // let resolved = resolved.unwrap();

      // assert_eq!(
      //   resolved.resolved_path,
      //   cwd
      //     .join("shims")
      //     .join("module-a.js")
      //     .to_string_lossy()
      //     .to_string()
      // );

      // normal resolve
      // let resolved = resolver.resolve("./module-a.js", cwd.join("shims"), &ResolveKind::Import);
      // assert!(resolved.is_some());
      // let resolved = resolved.unwrap();

      // assert_eq!(
      //   resolved.resolved_path,
      //   cwd
      //     .join("shims")
      //     .join("module-a.js")
      //     .to_string_lossy()
      //     .to_string()
      // );
    }
  );
}

#[test]
fn resolve_browser_ignore() {
  fixture!(
    "tests/fixtures/resolve-node-modules/browser/node_modules/ignore/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new(ResolveConfig::default());

      let resolved = resolver.resolve("module-a", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();

      assert_eq!(resolved.resolved_path, "module-a".to_string());
      assert!(resolved.external);

      let resolved = resolver.resolve("./only.js", cwd.join("server"), &ResolveKind::Import);
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
      assert!(!resolved.side_effects);
    }
  );
}
