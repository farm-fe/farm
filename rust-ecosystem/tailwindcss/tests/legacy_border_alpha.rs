use farmfe_ecosystem_tailwindcss::{compile, CompilerOptions, TailwindConfig};
fn build(config: Option<serde_json::Value>, source: &str, classes: &[&str]) -> String {
  let mut compiler = compile(
    source,
    CompilerOptions {
      config: config.map(TailwindConfig::new),
      ..Default::default()
    },
  )
  .unwrap();
  compiler.build(&classes.iter().map(|s| s.to_string()).collect::<Vec<_>>())
}
fn legacy() -> Option<serde_json::Value> {
  Some(serde_json::json!({"corePlugins":{"preflight":false}}))
}
#[test]
fn arbitrary_alpha_numbers_become_percentages_for_all_color_utilities() {
  for (modifier, expected) in [
    ("20", "20%"),
    ("[0.2]", "20%"),
    ("[.5]", "50%"),
    ("[20%]", "20%"),
    ("[0]", "0%"),
    ("[1]", "100%"),
  ] {
    for root in ["border", "bg", "text", "from", "via", "to"] {
      let class = format!("{root}-white/{modifier}");
      let css = build(None, "@tailwind utilities;", &[&class]);
      assert!(
        css.contains(&format!(
          "color-mix(in oklab, var(--color-white) {expected}, transparent)"
        )),
        "{class}: {css}"
      );
    }
  }
}
#[test]
fn arbitrary_alpha_decodes_percentage_expressions_without_corrupting_variables() {
  for (modifier, expected) in [
    ("[var(--my_alpha)]", "calc(var(--my_alpha) * 100%)"),
    ("[calc(20%_+_10%)]", "calc(20% + 10%)"),
    (
      "[calc(var(--percentage)_+_0%)]",
      "calc(var(--percentage) + 0%)",
    ),
    (
      "[calc(var(--ratio)_*_.5)]",
      "calc(calc(var(--ratio) * .5) * 100%)",
    ),
    (
      "[calc(var(--opacity)_*_100%)]",
      "calc(var(--opacity) * 100%)",
    ),
  ] {
    let class = format!("hover:border-white/{modifier}!");
    let css = build(legacy(), "@tailwind utilities;", &[&class]);
    assert!(
      css.contains(&format!(
        "var(--color-white) {expected}, transparent) !important"
      )),
      "{class}: {css}"
    );
    assert!(css.contains(":hover"), "{css}");
  }
}
#[test]
fn explicit_preflight_false_preserves_authored_style_for_border_widths() {
  let css = build(
    legacy(),
    "@tailwind utilities;",
    &[
      "border",
      "border-2",
      "border-t",
      "border-r-2",
      "border-b-4",
      "border-l",
      "border-x-2",
      "border-y-4",
      "hover:border-2!",
    ],
  );
  for property in [
    "border-width",
    "border-top-width",
    "border-right-width",
    "border-bottom-width",
    "border-left-width",
  ] {
    assert!(css.contains(property), "{css}");
  }
  assert!(
    !css.contains("-style:"),
    "legacy width utilities must not force visible borders: {css}"
  );
  assert!(css.contains("border-width: 2px !important"), "{css}");
}
#[test]
fn legacy_border_policy_does_not_change_default_or_other_configurations() {
  for config in [
    None,
    Some(serde_json::json!({})),
    Some(serde_json::json!({"corePlugins":{"preflight":true}})),
    Some(serde_json::json!({"corePlugins":{"preflight":"false"}})),
  ] {
    let css = build(config, "@tailwind utilities;", &["border", "border-t-2"]);
    assert!(
      css.contains("border-style: solid") && css.contains("border-top-style: solid"),
      "{css}"
    );
  }
}
#[test]
fn legacy_border_apply_preserves_explicit_authored_style() {
  let css = build(
    legacy(),
    ".card { border-style: dashed; @apply border-2 hover:border-t-4; } @tailwind utilities;",
    &[],
  );
  assert!(!css.contains("@apply"), "{css}");
  assert!(css.contains("border-style: dashed"), "{css}");
  assert!(!css.contains("style: solid"), "{css}");
  assert!(css.contains("border-top-width: 4px"), "{css}");
}
#[test]
fn legacy_policy_leaves_custom_utilities_untouched() {
  let css = build(
    legacy(),
    "@utility border { border-width: 3px; border-style: double; } @tailwind utilities;",
    &["border"],
  );
  assert!(
    css.contains("border-width: 3px") && css.contains("border-style: double"),
    "{css}"
  );
}
#[test]
fn legacy_width_policy_keeps_explicit_border_style_utilities() {
  let css = build(
    legacy(),
    "@tailwind utilities;",
    &["border", "border-solid", "hover:border-dashed!"],
  );
  assert!(
    css.contains(".border-solid") && css.contains("border-style: solid"),
    "{css}"
  );
  assert!(css.contains("border-style: dashed !important"), "{css}");
}

#[test]
fn invalid_alpha_values_do_not_generate_invalid_color_mix_weights() {
  for modifier in [
    "[NaN]",
    "[inf]",
    "[Infinity]",
    "[-0.2]",
    "[1.2]",
    "[120%]",
    "[banana]",
    "[20px]",
  ] {
    let class = format!("border-white/{modifier}");
    let css = build(None, "@tailwind utilities;", &[&class]);
    assert!(
      !css.contains("color-mix("),
      "unsupported alpha must be rejected, not relabeled: {class}: {css}"
    );
  }
}
