use farmfe_core::{
  config::{OutputConfig, ResolveConfig, TargetEnv},
  plugin::ResolveKind,
};
use farmfe_plugin_resolve::resolver::Resolver;
use farmfe_testing_helpers::fixture;

// imports node fields can be import node_modules if use node need replace package name

// package.json
// {
//   "imports": {
//     "#dep": {
//       "node": "dep-node-native",
//       "default": "./dep-polyfill.js"
//     }
//   },
//   "dependencies": {
//     "dep-node-native": "^1.0.0"
//   }
// }

#[test]
fn resolve_imports_basic() {
  fixture!(
    // TODO node environment
    "tests/fixtures/resolve-node-modules/imports/node_modules/chalk/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new(ResolveConfig::default(), OutputConfig::default());

      // Parsing packages in node_modules
      let resolved = resolver.resolve("#ansi-styles", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("source")
          .join("vendor")
          .join("ansi-styles")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_imports_replace_object() {
  fixture!(
    // TODO node environment
    "tests/fixtures/resolve-node-modules/imports/node_modules/chalk/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new(ResolveConfig::default(), OutputConfig::default());

      // Parsing packages in node_modules
      let resolved = resolver.resolve("#supports-color", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("source")
          .join("vendor")
          .join("supports-color")
          .join("browser.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_imports_replace_deps() {
  fixture!(
    // TODO node environment
    "tests/fixtures/resolve-node-modules/imports/node_modules/chalk/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new(ResolveConfig::default(), OutputConfig::default());

      // import resolve other deps like `"#ansi-styles-execa": "execa"`
      let resolved = resolver.resolve("#ansi-styles-execa", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .parent()
          .unwrap()
          .to_path_buf()
          .join("execa")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_imports_target_browser() {
  fixture!(
    // TODO node environment
    "tests/fixtures/resolve-node-modules/imports/node_modules/chalk/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new(
        ResolveConfig::default(),
        OutputConfig {
          target_env: TargetEnv::Browser,
          ..Default::default()
        },
      );

      let resolved = resolver.resolve("#supports-color", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("source")
          .join("vendor")
          .join("supports-color")
          .join("browser.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_imports_target_node() {
  fixture!(
    // TODO node environment
    "tests/fixtures/resolve-node-modules/imports/node_modules/chalk/package.json",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new(
        ResolveConfig::default(),
        OutputConfig {
          target_env: TargetEnv::Node,
          ..Default::default()
        },
      );

      let resolved = resolver.resolve("#supports-color", cwd.clone(), &ResolveKind::Import);
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("source")
          .join("vendor")
          .join("supports-color")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}
