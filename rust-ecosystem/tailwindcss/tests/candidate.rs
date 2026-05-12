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
