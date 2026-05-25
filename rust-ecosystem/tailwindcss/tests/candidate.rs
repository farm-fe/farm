use farmfe_ecosystem_tailwindcss::candidate::parse_candidate;

#[test]
fn test_parse_static_utility() {
  let result = parse_candidate("flex");
  assert!(result.is_some());
  let c = result.unwrap();
  assert_eq!(c.utility_root, "flex");
  assert!(c.variants.is_empty());
  assert!(!c.important);
}

#[test]
fn test_parse_functional_utility() {
  let result = parse_candidate("bg-red-500");
  assert!(result.is_some());
}

#[test]
fn test_parse_utility_with_variant() {
  let result = parse_candidate("hover:bg-red-500");
  assert!(result.is_some());
  let c = result.unwrap();
  assert!(!c.variants.is_empty());
}

#[test]
fn test_parse_utility_with_important_prefix() {
  let result = parse_candidate("!flex");
  assert!(result.is_some());
  let c = result.unwrap();
  assert!(c.important);
}

#[test]
fn test_parse_utility_with_important_suffix() {
  let result = parse_candidate("flex!");
  assert!(result.is_some());
  let c = result.unwrap();
  assert!(c.important);
}

#[test]
fn test_parse_stacked_variants() {
  let result = parse_candidate("focus:hover:flex");
  assert!(result.is_some());
  let c = result.unwrap();
  assert_eq!(c.variants.len(), 2);
}

#[test]
fn test_parse_arbitrary_value() {
  let result = parse_candidate("bg-[#0088cc]");
  assert!(result.is_some());
}

#[test]
fn test_parse_with_modifier() {
  let result = parse_candidate("bg-red-500/50");
  assert!(result.is_some());
}

#[test]
fn test_parse_arbitrary_property() {
  let result = parse_candidate("[color:red]");
  assert!(result.is_some());
}

#[test]
fn test_parse_invalid_candidate_returns_none() {
  assert!(parse_candidate("").is_none());
}

#[test]
fn test_parse_arbitrary_variant() {
  let result = parse_candidate("[&_p]:flex");
  assert!(result.is_some());
}

// ── Phase 13: negative utilities ──────────────────────────────────────────────

#[test]
fn test_parse_negative_utility() {
  let c = parse_candidate("-mt-4").expect("should parse");
  assert!(c.negative);
  assert_eq!(c.utility_root, "mt");
  assert_eq!(c.utility_value.as_deref(), Some("4"));
}

#[test]
fn test_parse_negative_utility_with_variant() {
  let c = parse_candidate("hover:-mt-4").expect("should parse");
  assert!(c.negative);
  assert_eq!(c.variants, vec!["hover".to_string()]);
  assert_eq!(c.utility_root, "mt");
}

#[test]
fn test_parse_non_negative_when_no_dash_prefix() {
  let c = parse_candidate("mt-4").expect("should parse");
  assert!(!c.negative);
}

#[test]
fn test_parse_negative_with_important_prefix() {
  // `!-mt-4`: important strip first, then negative
  let c = parse_candidate("!-mt-4").expect("should parse");
  assert!(c.important);
  assert!(c.negative);
  assert_eq!(c.utility_root, "mt");
}

#[test]
fn test_parse_negative_does_not_apply_to_arbitrary_property() {
  // `-[color:red]` is not a valid negative utility — treat as non-negative.
  let c = parse_candidate("-[color:red]");
  assert!(c.is_none() || !c.unwrap().negative);
}

// ── Phase 13: underscore normalisation in arbitrary values ────────────────────

#[test]
fn test_arbitrary_value_underscore_becomes_space() {
  let c = parse_candidate("font-[Helvetica_Neue]").expect("should parse");
  assert_eq!(c.arbitrary_value.as_deref(), Some("Helvetica Neue"));
}

#[test]
fn test_arbitrary_value_escaped_underscore_preserved() {
  let c = parse_candidate(r"font-[snake\_case]").expect("should parse");
  assert_eq!(c.arbitrary_value.as_deref(), Some("snake_case"));
}

#[test]
fn test_arbitrary_value_underscore_inside_url_preserved() {
  let c = parse_candidate("bg-[url(./a_b.png)]").expect("should parse");
  assert_eq!(c.arbitrary_value.as_deref(), Some("url(./a_b.png)"));
}

#[test]
fn test_arbitrary_property_value_underscore_becomes_space() {
  let c = parse_candidate("[font-family:Helvetica_Neue]").expect("should parse");
  assert_eq!(
    c.arbitrary_property.as_ref().map(|(_, v)| v.as_str()),
    Some("Helvetica Neue")
  );
}

// --- Phase 13 closure: paren-arbitrary CSS variable shorthand ---
// Ported from upstream tailwindlabs/tailwindcss `candidate.test.ts`.

#[test]
fn test_paren_shorthand_var_value() {
  let c = parse_candidate("bg-(--my-color)").expect("should parse");
  assert_eq!(c.utility_root, "bg");
  assert_eq!(c.arbitrary_value.as_deref(), Some("var(--my-color)"));
  assert_eq!(c.type_hint, None);
}

#[test]
fn test_paren_shorthand_rejects_non_var_value() {
  // Upstream: `bg-(my-color)` is invalid because the inner value does not
  // start with `--`.
  assert!(parse_candidate("bg-(my-color)").is_none());
}

#[test]
fn test_paren_shorthand_with_type_hint() {
  let c = parse_candidate("bg-(color:--my-color)").expect("should parse");
  assert_eq!(c.utility_root, "bg");
  assert_eq!(c.arbitrary_value.as_deref(), Some("var(--my-color)"));
  assert_eq!(c.type_hint.as_deref(), Some("color"));
}

#[test]
fn test_paren_shorthand_escaped_underscore_decoded() {
  // `\_` inside the paren-shorthand decodes to a literal `_`. Plain `_` is
  // also preserved as `_` (unlike bracket-arbitrary values which convert
  // underscores to spaces).
  let c = parse_candidate(r"flex-(--\_foo)").expect("should parse");
  assert_eq!(c.arbitrary_value.as_deref(), Some("var(--_foo)"));

  let c2 = parse_candidate("flex-(--_foo)").expect("should parse");
  assert_eq!(c2.arbitrary_value.as_deref(), Some("var(--_foo)"));
}

#[test]
fn test_paren_shorthand_empty_is_rejected() {
  assert!(parse_candidate("bg-()").is_none());
}
