//! Theme tests — Phase 14 parity port.
//!
//! Cases mirror behaviours of the upstream `Theme` class in
//! `tailwindlabs/tailwindcss/packages/tailwindcss/src/theme.ts`.

use farmfe_ecosystem_tailwindcss::theme::{Theme, ThemeOptions};

// ----- Original Phase 3 smoke tests (preserved, rewritten for new API) -----

#[test]
fn test_theme_parse_simple_variables() {
  let mut theme = Theme::new();
  theme.insert("--color-red-500", "#ef4444");
  theme.insert("--color-blue-500", "#3b82f6");

  assert_eq!(
    theme.lookup_var("--color-red-500"),
    Some("#ef4444".to_string())
  );
  assert_eq!(
    theme.lookup_var("--color-blue-500"),
    Some("#3b82f6".to_string())
  );
  assert_eq!(theme.lookup_var("--color-green-500"), None);
}

#[test]
fn test_theme_resolve_by_key_path() {
  let mut theme = Theme::new();
  theme.insert("--color-red-500", "#ef4444");

  assert_eq!(
    theme.resolve_by_key_path("colors.red.500"),
    Some("#ef4444".to_string())
  );
}

#[test]
fn test_theme_empty() {
  let theme = Theme::new();
  assert_eq!(theme.lookup_var("--anything"), None);
  assert_eq!(theme.size(), 0);
}

// ----- Phase 14: upstream-parity behaviour --------------------------------

#[test]
fn test_theme_add_and_get() {
  let mut theme = Theme::new();
  theme.add("--color-red", "#f00", ThemeOptions::NONE);
  theme.add("--color-blue", "#00f", ThemeOptions::NONE);

  assert_eq!(theme.size(), 2);
  assert_eq!(theme.get(&["--color-red"]), Some("#f00"));
  assert_eq!(theme.get(&["--missing", "--color-blue"]), Some("#00f"));
  assert_eq!(theme.get(&["--missing"]), None);
}

#[test]
fn test_theme_initial_deletes_entry() {
  let mut theme = Theme::new();
  theme.add("--color-red", "#f00", ThemeOptions::NONE);
  theme.add("--color-red", "initial", ThemeOptions::NONE);
  assert_eq!(theme.size(), 0);
}

#[test]
fn test_theme_default_does_not_override_user_value() {
  let mut theme = Theme::new();
  theme.add("--color-red", "user-value", ThemeOptions::NONE);
  theme.add("--color-red", "default-value", ThemeOptions::DEFAULT);

  assert_eq!(theme.get(&["--color-red"]), Some("user-value"));
  assert!(!theme.has_default("--color-red"));
}

#[test]
fn test_theme_default_set_when_no_value() {
  let mut theme = Theme::new();
  theme.add("--color-red", "default-value", ThemeOptions::DEFAULT);
  assert_eq!(theme.get(&["--color-red"]), Some("default-value"));
  assert!(theme.has_default("--color-red"));
}

#[test]
fn test_theme_namespace_wildcard_clears() {
  let mut theme = Theme::new();
  theme.add("--color-red", "#f00", ThemeOptions::NONE);
  theme.add("--color-blue", "#00f", ThemeOptions::NONE);
  theme.add("--spacing-1", "0.25rem", ThemeOptions::NONE);

  theme.add("--color-*", "initial", ThemeOptions::NONE);

  assert_eq!(theme.get(&["--color-red"]), None);
  assert_eq!(theme.get(&["--color-blue"]), None);
  assert_eq!(theme.get(&["--spacing-1"]), Some("0.25rem"));
}

#[test]
fn test_theme_universal_wildcard_clears_all() {
  let mut theme = Theme::new();
  theme.add("--color-red", "#f00", ThemeOptions::NONE);
  theme.add("--spacing-1", "0.25rem", ThemeOptions::NONE);

  theme.add("--*", "initial", ThemeOptions::NONE);
  assert_eq!(theme.size(), 0);
}

#[test]
fn test_theme_ignored_key_for_font_namespace() {
  // The `--font` namespace excludes `--font-weight` and `--font-size`.
  let mut theme = Theme::new();
  theme.add("--font-sans", "system-ui", ThemeOptions::NONE);
  theme.add("--font-weight-bold", "700", ThemeOptions::NONE);
  theme.add("--font-size-sm", "0.875rem", ThemeOptions::NONE);

  let keys = theme.keys_in_namespaces(&["--font"]);
  assert!(keys.contains(&"sans".to_string()));
  assert!(!keys.iter().any(|k| k.starts_with("weight")));
  assert!(!keys.iter().any(|k| k.starts_with("size")));
}

#[test]
fn test_theme_keys_in_namespaces_skips_nested_subvariables() {
  let mut theme = Theme::new();
  theme.add("--text-sm", "0.875rem", ThemeOptions::NONE);
  theme.add("--text-sm--line-height", "1.25rem", ThemeOptions::NONE);

  let keys = theme.keys_in_namespaces(&["--text"]);
  assert_eq!(keys, vec!["sm".to_string()]);
}

#[test]
fn test_theme_resolve_returns_var_reference() {
  let mut theme = Theme::new();
  theme.add("--color-red-500", "#ef4444", ThemeOptions::NONE);

  let result = theme.resolve(Some("red-500"), &["--color"], ThemeOptions::NONE);
  assert_eq!(result, Some("var(--color-red-500)".to_string()));
}

