use crate::ast::AstNode;
use crate::theme::Theme;
use crate::walk::{WalkAction, walk};

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
  // Scan once, preserving quoted strings and unknown functions. In particular,
  // an unresolved theme() must not restart the scan at the same position.
  let mut result = String::new();
  let bytes = value.as_bytes();
  let mut cursor = 0;
  let mut literal_start = 0;
  while cursor < bytes.len() {
    if bytes[cursor] == b'\\' {
      cursor = (cursor + 2).min(bytes.len());
      continue;
    }
    if bytes[cursor] == b'\'' || bytes[cursor] == b'"' {
      let quote = bytes[cursor];
      cursor += 1;
      while cursor < bytes.len() {
        if bytes[cursor] == b'\\' {
          cursor = (cursor + 2).min(bytes.len());
        } else if bytes[cursor] == quote {
          cursor += 1;
          break;
        } else {
          cursor += 1;
        }
      }
      continue;
    }
    let boundary = cursor == 0
      || (bytes[cursor - 1].is_ascii()
        && !matches!(bytes[cursor - 1], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-'));
    let internal = boundary && bytes[cursor..].starts_with(b"--theme(");
    let legacy = boundary && bytes[cursor..].starts_with(b"theme(");
    if !internal && !legacy {
      cursor += 1;
      continue;
    }
    let open = cursor + if internal { 7 } else { 5 };
    let Some(close) = closing_parenthesis(bytes, open) else {
      break;
    };
    let arguments = &value[open + 1..close];
    let replacement = if internal {
      // Tailwind's internal --theme() becomes a native CSS variable lookup;
      // its complete comma-separated fallback is preserved.
      format!("var({})", replace_theme_functions(arguments, theme))
    } else {
      theme
        .resolve_by_key_path(arguments.trim())
        .unwrap_or_else(|| value[cursor..=close].to_string())
    };
    result.push_str(&value[literal_start..cursor]);
    result.push_str(&replacement);
    cursor = close + 1;
    literal_start = cursor;
  }
  result.push_str(&value[literal_start..]);
  result
}

fn closing_parenthesis(bytes: &[u8], open: usize) -> Option<usize> {
  let mut depth = 1;
  let mut quote = None;
  let mut cursor = open + 1;
  while cursor < bytes.len() {
    let ch = bytes[cursor];
    if ch == b'\\' {
      cursor += 2;
      continue;
    }
    if let Some(current) = quote {
      if ch == current {
        quote = None;
      }
    } else {
      match ch {
        b'\'' | b'"' => quote = Some(ch),
        b'(' => depth += 1,
        b')' => {
          depth -= 1;
          if depth == 0 {
            return Some(cursor);
          }
        }
        _ => {}
      }
    }
    cursor += 1;
  }
  None
}
