use farmfe_core::config::{Config, OutputConfig, TargetEnv};
use farmfe_core::{context::CompilationContext, plugin::ResolveKind};
use farmfe_plugin_resolve::resolver::Resolver;
use farmfe_testing_helpers::fixture;
use std::sync::Arc;

#[test]
fn resolve_exports_basic() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();
      // Parsing packages in node_modules
      let resolved = resolver.resolve(
        "basic/base",
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
          .join("basic")
          .join("lib")
          .join("basic-base-exports.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_exports_deep_import() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();
      // Parsing packages in node_modules
      let resolved = resolver.resolve(
        "solid-js/runtime/helper/esm/transform",
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
          .join("solid-js")
          .join("runtime")
          .join("helper")
          .join("esm")
          .join("index.js")
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
        "replace/feature",
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
          .join("replace")
          .join("lib")
          .join("browser-feature.js")
          .to_string_lossy()
          .to_string()
      );

      let resolved = resolver.resolve(
        "replace/submodule.js",
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
          .join("replace")
          .join("lib")
          .join("submodule.js")
          .to_string_lossy()
          .to_string()
      );

      // let resolved = resolver.resolve(
      //   "replace/lib/basic-exports.js",
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
      //     .join("replace")
      //     .join("lib")
      //     .join("basic-exports.js")
      //     .to_string_lossy()
      //     .to_string()
      // );

      let resolved = resolver.resolve(
        "replace",
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
          .join("replace")
          .join("lib")
          .join("basic-exports-base-fields.js")
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
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("nesting")
          .join("server")
          .join("esm-bundler.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn export_nesting_node_env() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let context = CompilationContext::new(
        Config {
          output: OutputConfig {
            target_env: TargetEnv::Node,
            ..Default::default()
          },
          ..Default::default()
        },
        vec![],
      )
      .unwrap();
      let resolved_1 = resolver.resolve(
        "nesting/config",
        cwd.clone(),
        &ResolveKind::Import,
        &Arc::new(context),
      );
      assert!(resolved_1.is_some());
      let resolved_1 = resolved_1.unwrap();
      assert_eq!(
        resolved_1.resolved_path,
        cwd
          .join("node_modules")
          .join("nesting")
          .join("server")
          .join("index.mjs")
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
fn resolve_try_file() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "priority/es",
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
          .join("priority")
          .join("es")
          .join("ans")
          .join("index.js")
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
fn resolve_exports_nesting_dot_fields() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "solid-js",
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
          .join("solid-js")
          .join("dist")
          .join("dev.js")
          .to_string_lossy()
          .to_string()
      );

      let resolved = resolver.resolve(
        "solid-js",
        cwd.clone(),
        &ResolveKind::Require,
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("solid-js")
          .join("dist")
          .join("dev.cjs")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_exports_nesting_dot_value() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "solid-js/jsx-runtime",
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
          .join("solid-js")
          .join("dist")
          .join("test-exports.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_exports_nesting_base_path() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "solid-js/store",
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
          .join("solid-js")
          .join("store")
          .join("dist")
          .join("dev.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_exports_nesting_path() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "solid-js/store",
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
          .join("solid-js")
          .join("store")
          .join("dist")
          .join("dev.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_exports_nesting_base() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "solid-js/dist/dev.js",
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
          .join("solid-js")
          .join("dist")
          .join("dev.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]
fn resolve_exports_string_fields() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "exports",
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
          .join("exports")
          .join("exports.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]

fn resolve_exports_no_fields() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "@dev/util",
        cwd.clone(),
        &ResolveKind::Require,
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("@dev")
          .join("util")
          .join("esm")
          .join("index.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}

#[test]

fn resolve_exports_nesting_no_fields() {
  fixture!(
    "tests/fixtures/resolve-node-modules/exports/index.ts",
    |file, _| {
      let cwd = file.parent().unwrap().to_path_buf();
      let resolver = Resolver::new();

      let resolved = resolver.resolve(
        "@dev/util/lib/is-string",
        cwd.clone(),
        &ResolveKind::Require,
        &Arc::new(CompilationContext::default()),
      );
      assert!(resolved.is_some());
      let resolved = resolved.unwrap();
      assert_eq!(
        resolved.resolved_path,
        cwd
          .join("node_modules")
          .join("@dev")
          .join("util")
          .join("lib")
          .join("is-string.js")
          .to_string_lossy()
          .to_string()
      );
    }
  );
}
