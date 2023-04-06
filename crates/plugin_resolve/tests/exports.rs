use farmfe_core::{config::ResolveConfig, plugin::ResolveKind};
use farmfe_plugin_resolve::resolver::Resolver;
use farmfe_testing_helpers::fixture;

#[test]
fn resolve_exports_basic() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new(ResolveConfig::default());
      // Parsing packages in node_modules
      let resolved = resolver.resolve("basic", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      println!("resolved resolved_path: {:?}", resolved.resolved_path);
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
      let resolver = Resolver::new(ResolveConfig::default());

      let resolved = resolver.resolve("replace/submodule.js", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("replace")
          .join("lib")
          .join("submodule.js")
          .to_string_lossy()
          .to_string()
      );

      let resolved = resolver.resolve(
        "replace/lib/basic-exports.js",
        cwd.clone(),
        &ResolveKind::Import,
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("replace")
          .join("lib")
          .join("basic-exports.js")
          .to_string_lossy()
          .to_string()
      );

      let resolved = resolver.resolve("replace", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("replace")
          .join("lib")
          .join("basic-exports.js")
          .to_string_lossy()
          .to_string()
      );

      let resolved = resolver.resolve("replace/feature", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("replace")
          .join("lib")
          .join("browser-feature.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_exports_import_require() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new(ResolveConfig::default());

      let resolved = resolver.resolve("require-import/config", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("require-import")
          .join("lib")
          .join("base-import.js")
          .to_string_lossy()
          .to_string()
      );

      let resolved = resolver.resolve("require-import/config", cwd.clone(), &ResolveKind::Require);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("require-import")
          .join("lib")
          .join("base-require.cjs")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_exports_nesting() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new(ResolveConfig::default());

      let resolved = resolver.resolve("nesting/config", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("nesting")
          .join("dist")
          .join("esm-bundler.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_exports_degrade() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new(ResolveConfig::default());

      let resolved = resolver.resolve("degrade", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("degrade")
          .join("index.mjs")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_exports_direct_analysis() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new(ResolveConfig::default());

      let resolved = resolver.resolve("direct-analysis/module", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("direct-analysis")
          .join("direct-analysis-module.mjs")
          .to_string_lossy()
          .to_string()
      );

      let resolved = resolver.resolve(
        "direct-analysis/direct-analysis.js",
        cwd.clone(),
        &ResolveKind::Import,
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("direct-analysis")
          .join("direct-analysis.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}
