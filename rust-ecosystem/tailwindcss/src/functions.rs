use crate::ast::AstNode;
use crate::theme::Theme;
use crate::walk::{walk, WalkAction};

/// Substitute `theme()` calls in CSS declaration values with resolved theme
/// values.
pub fn substitute_css_functions(nodes: Vec<AstNode>, theme: &Theme) -> Vec<AstNode> {
  walk(nodes, &mut |node: &AstNode, _path, _depth| {
    if let AstNode::Declaration(decl) = node {
      if let Some(ref value) = decl.value {
        let new_value = replace_theme_functions(value, theme);
        if new_value != *value {
          let mut new_decl = decl.clone();
          new_decl.value = Some(new_value);
          return WalkAction::Replace(vec![AstNode::Declaration(new_decl)]);
        }
      }
    }
    WalkAction::Continue
  })
}

/// Replace all `theme(...)` occurrences in a CSS value string with the
/// resolved theme values.
fn replace_theme_functions(value: &str, theme: &Theme) -> String {
  if !value.contains("theme(") {
    return value.to_string();
  }

  let mut result = value.to_string();

  // Iteratively replace the first theme() found
  while let Some(start) = result.find("theme(") {
    let after_paren = &result[start + 6..];
    let mut depth = 1usize;
    let mut end = None;

    for (i, ch) in after_paren.char_indices() {
      match ch {
        '(' => depth += 1,
        ')' => {
          depth -= 1;
          if depth == 0 {
            end = Some(i);
            break;
          }
        }
        _ => {}
      }
    }

    let Some(end_idx) = end else {
      break;
    };

    let key_path = after_paren[..end_idx].trim();
    let replacement = theme
      .resolve_by_key_path(key_path)
      .unwrap_or_else(|| format!("theme({})", key_path));

    let full_end = start + 6 + end_idx + 1;
    result.replace_range(start..full_end, &replacement);
  }

  result
}
