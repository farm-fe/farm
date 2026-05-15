//! Ported from upstream `packages/tailwindcss/src/attribute-selector-parser.test.ts`.

use farmfe_ecosystem_tailwindcss::attribute_selector_parser::{
  parse, AttributeOperator, AttributeQuote, AttributeSelector, AttributeSensitivity,
};

#[test]
fn invalid_inputs_return_none() {
  let cases = [
    "",
    "]",
    "[]",
    "[",
    "=\"value\"",
    "data-foo]",
    "[data-foo",
    "[data-foo=\"foo]",
    "[data-foo * = foo]",
    "[data-foo*=]",
    "[data-foo=value x]",
    "[data-foo=value ix]",
  ];
  for c in cases {
    assert!(parse(c).is_none(), "expected None for {c:?}");
  }
}

fn s(v: &str) -> Option<String> {
  Some(v.to_string())
}

#[test]
fn parses_bare_attribute() {
  assert_eq!(
    parse("[data-foo]"),
    Some(AttributeSelector {
      attribute: "data-foo".into(),
      operator: None,
      quote: None,
      value: None,
      sensitivity: None,
    })
  );
}

#[test]
fn parses_padded_attribute() {
  assert_eq!(
    parse("[ data-foo ]"),
    Some(AttributeSelector {
      attribute: "data-foo".into(),
      operator: None,
      quote: None,
      value: None,
      sensitivity: None,
    })
  );
}

#[test]
fn parses_eq_unquoted() {
  assert_eq!(
    parse("[data-state=expanded]"),
    Some(AttributeSelector {
      attribute: "data-state".into(),
      operator: Some(AttributeOperator::Equals),
      quote: None,
      value: s("expanded"),
      sensitivity: None,
    })
  );
}

#[test]
fn parses_eq_unquoted_padded() {
  assert_eq!(
    parse("[data-state = expanded ]"),
    Some(AttributeSelector {
      attribute: "data-state".into(),
      operator: Some(AttributeOperator::Equals),
      quote: None,
      value: s("expanded"),
      sensitivity: None,
    })
  );
}

#[test]
fn parses_substring_quoted() {
  assert_eq!(
    parse("[data-state*=\"expanded\"]"),
    Some(AttributeSelector {
      attribute: "data-state".into(),
      operator: Some(AttributeOperator::Contains),
      quote: Some(AttributeQuote::Double),
      value: s("expanded"),
      sensitivity: None,
    })
  );
}

#[test]
fn parses_substring_quoted_with_sensitivity() {
  assert_eq!(
    parse("[data-state*=\"expanded\"i]"),
    Some(AttributeSelector {
      attribute: "data-state".into(),
      operator: Some(AttributeOperator::Contains),
      quote: Some(AttributeQuote::Double),
      value: s("expanded"),
      sensitivity: Some(AttributeSensitivity::Insensitive),
    })
  );
}

#[test]
fn parses_substring_unquoted_with_sensitivity() {
  assert_eq!(
    parse("[data-state*=expanded i]"),
    Some(AttributeSelector {
      attribute: "data-state".into(),
      operator: Some(AttributeOperator::Contains),
      quote: None,
      value: s("expanded"),
      sensitivity: Some(AttributeSensitivity::Insensitive),
    })
  );
}

#[test]
fn real_world_example() {
  assert_eq!(
    parse("[data-url$=\".com\"i]"),
    Some(AttributeSelector {
      attribute: "data-url".into(),
      operator: Some(AttributeOperator::EndsWith),
      quote: Some(AttributeQuote::Double),
      value: s(".com"),
      sensitivity: Some(AttributeSensitivity::Insensitive),
    })
  );
}

#[test]
fn operator_str_round_trip() {
  for (op, s) in [
    (AttributeOperator::Equals, "="),
    (AttributeOperator::Includes, "~="),
    (AttributeOperator::DashMatch, "|="),
    (AttributeOperator::StartsWith, "^="),
    (AttributeOperator::EndsWith, "$="),
    (AttributeOperator::Contains, "*="),
  ] {
    assert_eq!(op.as_str(), s);
  }
}
