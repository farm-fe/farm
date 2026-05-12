use farmfe_ecosystem_tailwindcss::ast::to_css;
use farmfe_ecosystem_tailwindcss::candidate::ParsedCandidate;
use farmfe_ecosystem_tailwindcss::theme::Theme;
use farmfe_ecosystem_tailwindcss::utilities::UtilityRegistry;

fn make_static(name: &str) -> ParsedCandidate {
  ParsedCandidate {
    utility_root: name.to_string(),
    utility_value: None,
    arbitrary_property: None,
    arbitrary_value: None,
    type_hint: None,
    variants: vec![],
    modifier: None,
    modifier_is_arbitrary: false,
    important: false,
    is_static: true,
    raw: name.to_string(),
  }
}

#[test]
fn test_flex_utility() {
  let registry = UtilityRegistry::builtin();
  let theme = Theme::default();
  let candidate = make_static("flex");
  let result = registry.generate(&candidate, &theme);
  let output = to_css(&result);
  assert!(!output.is_empty());
  assert!(output.contains("display: flex"));
}

#[test]
fn test_display_block_utility() {
  let registry = UtilityRegistry::builtin();
  let theme = Theme::default();
  let candidate = make_static("block");
  let result = registry.generate(&candidate, &theme);
  let output = to_css(&result);
  assert_eq!(output.trim(), ".block {\n  display: block;\n}");
}

#[test]
fn test_unknown_utility_returns_empty() {
  let registry = UtilityRegistry::builtin();
  let theme = Theme::default();
  let candidate = make_static("nonexistent-utility-xyz");
  let result = registry.generate(&candidate, &theme);
  assert!(result.is_empty());
}

#[test]
fn test_arbitrary_property() {
  let registry = UtilityRegistry::builtin();
  let theme = Theme::default();
  let candidate = ParsedCandidate {
    utility_root: String::new(),
    utility_value: None,
    arbitrary_property: Some(("color".to_string(), "red".to_string())),
    arbitrary_value: None,
    type_hint: None,
    variants: vec![],
    modifier: None,
    modifier_is_arbitrary: false,
    important: false,
    is_static: false,
    raw: "[color:red]".to_string(),
  };
  let result = registry.generate(&candidate, &theme);
  assert!(!result.is_empty());
  let output = to_css(&result);
  assert!(output.contains("color: red"));
}
