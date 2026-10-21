//! Native highlighting shared with content transforms; no JavaScript execution.
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use two_face::{
  re_exports::syntect::{
    easy::HighlightLines, highlighting::Style, parsing::SyntaxSet, util::LinesWithEndings,
  },
  theme::EmbeddedThemeName,
};

pub const CONFIG_KEY: &str = "farm:syntax-highlight";

/// Parse one-based, inclusive line ranges shared by fences and JSX markers.
/// Validate the bound before expanding a range to avoid unbounded allocation.
pub fn parse_line_ranges(
  input: &str,
  line_count: usize,
) -> Result<std::collections::BTreeSet<usize>, String> {
  let mut lines = std::collections::BTreeSet::new();
  for range in input.split(',') {
    let (start, end) = range
      .trim()
      .split_once('-')
      .map_or((range.trim(), range.trim()), |(a, b)| (a.trim(), b.trim()));
    let start = start
      .parse::<usize>()
      .map_err(|_| "invalid code line number")?;
    let end = end
      .parse::<usize>()
      .map_err(|_| "invalid code line number")?;
    if start == 0 || end < start || end > line_count {
      return Err("code line range is outside the block".into());
    }
    lines.extend(start..=end);
  }
  Ok(lines)
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeName {
  InspiredGithub,
  Base16OceanDark,
}

impl ThemeName {
  fn embedded(self) -> EmbeddedThemeName {
    match self {
      Self::InspiredGithub => EmbeddedThemeName::InspiredGithub,
      Self::Base16OceanDark => EmbeddedThemeName::Base16OceanDark,
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct HighlightOptions {
  pub light: ThemeName,
  pub dark: ThemeName,
}

impl Default for HighlightOptions {
  fn default() -> Self {
    Self {
      light: ThemeName::InspiredGithub,
      dark: ThemeName::Base16OceanDark,
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct Token {
  pub text: String,
  pub light: String,
  pub dark: String,
}
pub struct Highlighted {
  pub lines: Vec<Vec<Token>>,
  pub known_language: bool,
}
static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(two_face::syntax::extra_newlines);

fn color(style: Style) -> String {
  format!(
    "#{:02x}{:02x}{:02x}",
    style.foreground.r, style.foreground.g, style.foreground.b
  )
}

pub fn highlight(
  code: &str,
  language: &str,
  options: &HighlightOptions,
) -> Result<Highlighted, String> {
  let language = match language {
    "shell" | "bash" => "sh",
    "typescript" => "ts",
    "javascript" => "js",
    "jsx" => "tsx",
    "text" | "plaintext" | "txt" | "" => "txt",
    other => other,
  };
  let syntax = SYNTAXES.find_syntax_by_token(language);
  let known_language = syntax.is_some() || language == "txt";
  let syntax = syntax.unwrap_or_else(|| SYNTAXES.find_syntax_plain_text());
  let themes = two_face::theme::extra();
  let mut light = HighlightLines::new(syntax, &themes[options.light.embedded()]);
  let mut dark = HighlightLines::new(syntax, &themes[options.dark.embedded()]);
  let mut lines = vec![];
  for line in LinesWithEndings::from(code) {
    let light = light
      .highlight_line(line, &SYNTAXES)
      .map_err(|e| format!("[DOCS_HIGHLIGHT] {e}"))?;
    let dark = dark
      .highlight_line(line, &SYNTAXES)
      .map_err(|e| format!("[DOCS_HIGHLIGHT] {e}"))?;
    // Theme scopes may split the same line at different boundaries. Intersect
    // both streams, retaining exact UTF-8 slices rather than retokenizing text.
    let (mut l, mut d, mut lo, mut doff) = (0, 0, 0, 0);
    let mut tokens = vec![];
    while l < light.len() && d < dark.len() {
      let len = (light[l].1.len() - lo).min(dark[d].1.len() - doff);
      if len > 0 {
        tokens.push(Token {
          text: light[l].1[lo..lo + len].into(),
          light: color(light[l].0),
          dark: color(dark[d].0),
        });
      }
      lo += len;
      doff += len;
      if lo == light[l].1.len() {
        l += 1;
        lo = 0;
      }
      if doff == dark[d].1.len() {
        d += 1;
        doff = 0;
      }
    }
    if tokens.iter().map(|t| t.text.as_str()).collect::<String>() != line {
      return Err("[DOCS_HIGHLIGHT] token stream changed source text".into());
    }
    lines.push(tokens);
  }
  Ok(Highlighted {
    lines,
    known_language,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn line_ranges_validate_bounds_before_expansion() {
    assert_eq!(
      parse_line_ranges(" 1, 3 - 4,3 ", 4)
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>(),
      [1, 3, 4]
    );
    for value in [
      "",
      "0",
      "2-1",
      "1-",
      "1,,2",
      "-1",
      "1-18446744073709551615",
      "184467440737095516160",
    ] {
      assert!(parse_line_ranges(value, 4).is_err(), "{value}");
    }
    assert!(parse_line_ranges("1", 0).is_err());
  }
  #[test]
  fn preserves_unicode_crlf_and_reports_unknown_language() {
    let code = "const 世界 = 'hello';\r\n// comment\n";
    for language in [
      "ts",
      "tsx",
      "jsx",
      "js",
      "rust",
      "json",
      "sh",
      "html",
      "css",
      "no-such-language",
    ] {
      let value = highlight(code, language, &HighlightOptions::default()).unwrap();
      assert_eq!(
        value
          .lines
          .iter()
          .flatten()
          .map(|t| t.text.as_str())
          .collect::<String>(),
        code
      );
      assert_eq!(
        value.known_language,
        language != "no-such-language",
        "{language}"
      );
    }
    let value = highlight(code, "ts", &HighlightOptions::default()).unwrap();
    assert!(value.lines.iter().flatten().any(|t| t.light != t.dark));
    assert!(
      value
        .lines
        .iter()
        .flatten()
        .map(|t| &t.light)
        .collect::<std::collections::HashSet<_>>()
        .len()
        > 1
    );
  }
}
