//! Native highlighting shared with content transforms; no JavaScript execution.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::LazyLock;
use two_face::{
  re_exports::syntect::{
    easy::HighlightLines,
    highlighting::{HighlightIterator, HighlightState, Highlighter, Style},
    parsing::{ParseState, ScopeStack, SyntaxSet},
    util::LinesWithEndings,
  },
  theme::{EmbeddedLazyThemeSet, EmbeddedThemeName},
};

mod editor_theme;
mod farm_theme;
mod theme_config;
pub use theme_config::{CodeThemeConfig, HexColor, TokenColors, TokenOverrides};
mod static_html;
pub use static_html::render_static_lines;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeName {
  InspiredGithub,
  Base16OceanDark,
  VscodeLight,
  VscodeDark,
  FarmLight,
  FarmDark,
}

impl ThemeName {
  fn theme(
    self,
    themes: &EmbeddedLazyThemeSet,
  ) -> Result<&two_face::re_exports::syntect::highlighting::Theme, String> {
    match self {
      Self::InspiredGithub => Ok(&themes[EmbeddedThemeName::InspiredGithub]),
      Self::Base16OceanDark => Ok(&themes[EmbeddedThemeName::Base16OceanDark]),
      Self::VscodeLight => EDITOR_LIGHT.as_ref().map_err(Clone::clone),
      Self::VscodeDark => EDITOR_DARK.as_ref().map_err(Clone::clone),
      Self::FarmLight => FARM_LIGHT.as_ref().map_err(Clone::clone),
      Self::FarmDark => FARM_DARK.as_ref().map_err(Clone::clone),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct HighlightOptions {
  pub light: ThemeName,
  pub dark: ThemeName,
  pub representation: HighlightRepresentation,
  pub tokens: TokenOverrides,
}

/// Generated code-block representation. JSX remains the compatibility default.
/// Static HTML contains only escaped, compiler-generated line/token spans.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HighlightRepresentation {
  #[default]
  Jsx,
  StaticHtml,
}

impl Default for HighlightOptions {
  fn default() -> Self {
    Self {
      light: ThemeName::VscodeLight,
      dark: ThemeName::VscodeDark,
      representation: HighlightRepresentation::Jsx,
      tokens: TokenOverrides::default(),
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
// Each linked native library owns its immutable theme set. Per-block parser and
// highlighter state remains local; only lazy deserialization results are reused.
static THEMES: LazyLock<EmbeddedLazyThemeSet> = LazyLock::new(two_face::theme::extra);
static FARM_LIGHT: LazyLock<Result<two_face::re_exports::syntect::highlighting::Theme, String>> =
  LazyLock::new(|| farm_theme::theme(false));
static FARM_DARK: LazyLock<Result<two_face::re_exports::syntect::highlighting::Theme, String>> =
  LazyLock::new(|| farm_theme::theme(true));
fn selected_theme<'a>(
  name: ThemeName,
  colors: &TokenColors,
  themes: &'a EmbeddedLazyThemeSet,
) -> Result<Cow<'a, two_face::re_exports::syntect::highlighting::Theme>, String> {
  let base = name.theme(themes)?;
  if colors.is_empty() {
    return Ok(Cow::Borrowed(base));
  }
  let mut customized = base.clone();
  colors.apply(&mut customized)?;
  Ok(Cow::Owned(customized))
}
static EDITOR_LIGHT: LazyLock<Result<two_face::re_exports::syntect::highlighting::Theme, String>> =
  LazyLock::new(|| editor_theme::theme(false));
static EDITOR_DARK: LazyLock<Result<two_face::re_exports::syntect::highlighting::Theme, String>> =
  LazyLock::new(|| editor_theme::theme(true));

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
  if single_parse_enabled(
    std::env::var("FARM_DOCS_HIGHLIGHT_SINGLE_PARSE")
      .ok()
      .as_deref(),
  ) {
    highlight_single_parse_with_themes(code, language, options, &THEMES)
  } else {
    highlight_with_themes(code, language, options, &THEMES)
  }
}

fn single_parse_enabled(value: Option<&str>) -> bool {
  // The token-for-token dual-theme comparison and full-site paired runs cover
  // the shared grammar parser. Keep an explicit legacy switch for diagnosis.
  value.is_none() || value == Some("1")
}

fn highlight_with_themes(
  code: &str,
  language: &str,
  options: &HighlightOptions,
  themes: &EmbeddedLazyThemeSet,
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
  let light_theme = selected_theme(options.light, &options.tokens.light, themes)?;
  let dark_theme = selected_theme(options.dark, &options.tokens.dark, themes)?;
  let mut light = HighlightLines::new(syntax, light_theme.as_ref());
  let mut dark = HighlightLines::new(syntax, dark_theme.as_ref());
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

/// Experimental, block-local grammar parser with two independent theme states.
/// Keep the original dual-HighlightLines path independent as the default.
fn highlight_single_parse_with_themes(
  code: &str,
  language: &str,
  options: &HighlightOptions,
  themes: &EmbeddedLazyThemeSet,
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
  let mut parser = ParseState::new(syntax);
  let light_theme = selected_theme(options.light, &options.tokens.light, themes)?;
  let dark_theme = selected_theme(options.dark, &options.tokens.dark, themes)?;
  let light_highlighter = Highlighter::new(light_theme.as_ref());
  let dark_highlighter = Highlighter::new(dark_theme.as_ref());
  let mut light_state = HighlightState::new(&light_highlighter, ScopeStack::new());
  let mut dark_state = HighlightState::new(&dark_highlighter, ScopeStack::new());
  let mut lines = vec![];
  for line in LinesWithEndings::from(code) {
    let operations = parser.parse_line(line, &SYNTAXES).map_err(|error| {
      // Match HighlightLines' ParsingError -> syntect::Error display contract.
      format!(
        "[DOCS_HIGHLIGHT] {}",
        two_face::re_exports::syntect::Error::from(error)
      )
    })?;
    // Exhaust both iterators, including trailing scope operations. All mutable
    // parser/theme state is retained across lines only within this code block.
    let light: Vec<_> =
      HighlightIterator::new(&mut light_state, &operations, line, &light_highlighter).collect();
    let dark: Vec<_> =
      HighlightIterator::new(&mut dark_state, &operations, line, &dark_highlighter).collect();
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

  fn assert_same(actual: Highlighted, expected: Highlighted) {
    assert_eq!(actual.known_language, expected.known_language);
    assert_eq!(actual.lines, expected.lines);
  }

  #[test]
  fn single_parse_is_default_with_explicit_legacy_switch() {
    assert!(single_parse_enabled(None));
    assert!(single_parse_enabled(Some("1")));
    for value in [Some(""), Some("0"), Some("true"), Some(" 1"), Some("1 ")] {
      assert!(!single_parse_enabled(value));
    }
  }

  fn strategy_examples() -> [(&'static str, &'static str); 18] {
    [
      (
        "typescript",
        "/* comment\ncontinued */ const 雪 = '😀';\r\nconst n = 1;\n",
      ),
      (
        "ts",
        "const tpl = `hello\n${1 + 2}`;\nconst re = /[a-z]+/g;",
      ),
      (
        "tsx",
        "export const x = <div title=\"雪\">Hello 😀</div>;\n",
      ),
      ("jsx", "const x = <><span>{value}</span></>;\r\n"),
      ("javascript", "// comment\r\nconst x = /a+/g;\n"),
      (
        "rust",
        "/* nested /* comment */ */\nfn main() { println!(\"雪\"); }",
      ),
      ("json", "{\r\n  \"text\": \"雪😀\",\n  \"n\": 42\r\n}\n"),
      ("yaml", "title: 雪\nbody: |\n  one\n  two\nitems: [a, b]\n"),
      (
        "html",
        "<style>body { color: red; }</style>\n<script>const x = `a\n${1}`;</script>\n<div>&amp;</div>\n",
      ),
      (
        "css",
        "/* open\nclose */\na:hover { content: '雪'; color: #abc; }\n",
      ),
      (
        "bash",
        "#!/bin/sh\ncat <<'EOF'\n$HOME 雪\nEOF\nprintf '%s' \"$HOME\"\n",
      ),
      ("python", "text = \"\"\"first\nsecond\"\"\"\nprint(text)\n"),
      ("sql", "-- comment\nSELECT '雪', 1 /* end */;\n"),
      ("no-such-language", "<unknown>雪 & 😀\r\nlast"),
      ("text", "plain <script> & 雪\r\n"),
      ("", "plain\r\n\nlast"),
      ("ts", "\r\n\n"),
      ("ts", ""),
    ]
  }

  #[test]
  fn single_parse_matches_independent_original_dual_parser() {
    for (light, dark) in [
      (ThemeName::InspiredGithub, ThemeName::Base16OceanDark),
      (ThemeName::Base16OceanDark, ThemeName::InspiredGithub),
      (ThemeName::InspiredGithub, ThemeName::InspiredGithub),
      (ThemeName::Base16OceanDark, ThemeName::Base16OceanDark),
      (ThemeName::VscodeLight, ThemeName::VscodeDark),
      (ThemeName::VscodeDark, ThemeName::VscodeLight),
      (ThemeName::VscodeLight, ThemeName::VscodeLight),
      (ThemeName::VscodeDark, ThemeName::VscodeDark),
    ] {
      for representation in [
        HighlightRepresentation::Jsx,
        HighlightRepresentation::StaticHtml,
      ] {
        let options = HighlightOptions {
          light,
          dark,
          representation,
          tokens: TokenOverrides::default(),
        };
        for (language, code) in strategy_examples() {
          // This independent old implementation still owns two HighlightLines.
          let expected = highlight_with_themes(code, language, &options, &THEMES).unwrap();
          let actual =
            highlight_single_parse_with_themes(code, language, &options, &THEMES).unwrap();
          assert_same(actual, expected);
        }
      }
    }
  }

  #[test]
  fn single_parse_blocks_reset_and_concurrent_strategies_match_legacy() {
    let workers: Vec<_> = (0..4)
      .map(|worker| {
        std::thread::spawn(move || {
          let options = if worker % 2 == 0 {
            HighlightOptions::default()
          } else {
            HighlightOptions {
              light: ThemeName::Base16OceanDark,
              dark: ThemeName::InspiredGithub,
              ..Default::default()
            }
          };
          for (language, unfinished, independent) in [
            ("ts", "/* unfinished comment\n", "const reset = '雪';\r\n"),
            (
              "html",
              "<script>const x = `unfinished\n",
              "<div>reset😀</div>\n",
            ),
            ("rust", "/* unfinished\n", "fn reset() {}"),
          ] {
            assert_same(
              highlight_single_parse_with_themes(unfinished, language, &options, &THEMES).unwrap(),
              highlight_with_themes(unfinished, language, &options, &THEMES).unwrap(),
            );
            assert_same(
              highlight_single_parse_with_themes(independent, language, &options, &THEMES).unwrap(),
              highlight_with_themes(independent, language, &options, &THEMES).unwrap(),
            );
          }
        })
      })
      .collect();
    for worker in workers {
      worker.join().unwrap();
    }
  }

  #[test]
  fn shared_themes_match_fresh_themes_for_selected_orders_and_languages() {
    let examples = [
      ("ts", "/* open comment\ncontinued */ const λ = 'x';\r\n"),
      ("tsx", "export const x = <div title=\"x\">Hello</div>;\n"),
      ("javascript", "const re = /a+/g;\nconst x = `text ${1}`;\n"),
      ("rust", "fn main() {\n  println!(\"<&>\");\n}\n"),
      ("json", "{\"text\": \"λ\", \"count\": 2}\n"),
      (
        "html",
        "<style>body { color: red; }</style>\n<div>&amp;</div>\n",
      ),
      ("css", "/* comment */\na:hover { color: #abc; }\n"),
      ("bash", "#!/bin/sh\nprintf '%s' \"$HOME\"\n"),
      ("text", "plain <script> & λ\r\n"),
      ("no-such-language", "unchanged <unknown>\n"),
      ("ts", ""),
    ];
    for (light, dark) in [
      (ThemeName::InspiredGithub, ThemeName::Base16OceanDark),
      (ThemeName::Base16OceanDark, ThemeName::InspiredGithub),
      (ThemeName::InspiredGithub, ThemeName::InspiredGithub),
      (ThemeName::Base16OceanDark, ThemeName::Base16OceanDark),
      (ThemeName::VscodeLight, ThemeName::VscodeDark),
      (ThemeName::VscodeDark, ThemeName::VscodeLight),
      (ThemeName::VscodeLight, ThemeName::VscodeLight),
      (ThemeName::VscodeDark, ThemeName::VscodeDark),
    ] {
      let options = HighlightOptions {
        light,
        dark,
        ..Default::default()
      };
      for (language, code) in examples {
        // Exactly the former per-call theme initialization, with the same token
        // algorithm. Compare every token's text and both colors, not just bytes.
        let fresh = two_face::theme::extra();
        assert_same(
          highlight(code, language, &options).unwrap(),
          highlight_with_themes(code, language, &options, &fresh).unwrap(),
        );
      }
    }
  }

  #[test]
  fn shared_theme_initialization_is_send_sync_and_block_state_is_isolated() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<EmbeddedLazyThemeSet>();
    assert_send_sync::<LazyLock<EmbeddedLazyThemeSet>>();
    let workers: Vec<_> = (0..4)
      .map(|worker| {
        std::thread::spawn(move || {
          let options = if worker % 2 == 0 {
            HighlightOptions::default()
          } else {
            HighlightOptions {
              light: ThemeName::Base16OceanDark,
              dark: ThemeName::InspiredGithub,
              ..Default::default()
            }
          };
          for (language, unfinished, independent) in [
            ("ts", "/* unfinished comment\n", "const answer = 42;\n"),
            (
              "html",
              "<script>const x = `unfinished\n",
              "<div>independent</div>\n",
            ),
            ("rust", "/* unfinished\n", "fn independent() {}\n"),
          ] {
            let _ = highlight(unfinished, language, &options).unwrap();
            let fresh = two_face::theme::extra();
            assert_same(
              highlight(independent, language, &options).unwrap(),
              highlight_with_themes(independent, language, &options, &fresh).unwrap(),
            );
          }
          // The pointers identify immutable themes within this one linked library,
          // not an assumption of sharing Rust objects across dynamic libraries.
          (
            &THEMES[EmbeddedThemeName::InspiredGithub] as *const _ as usize,
            &THEMES[EmbeddedThemeName::Base16OceanDark] as *const _ as usize,
          )
        })
      })
      .collect();
    let addresses: Vec<_> = workers
      .into_iter()
      .map(|worker| worker.join().unwrap())
      .collect();
    assert!(addresses.windows(2).all(|pair| pair[0] == pair[1]));
  }
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
  #[test]
  fn editor_palette_covers_tsx_units_without_changing_source() {
    let source = "interface User { name: string; }\nclass Service<T> {\n  async load(id: number): Promise<void> {\n    const item: T | null = null; // comment\n    console.log(\"hello\", id);\n  }\n}\nconst view = <Button disabled={false}>Hello</Button>;\n";
    let options = HighlightOptions::default();
    let highlighted = highlight_single_parse_with_themes(source, "tsx", &options, &THEMES).unwrap();
    assert!(highlighted.known_language);
    assert_eq!(
      highlighted
        .lines
        .iter()
        .flatten()
        .map(|token| token.text.as_str())
        .collect::<String>(),
      source
    );
    let find = |text: &str| {
      highlighted
        .lines
        .iter()
        .flatten()
        .find(|token| token.text == text)
        .unwrap()
    };
    for name in ["User", "Service", "Promise", "T", "Button"] {
      assert_eq!(
        (&find(name).light[..], &find(name).dark[..]),
        ("#267f99", "#4ec9b0"),
        "{name}"
      );
    }
    assert_eq!(
      (&find("void").light[..], &find("void").dark[..]),
      ("#0000ff", "#569cd6")
    );
    assert_eq!(
      (&find("load").light[..], &find("load").dark[..]),
      ("#795e26", "#dcdcaa")
    );
    assert_eq!(
      (&find("id").light[..], &find("id").dark[..]),
      ("#001080", "#9cdcfe")
    );
    assert_eq!(find("hello").dark, "#ce9178");
    assert_eq!(find(" comment").dark, "#6a9955");
    assert_eq!(
      highlighted.lines,
      highlight_with_themes(source, "tsx", &options, &THEMES)
        .unwrap()
        .lines
    );
  }
}
