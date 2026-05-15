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

#[test]
fn build_registers_user_at_utility_block() {
  let mut compiler = compile(
    "@utility tab-4 {\n  tab-size: 4;\n}\n@tailwind utilities;\n",
    CompilerOptions::default(),
  )
  .unwrap();

  let css = compiler.build(&["tab-4".to_string()]);
  assert!(
    css.contains("tab-size: 4") || css.contains("tab-size:4"),
    "expected user @utility tab-4 to emit tab-size: 4, got: {css}"
  );
  assert!(
    css.contains(".tab-4"),
    "expected .tab-4 class selector to be emitted: {css}"
  );
  assert!(
    !css.contains("@utility"),
    "expected @utility rule to be stripped from output: {css}"
  );
}

#[test]
fn build_user_utility_can_override_builtin() {
  let mut compiler = compile(
    "@utility block {\n  display: grid;\n}\n@tailwind utilities;\n",
    CompilerOptions::default(),
  )
  .unwrap();

  let css = compiler.build(&["block".to_string()]);
  assert!(
    css.contains("display: grid") || css.contains("display:grid"),
    "user @utility block should override built-in to emit display: grid: {css}"
  );
  assert!(
    !css.contains("display: block") && !css.contains("display:block"),
    "built-in display: block should not be emitted when overridden: {css}"
  );
}

#[test]
fn build_registers_custom_variant_selector_form() {
  let mut compiler = compile(
    "@custom-variant pointer-coarse (&:hover);\n@tailwind utilities;\n",
    CompilerOptions::default(),
  )
  .unwrap();

  let css = compiler.build(&["pointer-coarse:flex".to_string()]);
  assert!(
    css.contains(":hover"),
    "expected custom variant body `&:hover` to produce `:hover`, got: {css}"
  );
  assert!(
    css.contains("display: flex") || css.contains("display:flex"),
    "expected display: flex from utility: {css}"
  );
  assert!(
    !css.contains("@custom-variant"),
    "expected @custom-variant rule to be stripped from output: {css}"
  );
}

#[test]
fn build_registers_custom_variant_at_rule_form() {
  let mut compiler = compile(
    "@custom-variant pointer-coarse (@media (pointer: coarse));\n@tailwind utilities;\n",
    CompilerOptions::default(),
  )
  .unwrap();

  let css = compiler.build(&["pointer-coarse:flex".to_string()]);
  assert!(
    css.contains("@media") && css.contains("pointer: coarse"),
    "expected custom variant @media wrapper, got: {css}"
  );
}

#[test]
fn build_registers_user_at_theme_color_for_bg_utility() {
  let mut compiler = compile(
    "@theme {\n  --color-brand: #123456;\n}\n@tailwind utilities;\n",
    CompilerOptions::default(),
  )
  .unwrap();

  let css = compiler.build(&["bg-brand".to_string()]);
  assert!(
    css.contains("#123456"),
    "expected user --color-brand to flow into bg-brand: {css}"
  );
  assert!(
    css.contains(".bg-brand"),
    "expected .bg-brand class selector: {css}"
  );
  assert!(
    !css.contains("@theme"),
    "expected @theme rule to be stripped from output: {css}"
  );
}

#[test]
fn build_at_theme_supports_namespace_reset() {
  // The user resets every `--color-*` token via the `-*: initial` form,
  // then re-defines a single namespace entry. After the reset, the built-in
  // `--color-red-500` should no longer resolve, so `bg-red-500` produces
  // nothing while `bg-brand` still works.
  let mut compiler = compile(
    "@theme {\n  --color-*: initial;\n  --color-brand: #abcdef;\n}\n@tailwind utilities;\n",
    CompilerOptions::default(),
  )
  .unwrap();

  let css = compiler.build(&["bg-red-500".to_string(), "bg-brand".to_string()]);
  assert!(
    css.contains("#abcdef"),
    "expected --color-brand to resolve after reset: {css}"
  );
  assert!(
    !css.contains(".bg-red-500"),
    "expected bg-red-500 to be dropped after `--color-*: initial`: {css}"
  );
}

#[test]
fn build_at_theme_default_modifier_does_not_override_existing() {
  // First block sets the brand colour without any modifier.
  // Second block redefines it as `default`, which must be ignored because the
  // existing non-default entry takes precedence (mirrors upstream
  // `ThemeOptions::DEFAULT` semantics).
  let mut compiler = compile(
    concat!(
      "@theme {\n  --color-brand: #111111;\n}\n",
      "@theme default {\n  --color-brand: #999999;\n}\n",
      "@tailwind utilities;\n",
    ),
    CompilerOptions::default(),
  )
  .unwrap();

  let css = compiler.build(&["bg-brand".to_string()]);
  assert!(
    css.contains("#111111"),
    "expected user value to win over `default` redefinition: {css}"
  );
  assert!(
    !css.contains("#999999"),
    "default-modifier value should not override prior user value: {css}"
  );
}

#[test]
fn build_strips_at_theme_when_empty() {
  let mut compiler = compile(
    "@theme {\n  --color-brand: #321;\n}\n.foo { color: red; }\n",
    CompilerOptions::default(),
  )
  .unwrap();

  let css = compiler.build(&[]);
  assert!(css.contains(".foo"));
  assert!(
    !css.contains("@theme"),
    "expected @theme to be stripped even with no candidates: {css}"
  );
}
