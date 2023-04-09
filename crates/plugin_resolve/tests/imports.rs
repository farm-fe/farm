use farmfe_core::{config::ResolveConfig, plugin::ResolveKind};
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
      let resolver = Resolver::new(ResolveConfig::default());
      // Parsing packages in node_modules
      // let resolved = resolver.resolve("#ansi-styles", cwd.clone(), &ResolveKind::Import);
      // assert!(resolved.is_some());
      // let resolved = resolved.unwrap();
      // assert_eq!(
      //   resolved.resolved_path,
      //   cwd
      //     .join("source")
      //     .join("vendor")
      //     .join("ansi-styles")
      //     .join("index.js")
      //     .to_string_lossy()
      //     .to_string()
      // );

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
