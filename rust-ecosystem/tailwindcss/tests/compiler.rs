use farmfe_ecosystem_tailwindcss::{compile, CompilerOptions, Features, Polyfills, TailwindConfig};
use farmfe_testing_helpers::assert_snapshot;
use serde_json::json;

#[test]
fn core_compiler_accepts_external_config() {
  let mut compiler = compile(
    ".foo { color: red; }",
    CompilerOptions {
      features: Features::AT_APPLY | Features::THEME_FUNCTION,
      polyfills: Polyfills::AT_MEDIA_HOVER,
      dependencies: vec!["/tmp/input.css".to_string()],
      source_maps_enabled: true,
      config: Some(TailwindConfig::new(json!({
        "theme": {
          "extend": {
            "colors": {
              "brand": "#123456"
            }
          }
        },
        "content": ["src/**/*.tsx"]
      }))),
    },
  )
  .unwrap();

  let output = format!(
    "css: {}\nmap: {}\ndeps: {}\nhas_config: {}\nfeatures: {}\npolyfills: {}",
    compiler.build(&[]),
    compiler.build_source_map().unwrap_or_default(),
    compiler.dependencies().join(","),
    compiler.config().is_some(),
    compiler.features.contains(Features::AT_APPLY),
    compiler.polyfills.contains(Polyfills::AT_MEDIA_HOVER),
  );

  assert_snapshot!(output);
}

#[test]
fn build_inlines_tailwind_utilities_marker() {
  let mut compiler = compile(
    ".prose { font-family: serif; }\n@tailwind utilities;\n",
    CompilerOptions::default(),
  )
  .unwrap();

  let css = compiler.build(&["flex".to_string(), "block".to_string()]);
  // The user rule is preserved, and the marker is replaced with generated
  // utility rules from the supplied candidates.
  assert!(
    css.contains(".prose"),
    "expected user rule to be preserved: {css}"
  );
  assert!(css.contains(".flex"), "expected .flex utility: {css}");
  assert!(css.contains(".block"), "expected .block utility: {css}");
  assert!(
    !css.contains("@tailwind"),
    "expected @tailwind marker to be removed: {css}"
  );
}

#[test]
fn build_inlines_import_tailwindcss_marker() {
  let mut compiler = compile(
    "@import \"tailwindcss\";\n.foo { color: red; }\n",
    CompilerOptions::default(),
  )
  .unwrap();

  let css = compiler.build(&["flex".to_string()]);
  assert!(css.contains(".foo"), "expected user rule: {css}");
  assert!(css.contains(".flex"), "expected .flex utility: {css}");
  assert!(
    !css.contains("@import") || !css.contains("tailwindcss"),
    "expected @import \"tailwindcss\" marker to be removed: {css}"
  );
}

#[test]
fn build_substitutes_at_apply_in_user_css() {
  let mut compiler = compile(
    ".btn {\n  @apply flex;\n}\n",
    CompilerOptions::default(),
  )
  .unwrap();

  let css = compiler.build(&[]);
  assert!(css.contains(".btn"), "expected .btn rule: {css}");
  assert!(
    css.contains("display: flex"),
    "expected @apply flex to be substituted with display: flex: {css}"
  );
  assert!(
    !css.contains("@apply"),
    "expected @apply directive to be removed: {css}"
  );
}

#[test]
fn build_drops_marker_when_no_candidates() {
  let mut compiler = compile(
    ".foo { color: red; }\n@tailwind utilities;\n",
    CompilerOptions::default(),
  )
  .unwrap();

  let css = compiler.build(&[]);
  assert!(css.contains(".foo"));
  assert!(!css.contains("@tailwind"));
}