#[test]
fn test_theme_resolve_inline_returns_value() {
  let mut theme = Theme::new();
  theme.add("--color-red-500", "#ef4444", ThemeOptions::INLINE);

  let result = theme.resolve(Some("red-500"), &["--color"], ThemeOptions::NONE);
  assert_eq!(result, Some("#ef4444".to_string()));
}

#[test]
fn test_theme_resolve_reference_includes_fallback() {
  let mut theme = Theme::new();
  theme.add("--color-red-500", "#ef4444", ThemeOptions::REFERENCE);

  let result = theme.resolve(Some("red-500"), &["--color"], ThemeOptions::NONE);
  assert_eq!(result, Some("var(--color-red-500, #ef4444)".to_string()));
}

#[test]
fn test_theme_resolve_with_dot_in_candidate_value() {
  // Candidate values containing `.` should also be tried with `_`.
  let mut theme = Theme::new();
  theme.add("--spacing-1_5", "0.375rem", ThemeOptions::NONE);

  let result = theme.resolve_value(Some("1.5"), &["--spacing"]);
  assert_eq!(result, Some("0.375rem".to_string()));
}

#[test]
fn test_theme_resolve_value_returns_raw() {
  let mut theme = Theme::new();
  theme.add("--color-red", "#f00", ThemeOptions::NONE);

  assert_eq!(
    theme.resolve_value(Some("red"), &["--color"]),
    Some("#f00".to_string())
  );
}

#[test]
fn test_theme_resolve_with_nested_keys() {
  let mut theme = Theme::new();
  theme.add("--text-sm", "0.875rem", ThemeOptions::INLINE);
  theme.add("--text-sm--line-height", "1.25rem", ThemeOptions::INLINE);

  let (value, extra) = theme
    .resolve_with("sm", &["--text"], &["--line-height"])
    .expect("should resolve");
  assert_eq!(value, "0.875rem");
  assert_eq!(extra.get("--line-height"), Some(&"1.25rem".to_string()));
}

#[test]
fn test_theme_namespace_returns_suffix_map() {
  let mut theme = Theme::new();
  theme.add("--color-red", "#f00", ThemeOptions::NONE);
  theme.add("--color-blue", "#00f", ThemeOptions::NONE);
  theme.add("--color", "default", ThemeOptions::NONE);

  let ns = theme.namespace("--color");
  assert_eq!(ns.get(&None), Some(&"default".to_string()));
  assert_eq!(ns.get(&Some("red".to_string())), Some(&"#f00".to_string()));
  assert_eq!(ns.get(&Some("blue".to_string())), Some(&"#00f".to_string()));
}

#[test]
fn test_theme_prefix_applied() {
  let mut theme = Theme::new();
  theme.prefix = Some("tw".to_string());
  theme.add("--color-red-500", "#ef4444", ThemeOptions::NONE);

  let result = theme.resolve(Some("red-500"), &["--color"], ThemeOptions::NONE);
  assert_eq!(result, Some("var(--tw-color-red-500)".to_string()));
}

#[test]
fn test_theme_clear_namespace_respects_clear_options() {
  let mut theme = Theme::new();
  theme.add("--color-red", "#f00", ThemeOptions::DEFAULT);
  theme.add("--color-blue", "#00f", ThemeOptions::NONE);

  theme.clear_namespace("--color", ThemeOptions::DEFAULT);
  assert!(theme.lookup_var("--color-red").is_none());
  assert_eq!(theme.lookup_var("--color-blue"), Some("#00f".to_string()));
}

#[test]
fn test_theme_mark_used_variable_idempotent() {
  let mut theme = Theme::new();
  theme.add("--color-red", "#f00", ThemeOptions::NONE);

  assert!(theme.mark_used_variable("--color-red"));
  assert!(!theme.mark_used_variable("--color-red"));
  assert!(theme.get_options("--color-red").contains(ThemeOptions::USED));
}

#[test]
fn test_theme_mark_used_variable_missing_returns_false() {
  let mut theme = Theme::new();
  assert!(!theme.mark_used_variable("--missing"));
}

#[test]
fn test_theme_entries_sorted_is_deterministic() {
  let mut theme = Theme::new();
  theme.add("--b", "2", ThemeOptions::NONE);
  theme.add("--a", "1", ThemeOptions::NONE);
  theme.add("--c", "3", ThemeOptions::NONE);

  let entries = theme.entries_sorted();
  assert_eq!(
    entries,
    vec![
      ("--a".to_string(), "1".to_string()),
      ("--b".to_string(), "2".to_string()),
      ("--c".to_string(), "3".to_string()),
    ]
  );
}

#[test]
fn test_theme_resolve_ignores_keys_under_ignored_namespace() {
  // Querying `--font` for value `weight-bold` would naively match
  // `--font-weight-bold`, but the ignored-key map should suppress it.
  let mut theme = Theme::new();
  theme.add("--font-weight-bold", "700", ThemeOptions::NONE);

  let result = theme.resolve(Some("weight-bold"), &["--font"], ThemeOptions::NONE);
  assert_eq!(result, None);
}
