//! Verbatim port of the "without replacer" describe block from
//! `packages/tailwindcss/src/utils/replace-shadow-colors.test.ts`. The "with
//! replacer" block depends on `replaceAlpha` from upstream `utilities.ts`,
//! which is out of scope for this phase.

use farmfe_ecosystem_tailwindcss::utils::replace_shadow_colors::replace_shadow_colors;

fn replacer(color: &str) -> String {
  format!("var(--tw-shadow-color, {color})")
}

#[test]
fn handles_var_shadow() {
  let parsed = replace_shadow_colors("var(--my-shadow)", replacer);
  assert_eq!(parsed, "var(--my-shadow)");
}

#[test]
fn handles_var_shadow_with_offset() {
  let parsed = replace_shadow_colors("1px var(--my-shadow)", replacer);
  assert_eq!(parsed, "1px var(--my-shadow)");
}

#[test]
fn handles_var_color_with_offsets() {
  let parsed = replace_shadow_colors("1px 1px var(--my-color)", replacer);
  assert_eq!(parsed, "1px 1px var(--tw-shadow-color, var(--my-color))");
}

#[test]
fn handles_var_color_with_zero_offsets() {
  let parsed = replace_shadow_colors("0 0 0 var(--my-color)", replacer);
  assert_eq!(parsed, "0 0 0 var(--tw-shadow-color, var(--my-color))");
}

#[test]
fn handles_two_values_with_currentcolor() {
  let parsed = replace_shadow_colors("1px 2px", replacer);
  assert_eq!(parsed, "1px 2px var(--tw-shadow-color, currentcolor)");
}

#[test]
fn handles_three_values_with_currentcolor() {
  let parsed = replace_shadow_colors("1px 2px 3px", replacer);
  assert_eq!(parsed, "1px 2px 3px var(--tw-shadow-color, currentcolor)");
}

#[test]
fn handles_four_values_with_currentcolor() {
  let parsed = replace_shadow_colors("1px 2px 3px 4px", replacer);
  assert_eq!(parsed, "1px 2px 3px 4px var(--tw-shadow-color, currentcolor)");
}

#[test]
fn handles_multiple_shadows() {
  let input =
    "var(--my-shadow), 1px 1px var(--my-color), 0 0 1px var(--my-color)";
  let parsed = replace_shadow_colors(input, replacer);
  assert_eq!(
    parsed,
    "var(--my-shadow), 1px 1px var(--tw-shadow-color, var(--my-color)), 0 0 1px var(--tw-shadow-color, var(--my-color))"
  );
}
