use std::sync::Arc;

use farmfe_core::{
  config::{Config, OutputConfig},
  context::CompilationContext,
  plugin::ResolveKind,
};
use farmfe_plugin_resolve::resolver::{ResolveOptions, Resolver};
use farmfe_testing_helpers::fixture;

#[test]
fn resolve_exports_does_not_partially_match_json_in_extensionless_names() {
  let cwd = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests/fixtures/resolve-node-modules/exports");
  let resolver = Resolver::new();
  for file in ["a.plain.b", "a.json.b", "a.json.b.js"] {
    let resolved = resolver
      .resolve(
        &format!("wildcard-suffix/modules/{file}"),
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      )
      .unwrap();
    let file = if file.ends_with(".js") {
      file.to_string()
    } else {
      format!("{file}.js")
    };
    assert_eq!(
      std::path::Path::new(&resolved.resolved_path),
      cwd.join("node_modules/wildcard-suffix/modules").join(file)
    );
  }
}

#[test]
fn resolve_exports_basic() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();
      // Parsing packages in node_modules
      let resolved = resolver.resolve(
        "basic",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
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
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "replace/submodule.js",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
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
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
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

      let resolved = resolver.resolve(
        "replace",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
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

      let resolved = resolver.resolve(
        "replace/feature",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
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
fn resolve_exports_nesting() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "nesting/config",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
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
fn resolve_exports_nesting_default() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "nest-resolve",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("nest-resolve")
          .join("index.js")
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
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "degrade",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("degrade")
          .join("index.umd.js")
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
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "direct-analysis/module",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
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
        "direct-analysis",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
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

#[test]
fn resolve_no_fields() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "no-fields",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("no-fields")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

// test priority
#[test]
fn resolve_priority() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "priority",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("priority")
          .join("index.mjs")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_exports_require() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();
      // Parsing packages in node_modules
      let resolved = resolver.resolve(
        "nesting-require",
        cwd.clone(),
        &ResolveKind::Require,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("nesting-require")
          .join("dist")
          .join("cjs")
          .join("src")
          .join("index-cjs.js")
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
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "require-import/config",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
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

      let resolved = resolver.resolve(
        "require-import/config",
        cwd.clone(),
        &ResolveKind::Require,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
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
fn resolve_exports_jsnext() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();
      // Parsing packages in node_modules
      let resolved = resolver.resolve(
        "resolve-jsnext",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("resolve-jsnext")
          .join("tslib.es6.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_exports_issue_997() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();
      // Parsing packages in node_modules
      let resolved = resolver.resolve(
        "@issues/997/query/react",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("@issues")
          .join("997")
          .join("dist")
          .join("query")
          .join("react")
          .join("rtk-query-react.modern.mjs")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_exports_browser() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "export-browser",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        // &Arc::new(CompilationContext::default()),
        &Arc::new(
          CompilationContext::new(
            Config {
              output: Box::new(OutputConfig {
                target_env: farmfe_core::config::TargetEnv::Node,
                ..Default::default()
              }),
              ..Default::default()
            },
            vec![],
          )
          .unwrap(),
        ),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("export-browser")
          // .join("dev-browser.js")
          .join("dev-ssr.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
// resolve exports value is string
fn resolve_exports_issue_1433() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();
      // Parsing packages in node_modules
      let resolved = resolver.resolve(
        "@issues/1433",
        cwd.clone(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("@issues")
          .join("1433")
          .join("lib")
          .join("default.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}
