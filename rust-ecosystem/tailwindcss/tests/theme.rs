use std::collections::HashMap;
use farmfe_ecosystem_tailwindcss::theme::Theme;

#[test]
fn test_theme_parse_simple_variables() {
  let mut variables = HashMap::new();
  variables.insert("--color-red-500".to_string(), "#ef4444".to_string());
  variables.insert("--color-blue-500".to_string(), "#3b82f6".to_string());
  let theme = Theme {
    variables,
    keyframes: HashMap::new(),
  };

  assert_eq!(theme.resolve("--color-red-500"), Some("#ef4444".to_string()));
  assert_eq!(theme.resolve("--color-blue-500"), Some("#3b82f6".to_string()));
  assert_eq!(theme.resolve("--color-green-500"), None);
}

#[test]
fn test_theme_resolve_by_key_path() {
  let mut variables = HashMap::new();
  variables.insert("--color-red-500".to_string(), "#ef4444".to_string());
  let theme = Theme {
    variables,
    keyframes: HashMap::new(),
  };

  // Dot-path resolution: colors.red.500 -> --color-red-500
  assert_eq!(
    theme.resolve_by_key_path("colors.red.500"),
    Some("#ef4444".to_string())
  );
}

#[test]
fn test_theme_empty() {
  let theme = Theme {
    variables: HashMap::new(),
    keyframes: HashMap::new(),
  };
  assert_eq!(theme.resolve("--anything"), None);
}
