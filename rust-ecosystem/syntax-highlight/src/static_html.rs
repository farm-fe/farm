use crate::Highlighted;
use std::collections::BTreeSet;

fn hexadecimal_color(value: &str) -> bool {
  matches!(value.len(), 7 | 9)
    && value.starts_with('#')
    && value.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
}

fn escape_text(value: &str, output: &mut String) {
  for character in value.chars() {
    match character {
      '&' => output.push_str("&amp;"),
      '<' => output.push_str("&lt;"),
      '>' => output.push_str("&gt;"),
      '"' => output.push_str("&quot;"),
      '\'' => output.push_str("&#x27;"),
      '\r' => output.push_str("&#13;"),
      character => output.push(character),
    }
  }
}

/// Render the same intrinsic line/token spans as the JSX emitter, without
/// creating a JavaScript object and element expression for every token.
///
/// Only generated tokens, validated hex colors, integer line numbers and fixed
/// attribute names enter markup. Authored MDX/HTML is never accepted as markup.
/// None retains JSX when HTML parser normalization could change token text.
pub fn render_static_lines(
  highlighted: &Highlighted,
  selected_lines: &BTreeSet<usize>,
) -> Result<Option<String>, String> {
  if selected_lines
    .iter()
    .any(|line| *line == 0 || *line > highlighted.lines.len())
  {
    return Err("[DOCS_HIGHLIGHT_HTML] selected line is outside the block".into());
  }
  for tokens in &highlighted.lines {
    for token in tokens {
      if token.text.contains('\0') {
        return Ok(None);
      }
      if !hexadecimal_color(&token.light) || !hexadecimal_color(&token.dark) {
        return Err("[DOCS_HIGHLIGHT_HTML] invalid generated color".into());
      }
    }
  }
  let mut output = String::new();
  for (index, tokens) in highlighted.lines.iter().enumerate() {
    let line = index + 1;
    output.push_str("<span class=\"farm-code-line\" data-line=\"");
    output.push_str(&line.to_string());
    output.push('"');
    if selected_lines.contains(&line) {
      output.push_str(" data-highlighted=\"true\"");
    }
    output.push('>');
    for token in tokens {
      output.push_str("<span class=\"farm-code-token\" style=\"--code-dark:");
      output.push_str(&token.dark);
      output.push_str(";--code-light:");
      output.push_str(&token.light);
      output.push_str("\">");
      escape_text(&token.text, &mut output);
      output.push_str("</span>");
    }
    output.push_str("</span>");
  }
  Ok(Some(output))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{HighlightOptions, HighlightRepresentation, Token, highlight};

  #[test]
  fn default_options_keep_jsx_compatibility() {
    let options = HighlightOptions::default();
    assert_eq!(options.representation, HighlightRepresentation::Jsx);
  }

  #[test]
  fn generated_html_escapes_code_without_turning_it_into_markup() {
    let highlighted = Highlighted {
      lines: vec![vec![Token {
        text: "</code><script>alert(\"x\")</script>&amp;'😀\n".into(),
        light: "#Ab12CD".into(),
        dark: "#98765432".into(),
      }]],
      known_language: true,
    };
    let html = render_static_lines(&highlighted, &BTreeSet::from([1]))
      .unwrap()
      .unwrap();
    assert!(html.contains("data-line=\"1\" data-highlighted=\"true\""));
    assert!(html.contains("--code-dark:#98765432;--code-light:#Ab12CD"));
    assert!(html.contains(
      "&lt;/code&gt;&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;&amp;amp;&#x27;😀\n"
    ));
    assert!(!html.contains("<script>"));
    assert_eq!(html.matches("class=\"farm-code-token\"").count(), 1);
  }

  #[test]
  fn carriage_returns_are_preserved_as_numeric_character_references() {
    for text in ["a\r\nb", "a\rb", "\r<script>&\r"] {
      let highlighted = Highlighted {
        lines: vec![vec![Token {
          text: text.into(),
          light: "#123456".into(),
          dark: "#abcdef".into(),
        }]],
        known_language: true,
      };
      let html = render_static_lines(&highlighted, &BTreeSet::new())
        .unwrap()
        .unwrap();
      assert!(!html.contains('\r'));
      assert_eq!(html.matches("&#13;").count(), text.matches('\r').count());
      assert!(!html.contains("<script>"));
      assert_eq!(html.matches('\n').count(), text.matches('\n').count());
    }
  }

  #[test]
  fn nul_keeps_the_jsx_fallback() {
    for text in ["a\0b"] {
      let highlighted = Highlighted {
        lines: vec![vec![Token {
          text: text.into(),
          light: "#123456".into(),
          dark: "#abcdef".into(),
        }]],
        known_language: true,
      };
      assert!(
        render_static_lines(&highlighted, &BTreeSet::new())
          .unwrap()
          .is_none()
      );
    }
  }

  #[test]
  fn unexpected_color_cannot_inject_a_style_or_attribute() {
    for color in [
      "red",
      "#fff",
      "#123456;display:none",
      "#123456\" onclick=\"x",
    ] {
      let highlighted = Highlighted {
        lines: vec![vec![Token {
          text: "safe".into(),
          light: color.into(),
          dark: "#abcdef".into(),
        }]],
        known_language: true,
      };
      assert!(render_static_lines(&highlighted, &BTreeSet::new()).is_err());
    }
  }

  #[test]
  fn real_two_theme_stream_keeps_all_tokens_and_lines() {
    let highlighted = highlight(
      "const answer = 1;\n// 中文 😀\n",
      "ts",
      &HighlightOptions::default(),
    )
    .unwrap();
    let html = render_static_lines(&highlighted, &BTreeSet::from([2]))
      .unwrap()
      .unwrap();
    let tokens = highlighted.lines.iter().map(Vec::len).sum::<usize>();
    assert_eq!(html.matches("class=\"farm-code-token\"").count(), tokens);
    assert_eq!(html.matches("class=\"farm-code-line\"").count(), 2);
    for token in highlighted.lines.iter().flatten() {
      assert!(html.contains(&format!(
        "--code-dark:{};--code-light:{}",
        token.dark, token.light
      )));
    }
    assert!(html.contains("中文"));
    assert!(html.contains('😀'));
    assert_eq!(html.matches("data-highlighted=\"true\"").count(), 1);
  }
}
