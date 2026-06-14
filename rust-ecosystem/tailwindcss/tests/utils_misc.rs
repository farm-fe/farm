//! Smoke tests for `utils/{dimensions, is_color, is_valid_arbitrary,
//! math_operators}` — upstream has no dedicated `.test.ts` files for these
//! (math-operators is tested transitively through `decode-arbitrary-value`),
//! so tests are derived from upstream JSDoc + tsdoc comments and from
//! representative cases exercised by `decode-arbitrary-value.test.ts`.

use farmfe_ecosystem_tailwindcss::utils::dimensions::parse_dimension;
use farmfe_ecosystem_tailwindcss::utils::is_color::is_color;
use farmfe_ecosystem_tailwindcss::utils::is_valid_arbitrary::is_valid_arbitrary;
use farmfe_ecosystem_tailwindcss::utils::math_operators::{
  add_whitespace_around_math_operators, has_math_fn,
};

// ----- dimensions -------------------------------------------------------------

#[test]
fn dimensions_parses_length_with_unit() {
  let d = parse_dimension("64rem").unwrap();
  assert_eq!(d.value, 64.0);
  assert_eq!(d.unit.as_deref(), Some("rem"));
}

#[test]
fn dimensions_parses_percentage() {
  let d = parse_dimension("100%").unwrap();
  assert_eq!(d.value, 100.0);
  assert_eq!(d.unit.as_deref(), Some("%"));
}

#[test]
fn dimensions_parses_unitless() {
  let d = parse_dimension("0.5").unwrap();
  assert_eq!(d.value, 0.5);
  assert_eq!(d.unit, None);
}

#[test]
fn dimensions_rejects_nonnumeric() {
  assert!(parse_dimension("abc").is_none());
  assert!(parse_dimension("1px2").is_none());
}

// ----- is_color ---------------------------------------------------------------

#[test]
fn is_color_hex() {
  assert!(is_color("#fff"));
  assert!(is_color("#ABCDEF"));
}

#[test]
fn is_color_named() {
  assert!(is_color("red"));
  assert!(is_color("REBECCAPURPLE"));
  assert!(is_color("transparent"));
  assert!(is_color("currentcolor"));
}

#[test]
fn is_color_fn_call() {
  assert!(is_color("rgb(0, 0, 0)"));
  assert!(is_color("rgba(0,0,0,0)"));
  assert!(is_color("hsl(120deg, 50%, 50%)"));
  assert!(is_color("oklab(0.5 0 0)"));
  assert!(is_color("oklch(0.5 0.2 100)"));
  assert!(is_color("color(display-p3 1 0 0)"));
  assert!(is_color("light-dark(white, black)"));
  assert!(is_color("color-mix(in oklab, red, blue)"));
}

#[test]
fn is_color_rejects_non_colors() {
  assert!(!is_color("16px"));
  assert!(!is_color("notacolor"));
  assert!(!is_color("12em"));
}

// ----- is_valid_arbitrary -----------------------------------------------------

#[test]
fn iva_balanced_strings_are_valid() {
  assert!(is_valid_arbitrary("foo"));
  assert!(is_valid_arbitrary("(a + b)"));
  assert!(is_valid_arbitrary("[a, b]"));
  assert!(is_valid_arbitrary("foo(bar(baz))"));
}

#[test]
fn iva_unbalanced_close_brackets_rejected() {
  // Closing without matching opening returns false:
  assert!(!is_valid_arbitrary("foo)"));
  assert!(!is_valid_arbitrary("foo]"));
  // NOTE: an unclosed open paren at end is still considered "valid" by the
  // upstream JS implementation, which never checks residual stack depth.
  assert!(is_valid_arbitrary("foo("));
  assert!(is_valid_arbitrary("foo[bar"));
}

#[test]
fn iva_top_level_semicolon_rejected() {
  assert!(!is_valid_arbitrary("a;b"));
  assert!(is_valid_arbitrary("(a;b)"));
}

#[test]
fn iva_string_contents_dont_unbalance() {
  assert!(is_valid_arbitrary("\"(unmatched\""));
  assert!(is_valid_arbitrary("'](]['"));
}

#[test]
fn iva_curly_does_not_push_stack() {
  // `{` is intentionally not pushed to the closing-bracket stack. This
  // mirrors upstream behaviour and means a top-level `;` inside braces is
  // still detected as top-level. Wrapping in `[]` covers the `;`, so this
  // input is "valid".
  assert!(is_valid_arbitrary("[&{color:red;}]"));
  // But without surrounding brackets the `;` is top-level and is rejected:
  assert!(!is_valid_arbitrary("{color:red;}"));
}

#[test]
fn iva_escape_handled() {
  assert!(is_valid_arbitrary("foo\\(bar"));
}

// ----- math_operators (direct) ------------------------------------------------

#[test]
fn has_math_fn_detects_calc() {
  assert!(has_math_fn("calc(1 + 2)"));
  assert!(has_math_fn("foo calc(1+2)"));
  assert!(!has_math_fn("1 + 2"));
  assert!(!has_math_fn("var(--x)"));
}

#[test]
fn add_whitespace_noop_outside_math() {
  // No math fn -> input returned unchanged.
  assert_eq!(
    add_whitespace_around_math_operators("var(--foo)+1"),
    "var(--foo)+1"
  );
}

#[test]
fn add_whitespace_inside_calc() {
  assert_eq!(
    add_whitespace_around_math_operators("calc(1+2)"),
    "calc(1 + 2)"
  );
}
