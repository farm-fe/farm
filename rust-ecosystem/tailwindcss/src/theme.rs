use std::collections::HashMap;

/// Resolved Tailwind theme with CSS variables.
#[derive(Debug, Clone, Default)]
pub struct Theme {
  /// CSS custom properties defined in `@theme { ... }`
  pub variables: HashMap<String, String>,
  /// Keyframe definitions (name → list of AST nodes)
  pub keyframes: HashMap<String, Vec<crate::ast::AstNode>>,
}

impl Theme {
  /// Resolve a CSS variable name to its value.
  pub fn resolve(&self, name: &str) -> Option<String> {
    self.variables.get(name).cloned()
  }

  /// Resolve a dot-path key like "colors.red.500" to a CSS variable value.
  ///
  /// The dot-path is converted to `--<segments-joined-by-dashes>`. The
  /// leading namespace segment is singularised by convention (e.g.
  /// "colors" → "color") to match Tailwind v4's variable naming scheme.
  pub fn resolve_by_key_path(&self, key_path: &str) -> Option<String> {
    let var_name = key_path_to_var(key_path);
    self.variables.get(&var_name).cloned()
  }
}

/// Convert a dot-path theme key to a CSS variable name.
///
/// Rules:
/// - "colors.red.500" → "--color-red-500"  (trailing 's' on first segment stripped)
/// - "spacing.4"      → "--spacing-4"      (no trailing 's' to strip)
fn key_path_to_var(key_path: &str) -> String {
  let parts: Vec<&str> = key_path.split('.').collect();
  let mut result = String::from("--");

  for (i, part) in parts.iter().enumerate() {
    if i > 0 {
      result.push('-');
    }
    if i == 0 {
      // Singularise the first segment if it ends with 's'
      // e.g. "colors" → "color"
      if part.ends_with('s') && part.len() > 1 {
        result.push_str(&part[..part.len() - 1]);
      } else {
        result.push_str(part);
      }
    } else {
      result.push_str(part);
    }
  }

  result
}
