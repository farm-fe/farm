//! Ported from upstream `packages/tailwindcss/src/selector-parser.test.ts`.

use farmfe_ecosystem_tailwindcss::selector_parser::{parse, to_css, SelectorAstNode};

fn sel(v: &str) -> SelectorAstNode {
  SelectorAstNode::selector(v)
}
fn sep(v: &str) -> SelectorAstNode {
  SelectorAstNode::separator(v)
}
fn comb(v: &str) -> SelectorAstNode {
  SelectorAstNode::combinator(v)
}
fn func(v: &str, nodes: Vec<SelectorAstNode>) -> SelectorAstNode {
  SelectorAstNode::function(v, nodes)
}
fn val(v: &str) -> SelectorAstNode {
  SelectorAstNode::value(v)
}

// ── parse ────────────────────────────────────────────────────────────────────

#[test]
fn parse_simple() {
  assert_eq!(parse(".foo"), vec![sel(".foo")]);
}

#[test]
fn parse_compound() {
  assert_eq!(
    parse(".foo.bar:hover#id"),
    vec![sel(".foo"), sel(".bar"), sel(":hover"), sel("#id")]
  );
}

#[test]
fn parse_selector_list() {
  assert_eq!(parse(".foo,.bar"), vec![sel(".foo"), sep(","), sel(".bar")]);
}

#[test]
fn parse_attribute_combined_into_selector() {
  assert_eq!(
    parse(".foo[bar=\"baz\"]"),
    vec![sel(".foo"), sel("[bar=\"baz\"]")]
  );
}

#[test]
fn parse_function_recursive() {
  assert_eq!(
    parse(".foo:hover:not(.bar:focus)"),
    vec![
      sel(".foo"),
      sel(":hover"),
      func(":not", vec![sel(".bar"), sel(":focus")]),
    ]
  );
}

#[test]
fn parse_next_children_combinator() {
  assert_eq!(parse(".foo + p"), vec![sel(".foo"), comb(" + "), sel("p")]);
}

#[test]
fn parse_escaped_characters() {
  assert_eq!(parse("foo\\.bar"), vec![sel("foo\\.bar")]);
}

#[test]
fn parse_nth_child() {
  assert_eq!(
    parse(":nth-child(n+1)"),
    vec![func(":nth-child", vec![val("n+1")])]
  );
}

#[test]
fn parse_nested_has_nth_child() {
  assert_eq!(
    parse("&:has(.child:nth-child(2))"),
    vec![
      sel("&"),
      func(
        ":has",
        vec![sel(".child"), func(":nth-child", vec![val("2")])],
      ),
    ]
  );
}

#[test]
fn parse_has_with_lone_nth_child() {
  assert_eq!(
    parse("&:has(:nth-child(2))"),
    vec![
      sel("&"),
      func(":has", vec![func(":nth-child", vec![val("2")])]),
    ]
  );
}

#[test]
fn parse_amp_before_attribute() {
  assert_eq!(parse("&[data-foo]"), vec![sel("&"), sel("[data-foo]")]);
}

#[test]
fn parse_amp_after_attribute() {
  assert_eq!(parse("[data-foo]&"), vec![sel("[data-foo]"), sel("&")]);
}

#[test]
fn parse_star_before_attribute() {
  assert_eq!(parse("*[data-foo]"), vec![sel("*"), sel("[data-foo]")]);
}

#[test]
fn parse_star_after_attribute() {
  assert_eq!(parse("[data-foo]*"), vec![sel("[data-foo]"), sel("*")]);
}

// ── toCss ────────────────────────────────────────────────────────────────────

#[test]
fn to_css_simple() {
  assert_eq!(to_css(&parse(".foo")), ".foo");
}

#[test]
fn to_css_compound() {
  assert_eq!(to_css(&parse(".foo.bar:hover#id")), ".foo.bar:hover#id");
}

#[test]
fn to_css_selector_list() {
  assert_eq!(to_css(&parse(".foo,.bar")), ".foo,.bar");
}

#[test]
fn to_css_attribute_selector() {
  assert_eq!(to_css(&parse(".foo[bar=\"baz\"]")), ".foo[bar=\"baz\"]");
}

#[test]
fn to_css_function() {
  assert_eq!(
    to_css(&parse(".foo:hover:not(.bar:focus)")),
    ".foo:hover:not(.bar:focus)"
  );
}

#[test]
fn to_css_escaped_characters() {
  assert_eq!(to_css(&parse("foo\\.bar")), "foo\\.bar");
}

#[test]
fn to_css_nth_child() {
  assert_eq!(to_css(&parse(":nth-child(n+1)")), ":nth-child(n+1)");
}
