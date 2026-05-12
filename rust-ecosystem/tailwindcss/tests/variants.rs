use farmfe_ecosystem_tailwindcss::ast::to_css;
use farmfe_ecosystem_tailwindcss::candidate::ParsedCandidate;
use farmfe_ecosystem_tailwindcss::theme::Theme;
use farmfe_ecosystem_tailwindcss::utilities::UtilityRegistry;
use farmfe_ecosystem_tailwindcss::variants::VariantRegistry;

#[test]
fn test_hover_variant_generates_hover_selector() {
  let registry = UtilityRegistry::builtin();
  let theme = Theme::default();
  let candidate = ParsedCandidate {
    utility_root: "flex".to_string(),
    utility_value: None,
    arbitrary_property: None,
    arbitrary_value: None,
    type_hint: None,
    variants: vec!["hover".to_string()],
    modifier: None,
    modifier_is_arbitrary: false,
    important: false,
    is_static: true,
    raw: "hover:flex".to_string(),
  };

  let result = registry.generate(&candidate, &theme);
  let output = to_css(&result);
  assert!(output.contains(":hover"));
}

#[test]
fn test_focus_variant() {
  let registry = UtilityRegistry::builtin();
  let theme = Theme::default();
  let candidate = ParsedCandidate {
    utility_root: "block".to_string(),
    utility_value: None,
    arbitrary_property: None,
    arbitrary_value: None,
    type_hint: None,
    variants: vec!["focus".to_string()],
    modifier: None,
    modifier_is_arbitrary: false,
    important: false,
    is_static: true,
    raw: "focus:block".to_string(),
  };

  let result = registry.generate(&candidate, &theme);
  let output = to_css(&result);
  assert!(output.contains(":focus"));
}

#[test]
fn test_stacked_variants() {
  let registry = UtilityRegistry::builtin();
  let theme = Theme::default();
  let candidate = ParsedCandidate {
    utility_root: "flex".to_string(),
    utility_value: None,
    arbitrary_property: None,
    arbitrary_value: None,
    type_hint: None,
    variants: vec!["hover".to_string(), "focus".to_string()],
    modifier: None,
    modifier_is_arbitrary: false,
    important: false,
    is_static: true,
    raw: "focus:hover:flex".to_string(),
  };

  let result = registry.generate(&candidate, &theme);
  let output = to_css(&result);
  assert!(output.contains(":focus"));
  assert!(output.contains(":hover"));
}

#[test]
fn test_no_variant_returns_plain_selector() {
  let registry = UtilityRegistry::builtin();
  let theme = Theme::default();
  let candidate = ParsedCandidate {
    utility_root: "flex".to_string(),
    utility_value: None,
    arbitrary_property: None,
    arbitrary_value: None,
    type_hint: None,
    variants: vec![],
    modifier: None,
    modifier_is_arbitrary: false,
    important: false,
    is_static: true,
    raw: "flex".to_string(),
  };

  let result = registry.generate(&candidate, &theme);
  let output = to_css(&result);
  assert!(output.starts_with(".flex"));
}

#[test]
fn test_variant_registry_has_builtins() {
  let registry = VariantRegistry::builtin();
  assert!(registry.has("hover"));
  assert!(registry.has("focus"));
  assert!(registry.has("active"));
  assert!(registry.has("disabled"));
  assert!(!registry.has("nonexistent-variant"));
}
