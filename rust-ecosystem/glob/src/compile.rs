use serde::{Deserialize, Serialize};

use crate::utils::is_path_separator;

fn default_true() -> bool {
  true
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct CompileOptions {
  pub bash: bool,
  #[serde(default)]
  pub basename: bool,
  pub contains: bool,
  pub dot: bool,
  #[serde(default)]
  pub flags: String,
  #[serde(skip)]
  pub literal_plus_quantifier: bool,
  #[serde(default)]
  pub match_base: bool,
  pub nobrace: bool,
  pub nobracket: bool,
  pub noextglob: bool,
  pub noglobstar: bool,
  pub nocase: bool,
  pub nonegate: bool,
  #[serde(default = "default_true")]
  pub posix: bool,
  pub strict_brackets: bool,
  pub strict_slashes: bool,
  pub unescape: bool,
  pub windows: bool,
  pub regex: bool,
  #[serde(default)]
  pub keep_quotes: bool,
  #[serde(default)]
  pub max_length: Option<usize>,
}

impl Default for CompileOptions {
  fn default() -> Self {
    Self {
      bash: false,
      basename: false,
      contains: false,
      dot: false,
      flags: String::new(),
      literal_plus_quantifier: false,
      match_base: false,
      nobrace: false,
      nobracket: false,
      noextglob: false,
      noglobstar: false,
      nocase: false,
      nonegate: false,
      posix: true,
      strict_brackets: false,
      strict_slashes: false,
      unescape: false,
      windows: false,
      regex: false,
      keep_quotes: false,
      max_length: None,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParseState {
  pub input: String,
  pub output: String,
  pub negated: bool,
  pub fastpaths: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tokens: Option<Vec<ParseToken>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegexDescriptor {
  pub source: String,
  pub flags: String,
  pub output: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub state: Option<ParseState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParseToken {
  #[serde(rename = "type")]
  pub kind: String,
  pub value: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub output: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TokenKind {
  None,
  Literal,
  Bracket,
  Group,
  RegexEscape,
  Wildcard,
}

fn escape_regex_char(ch: char) -> &'static str {
  match ch {
    '$' => "\\$",
    '(' => "\\(",
    ')' => "\\)",
    '*' => "\\*",
    '+' => "\\+",
    '.' => "\\.",
    '?' => "\\?",
    '[' => "\\[",
    ']' => "\\]",
    '^' => "\\^",
    '{' => "\\{",
    '|' => "\\|",
    '}' => "\\}",
    _ => "",
  }
}

fn slash_literal(options: &CompileOptions) -> &'static str {
  if options.windows {
    r"[\\/]"
  } else {
    "/"
  }
}

fn qmark(options: &CompileOptions) -> &'static str {
  if options.windows {
    r"[^\\/]"
  } else {
    r"[^/]"
  }
}

fn qmark_no_dot(options: &CompileOptions) -> &'static str {
  if options.windows {
    r"[^.\\/]"
  } else {
    r"[^./]"
  }
}

fn one_char() -> &'static str {
  r"(?=.)"
}

fn star(options: &CompileOptions) -> String {
  if options.bash {
    ".*?".to_string()
  } else {
    format!("{}*?", qmark(options))
  }
}

fn segment_leader(options: &CompileOptions) -> &'static str {
  if options.dot {
    ""
  } else {
    r"(?!\.)"
  }
}

fn globstar_segment(options: &CompileOptions) -> String {
  if options.dot {
    format!(
      r"(?!\.{{1,2}}(?:{}|$)){}+",
      slash_literal(options),
      qmark(options)
    )
  } else {
    format!(r"(?!\.){}+", qmark(options))
  }
}

fn extglob_slashy_body(options: &CompileOptions) -> String {
  if options.windows {
    r"(?:(?:(?!(?:^|[\\/])\.).)*?)".to_string()
  } else {
    r"(?:(?:(?!(?:^|/)\.).)*?)".to_string()
  }
}

fn is_separator(ch: char, options: &CompileOptions) -> bool {
  if options.windows {
    is_path_separator(ch)
  } else {
    ch == '/'
  }
}

fn collect_enclosed(
  chars: &[char],
  start: usize,
  open: char,
  close: char,
) -> Option<(String, usize)> {
  let mut depth = 0usize;
  let mut index = start;

  while index < chars.len() {
    let ch = chars[index];

    if ch == '\\' {
      index += 2;
      continue;
    }

    if ch == open {
      depth += 1;
    } else if ch == close {
      depth = depth.saturating_sub(1);
      if depth == 0 {
        let inner = chars[start + 1..index].iter().collect::<String>();
        return Some((inner, index + 1));
      }
    }

    index += 1;
  }

  None
}

fn collect_bracket(chars: &[char], start: usize) -> Option<(String, usize)> {
  let mut inner = String::new();
  let mut index = start + 1;

  while index < chars.len() {
    let ch = chars[index];

    if ch == '\\' {
      inner.push(ch);
      if let Some(next) = chars.get(index + 1) {
        inner.push(*next);
        index += 2;
      } else {
        index += 1;
      }
      continue;
    }

    if ch == '[' && chars.get(index + 1) == Some(&':') {
      inner.push('[');
      inner.push(':');
      index += 2;

      while index < chars.len() {
        let current = chars[index];
        inner.push(current);

        if current == ':' && chars.get(index + 1) == Some(&']') {
          inner.push(']');
          index += 2;
          break;
        }

        index += 1;
      }

      continue;
    }

    if ch == ']' && !inner.is_empty() && inner != "^" && inner != "!" {
      return Some((inner, index + 1));
    }

    inner.push(ch);
    index += 1;
  }

  None
}

fn split_top_level(input: &str, delimiter: char) -> Vec<String> {
  let chars: Vec<char> = input.chars().collect();
  let mut parts = Vec::new();
  let mut current = String::new();
  let mut parens = 0usize;
  let mut braces = 0usize;
  let mut brackets = 0usize;
  let mut index = 0usize;

  while index < chars.len() {
    let ch = chars[index];

    if ch == '\\' {
      current.push(ch);
      if let Some(next) = chars.get(index + 1) {
        current.push(*next);
        index += 2;
        continue;
      }
    }

    match ch {
      '(' => parens += 1,
      ')' => parens = parens.saturating_sub(1),
      '{' => braces += 1,
      '}' => braces = braces.saturating_sub(1),
      '[' => brackets += 1,
      ']' => brackets = brackets.saturating_sub(1),
      _ => {}
    }

    if ch == delimiter && parens == 0 && braces == 0 && brackets == 0 {
      parts.push(current);
      current = String::new();
      index += 1;
      continue;
    }

    current.push(ch);
    index += 1;
  }

  parts.push(current);
  parts
}

fn split_top_level_range(input: &str) -> Option<(String, String)> {
  let chars: Vec<char> = input.chars().collect();
  let mut parens = 0usize;
  let mut braces = 0usize;
  let mut brackets = 0usize;
  let mut index = 0usize;
  let mut separator = None;

  while index < chars.len() {
    let ch = chars[index];

    if ch == '\\' {
      index += if index + 1 < chars.len() { 2 } else { 1 };
      continue;
    }

    match ch {
      '(' => parens += 1,
      ')' => parens = parens.saturating_sub(1),
      '{' => braces += 1,
      '}' => braces = braces.saturating_sub(1),
      '[' => brackets += 1,
      ']' => brackets = brackets.saturating_sub(1),
      '.' if chars.get(index + 1) == Some(&'.') && parens == 0 && braces == 0 && brackets == 0 => {
        if separator.is_some() {
          return None;
        }
        separator = Some(index);
        index += 2;
        continue;
      }
      _ => {}
    }

    index += 1;
  }

  let position = separator?;
  let start = chars[..position].iter().collect::<String>();
  let end = chars[position + 2..].iter().collect::<String>();

  if start.is_empty() || end.is_empty() {
    return None;
  }

  Some((start, end))
}

fn expand_numeric_range(start: i64, end: i64) -> Option<String> {
  let step = if start <= end { 1 } else { -1 };
  let count = start.abs_diff(end) + 1;
  if count > 1024 {
    return None;
  }

  let mut values = Vec::with_capacity(count as usize);
  let mut current = start;
  loop {
    values.push(escape_literal(&current.to_string()));
    if current == end {
      break;
    }
    current += step;
  }

  Some(format!("(?:{})", values.join("|")))
}

fn expand_alpha_range(start: char, end: char) -> Option<String> {
  if !start.is_ascii_alphanumeric() || !end.is_ascii_alphanumeric() {
    return None;
  }

  let start_code = start as u32;
  let end_code = end as u32;
  if start_code <= end_code {
    return Some(format!(
      "[{}-{}]",
      escape_literal(&start.to_string()),
      escape_literal(&end.to_string())
    ));
  }

  let count = start_code - end_code + 1;
  if count > 128 {
    return None;
  }

  let mut values = Vec::with_capacity(count as usize);
  for code in (end_code..=start_code).rev() {
    let ch = char::from_u32(code)?;
    values.push(escape_literal(&ch.to_string()));
  }
  Some(format!("(?:{})", values.join("|")))
}

fn compile_range(start: &str, end: &str) -> Option<String> {
  if let (Ok(left), Ok(right)) = (start.parse::<i64>(), end.parse::<i64>()) {
    return expand_numeric_range(left, right);
  }

  let mut start_chars = start.chars();
  let mut end_chars = end.chars();
  let left = start_chars.next()?;
  let right = end_chars.next()?;
  if start_chars.next().is_none() && end_chars.next().is_none() {
    return expand_alpha_range(left, right);
  }

  None
}

const POSIX_PUNCT_OUTPUT: &str = r##"\-!"#$%&'()\*+,./:;<=>?@[\]^_`{|}~"##;
const POSIX_PUNCT_ENGINE: &str = r##"\-!"#$%&'()\*+,./:;<=>?@\[\]^_`{|}~"##;

fn posix_class_source(name: &str) -> Option<&'static str> {
  match name {
    "alnum" => Some("a-zA-Z0-9"),
    "alpha" => Some("a-zA-Z"),
    "ascii" => Some(r"\x00-\x7F"),
    "blank" => Some(r" \t"),
    "cntrl" => Some(r"\x00-\x1F\x7F"),
    "digit" => Some("0-9"),
    "graph" => Some(r"\x21-\x7E"),
    "lower" => Some("a-z"),
    "print" => Some(r"\x20-\x7E "),
    "punct" => Some(POSIX_PUNCT_OUTPUT),
    "space" => Some(r" \t\r\n\v\f"),
    "upper" => Some("A-Z"),
    "word" => Some("A-Za-z0-9_"),
    "xdigit" => Some("A-Fa-f0-9"),
    _ => None,
  }
}

pub fn regex_output_for_engine(output: &str) -> String {
  output.replace(POSIX_PUNCT_OUTPUT, POSIX_PUNCT_ENGINE)
}

fn has_regex_chars(input: &str) -> bool {
  input.chars().any(|ch| {
    matches!(
      ch,
      '-' | '*' | '+' | '?' | '.' | '^' | '$' | '{' | '}' | '(' | ')' | '|' | '[' | ']'
    )
  })
}

fn escape_literal(input: &str) -> String {
  let mut output = String::with_capacity(input.len() * 2);
  for ch in input.chars() {
    let escaped = escape_regex_char(ch);
    if escaped.is_empty() {
      if ch == '\\' {
        output.push_str("\\\\");
      } else {
        output.push(ch);
      }
    } else {
      output.push_str(escaped);
    }
  }
  output
}

fn push_literal_char(output: &mut String, ch: char) {
  if ch == '\\' {
    output.push_str("\\\\");
    return;
  }

  let escaped = escape_regex_char(ch);
  if escaped.is_empty() {
    output.push(ch);
  } else {
    output.push_str(escaped);
  }
}

fn sanitize_nested_negation(input: &str, strip_terminal_anchor: bool) -> String {
  let mut output = input.to_string();

  if let Some(stripped) = output.strip_prefix("(?=.)") {
    output = stripped.to_string();
  }

  if let Some(stripped) = output.strip_suffix("/?") {
    output = stripped.to_string();
  }

  if strip_terminal_anchor {
    output = output.replace("$))", "))");
    output = output.replace("$)", ")");
  }

  output
}

fn contains_magic(input: &str) -> bool {
  let mut escaped = false;

  for ch in input.chars() {
    if escaped {
      escaped = false;
      continue;
    }

    if ch == '\\' {
      escaped = true;
      continue;
    }

    if matches!(
      ch,
      '*' | '?' | '[' | ']' | '{' | '}' | '(' | ')' | '@' | '!'
    ) {
      return true;
    }
  }

  false
}

fn dot_segment_guard(options: &CompileOptions) -> &'static str {
  if options.windows {
    r"(?!.*(?:^|[\\/])\.{1,2}(?:[\\/]|$))"
  } else {
    r"(?!.*(?:^|/)\.{1,2}(?:/|$))"
  }
}

fn has_explicit_dot_segment(input: &str, options: &CompileOptions) -> bool {
  let chars: Vec<char> = input.chars().collect();
  let mut current = String::new();
  let mut index = 0usize;

  while index <= chars.len() {
    let ch = chars.get(index).copied();

    if let Some(value) = ch {
      if value == '\\' && index + 1 < chars.len() && !options.windows {
        current.push(value);
        current.push(chars[index + 1]);
        index += 2;
        continue;
      }

      if is_separator(value, options) {
        if current == "." || current == ".." {
          return true;
        }
        current.clear();
        index += 1;
        continue;
      }

      current.push(value);
      index += 1;
      continue;
    }

    return current == "." || current == "..";
  }

  false
}

fn compile_bracket(inner: &str, options: &CompileOptions, segment_start: bool) -> String {
  if options.windows && inner == "/" {
    return slash_literal(options).to_string();
  }

  let chars: Vec<char> = inner.chars().collect();
  let mut body = String::new();
  let mut index = 0usize;
  let mut posix_converted = false;

  while index < chars.len() {
    let ch = chars[index];

    if options.posix && index == 0 && ch == '!' {
      body.push('^');
      index += 1;
      continue;
    }

    if ch == '\\' {
      body.push('\\');
      if let Some(next) = chars.get(index + 1) {
        body.push(*next);
        index += 2;
      } else {
        index += 1;
      }
      continue;
    }

    if options.posix && ch == '[' && chars.get(index + 1) == Some(&':') {
      let mut end = index + 2;
      let mut converted = false;
      while end + 1 < chars.len() {
        if chars[end] == ':' && chars[end + 1] == ']' {
          let name = chars[index + 2..end].iter().collect::<String>();
          if let Some(source) = posix_class_source(&name) {
            body.push_str(source);
            posix_converted = true;
            index = end + 2;
            converted = true;
          }
          break;
        }
        end += 1;
      }

      if converted {
        continue;
      }
    }

    if ch == '[' && (!options.posix || chars.get(index + 1) != Some(&':')) {
      body.push_str("\\[");
      index += 1;
      continue;
    }

    if ch == '-' && index + 1 == chars.len() {
      body.push_str("\\-");
      index += 1;
      continue;
    }

    if ch == ']' && body.is_empty() {
      body.push_str("\\]");
      index += 1;
      continue;
    }

    body.push(ch);
    index += 1;
  }

  let class_body = if body.starts_with('^') && !body.contains('/') && !posix_converted {
    format!("{body}/")
  } else {
    body
  };
  let class_output = format!("[{class_body}]");
  let prefix = if posix_converted && segment_start {
    one_char()
  } else {
    ""
  };

  if posix_converted || has_regex_chars(inner) {
    return format!("{prefix}{class_output}");
  }

  let literal = escape_literal(&format!("[{inner}]"));
  format!("{prefix}(?:{literal}|{class_output})")
}

fn compile_extglob(operator: char, inner: &str, options: &CompileOptions) -> Option<String> {
  let mut inner_options = options.clone();
  inner_options.literal_plus_quantifier = true;
  let alternatives = split_top_level(inner, '|')
    .into_iter()
    .map(|part| compile_body(&part, &inner_options))
    .collect::<Option<Vec<_>>>()?;
  let body = alternatives.join("|");

  if body.is_empty() {
    return Some(String::new());
  }

  let output = match operator {
    '@' => format!("(?:{body})"),
    '*' => format!("(?:{body})*"),
    '+' => format!("(?:{body})+"),
    '?' => format!("(?:{body})?"),
    _ => return None,
  };

  Some(output)
}

fn compile_group(inner: &str, options: &CompileOptions) -> Option<String> {
  if inner.starts_with('?') {
    return Some(format!("({inner})"));
  }

  let mut inner_options = options.clone();
  inner_options.literal_plus_quantifier = true;
  let alternatives = split_top_level(inner, '|')
    .into_iter()
    // Picomatch ordinary groups preserve regex-group semantics: their
    // first wildcard does not start a new filesystem segment.
    .map(|part| compile_body_with_context(&part, &inner_options, false))
    .collect::<Option<Vec<_>>>()?;

  Some(format!("({})", alternatives.join("|")))
}

fn is_regex_escape(next: char) -> bool {
  next.is_ascii_digit()
    || matches!(
      next,
      'b'
        | 'B'
        | 'd'
        | 'D'
        | 'f'
        | 'n'
        | 'p'
        | 'P'
        | 'r'
        | 's'
        | 'S'
        | 't'
        | 'u'
        | 'v'
        | 'w'
        | 'W'
        | 'x'
        | 'k'
        | '0'
    )
}

fn is_escaped_literal_in_windows(next: char) -> bool {
  matches!(
    next,
    '*' | '?' | '+' | '@' | '!' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '.' | ';' | '"' | '\''
  )
}

fn compile_body_with_context(
  input: &str,
  options: &CompileOptions,
  initial_segment_start: bool,
) -> Option<String> {
  let mut pattern = if initial_segment_start {
    input.strip_prefix("./").unwrap_or(input)
  } else {
    input
  };
  let mut optional_trailing_slash = false;
  let mut optional_descendants = false;

  if let Some(stripped) = pattern.strip_suffix("{,/}") {
    optional_trailing_slash = true;
    pattern = stripped;
  } else if let Some(stripped) = pattern.strip_suffix("{,/**}") {
    optional_descendants = true;
    pattern = stripped;
  }

  let mut output = String::with_capacity(pattern.len() * 2);
  let chars: Vec<char> = pattern.chars().collect();
  let mut index = 0;
  let mut segment_start = initial_segment_start;
  let mut last_was_wildcard = false;
  let mut last_segment_token_kind = TokenKind::None;
  let mut last_token_kind = TokenKind::None;

  while index < chars.len() {
    let ch = chars[index];

    if ch == '\\' {
      if let Some(next) = chars.get(index + 1).copied() {
        let mut slash_run = 1usize;
        while chars.get(index + slash_run) == Some(&'\\') {
          slash_run += 1;
        }

        if slash_run > 3 {
          let next_index = index + slash_run;
          if let Some(collapsed_next) = chars.get(next_index).copied() {
            if options.unescape {
              if options.windows && collapsed_next.is_ascii_alphanumeric() {
                output.push_str(&format!("(?:{}+)?", slash_literal(options)));
              }
              push_literal_char(&mut output, collapsed_next);
            } else {
              push_literal_char(&mut output, '\\');
              push_literal_char(&mut output, collapsed_next);
            }

            segment_start = false;
            last_was_wildcard = false;
            last_token_kind = TokenKind::Literal;
            index = next_index + 1;
            continue;
          }
        }

        if options.unescape && matches!(next, '{' | '}') {
          output.push(next);
          segment_start = false;
          last_was_wildcard = false;
          last_token_kind = TokenKind::Literal;
          index += 2;
          continue;
        }

        if options.windows && next == '/' {
          output.push_str(slash_literal(options));
          segment_start = true;
          last_was_wildcard = false;
          last_token_kind = TokenKind::None;
          index += 2;
          continue;
        }

        if options.unescape && next.is_ascii_alphanumeric() {
          if options.windows {
            output.push_str(&format!("(?:{}+)?", slash_literal(options)));
          }
          push_literal_char(&mut output, next);
          segment_start = false;
          last_was_wildcard = false;
          last_token_kind = TokenKind::Literal;
          index += 2;
          continue;
        }

        if is_regex_escape(next) {
          output.push('\\');
          output.push(next);
          segment_start = false;
          last_was_wildcard = false;
          last_token_kind = TokenKind::RegexEscape;
          index += 2;
          continue;
        }

        if options.windows && is_escaped_literal_in_windows(next) {
          push_literal_char(&mut output, next);
          segment_start = false;
          last_was_wildcard = false;
          last_token_kind = TokenKind::Literal;
          index += 2;
          continue;
        }

        if options.windows {
          output.push_str("\\\\");
          last_segment_token_kind = last_token_kind;
          segment_start = false;
          last_was_wildcard = false;
          last_token_kind = TokenKind::Literal;
          index += 1;
          continue;
        }

        if next == '\\' && !options.windows {
          output.push_str("\\\\");
          segment_start = false;
          last_was_wildcard = false;
          last_token_kind = TokenKind::Literal;
          index += 2;
          continue;
        }

        let escaped = escape_regex_char(next);
        if escaped.is_empty() {
          output.push(next);
        } else {
          output.push_str(escaped);
        }
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::Literal;
        index += 2;
        continue;
      }

      output.push_str("\\\\");
      segment_start = false;
      last_was_wildcard = false;
      last_token_kind = TokenKind::Literal;
      index += 1;
      continue;
    }

    if is_separator(ch, options) {
      output.push_str(slash_literal(options));
      last_segment_token_kind = last_token_kind;
      segment_start = true;
      last_was_wildcard = false;
      last_token_kind = TokenKind::None;
      index += 1;
      continue;
    }

    if ch == '"' {
      let mut next_index = index + 1;
      let mut literal = String::new();

      while next_index < chars.len() && chars[next_index] != '"' {
        literal.push(chars[next_index]);
        next_index += 1;
      }

      if next_index < chars.len() {
        if options.keep_quotes {
          output.push_str(r#"\""#);
          output.push_str(&escape_literal(&literal));
          output.push_str(r#"\""#);
        } else {
          output.push_str(&escape_literal(&literal));
        }
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::Literal;
        index = next_index + 1;
        continue;
      }
    }

    if ch == '!' && index == 0 && !options.nonegate && chars.get(1) != Some(&'(') {
      output.push('!');
      last_was_wildcard = false;
      segment_start = false;
      last_token_kind = TokenKind::Literal;
      index += 1;
      continue;
    }

    if ch == '[' {
      if options.nobracket {
        output.push_str("\\[");
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::Literal;
        index += 1;
        continue;
      }

      let Some((inner, next_index)) = collect_bracket(&chars, index) else {
        if options.strict_brackets {
          return None;
        }
        let remaining = chars[index..].iter().collect::<String>();
        if remaining.starts_with("[[:") {
          output.push_str(&remaining);
          segment_start = false;
          last_was_wildcard = false;
          last_token_kind = TokenKind::Bracket;
          index = chars.len();
          continue;
        }
        output.push_str("\\[");
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::Literal;
        index += 1;
        continue;
      };
      output.push_str(&compile_bracket(&inner, options, segment_start));
      segment_start = false;
      last_was_wildcard = true;
      last_token_kind = TokenKind::Bracket;
      index = next_index;
      continue;
    }

    if ch == '!'
      && chars.get(index + 1) == Some(&'(')
      && !options.noextglob
      && (chars.get(index + 2) != Some(&'?')
        || !matches!(chars.get(index + 3), Some('!' | '=' | '<' | ':')))
    {
      let (inner, next_index) = collect_enclosed(&chars, index + 1, '(', ')')?;
      let rest = chars[next_index..].iter().collect::<String>();

      if rest == "/**" && !inner.contains('/') && !inner.contains('\\') {
        let compiled_inner = compile_body(&inner, options)?;
        output.push_str(&format!(
          "(?:(?!(?:{})){})(?:{}{}|$)",
          compiled_inner,
          star(options),
          slash_literal(options),
          extglob_slashy_body(options)
        ));
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::Group;
        index = chars.len();
        continue;
      }

      let compiled_inner = if inner.ends_with("/**") {
        let prefix = &inner[..inner.len().saturating_sub(3)];
        let compiled_prefix = compile_body(prefix, options)?;
        format!(
          "{compiled_prefix}{}{body}",
          slash_literal(options),
          body = extglob_slashy_body(options)
        )
      } else if inner.len() == 1 && matches!(inner.chars().next(), Some('.' | '+' | '?' | '@')) {
        escape_literal(&inner)
      } else {
        let mut inner_options = options.clone();
        inner_options.literal_plus_quantifier = true;
        split_top_level(&inner, '|')
          .into_iter()
          .map(|part| {
            compile_body(&part, &inner_options)
              .map(|value| sanitize_nested_negation(&value, !rest.is_empty()))
          })
          .collect::<Option<Vec<_>>>()?
          .join("|")
      };
      let slashy = (inner.contains('/') || inner.contains('\\')) && inner != "/";
      let body_output = if slashy {
        extglob_slashy_body(options)
      } else {
        star(options)
      };
      let lookahead = if rest.starts_with('.')
        && !matches!(rest.chars().nth(1), Some('!'))
        && inner.contains('*')
        && !slashy
      {
        format!("{}{}", compiled_inner, compile_body(&rest, options)?)
      } else if slashy || next_index == chars.len() {
        format!("(?:{compiled_inner})$")
      } else {
        compiled_inner.clone()
      };

      if index == 0 {
        output.push_str("(?=.)");
      }

      if slashy || next_index == chars.len() {
        output.push_str(&format!("(?:(?!(?:{}))){}", lookahead, body_output));
      } else {
        output.push_str(&format!("(?:(?!(?:{})){})", lookahead, body_output));
      }
      segment_start = false;
      last_was_wildcard = false;
      last_token_kind = TokenKind::Group;
      index = next_index;
      continue;
    }

    if ch == '|' {
      output.push('|');
      segment_start = false;
      last_was_wildcard = false;
      last_token_kind = TokenKind::None;
      index += 1;
      continue;
    }

    if ch == '{' {
      if options.nobrace {
        output.push_str(r"\{");
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::Literal;
        index += 1;
        continue;
      }

      let (inner, next_index) = collect_enclosed(&chars, index, '{', '}')?;
      let comma_parts = split_top_level(&inner, ',');
      let range = split_top_level_range(&inner);

      if comma_parts.len() == 1 && range.is_none() {
        output.push_str(r"\{");
        output.push_str(&escape_literal(&inner));
        output.push_str(r"\}");
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::Literal;
        index = next_index;
        continue;
      }

      if let Some((start, end)) = range {
        output.push_str(&compile_range(&start, &end)?);
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::Group;
        index = next_index;
        continue;
      }

      let alternatives = comma_parts
        .into_iter()
        .map(|part| compile_body_with_context(&part, options, segment_start))
        .collect::<Option<Vec<_>>>()?;
      output.push_str(&format!("(?:{})", alternatives.join("|")));
      segment_start = false;
      last_was_wildcard = false;
      last_token_kind = TokenKind::Group;
      index = next_index;
      continue;
    }

    if ch == '@' && chars.get(index + 1) == Some(&'(') && !options.noextglob {
      let (inner, next_index) = collect_enclosed(&chars, index + 1, '(', ')')?;
      output.push_str(&compile_extglob('@', &inner, options)?);
      segment_start = false;
      last_was_wildcard = false;
      last_token_kind = TokenKind::Group;
      index = next_index;
      continue;
    }

    if ch == '(' {
      let Some((inner, next_index)) = collect_enclosed(&chars, index, '(', ')') else {
        if options.strict_brackets {
          return None;
        }

        output.push_str(r"\(");
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::Literal;
        index += 1;
        continue;
      };
      output.push_str(&compile_group(&inner, options)?);
      segment_start = false;
      last_was_wildcard = false;
      last_token_kind = TokenKind::Group;
      index = next_index;
      continue;
    }

    if matches!(ch, ')' | '}') {
      if options.strict_brackets {
        return None;
      }

      let escaped = escape_regex_char(ch);
      output.push_str(escaped);
      segment_start = false;
      last_was_wildcard = false;
      last_token_kind = TokenKind::Literal;
      index += 1;
      continue;
    }

    if ch == '?' {
      if chars.get(index + 1) == Some(&'(')
        && chars.get(index + 2) != Some(&'?')
        && !options.noextglob
      {
        let (inner, next_index) = collect_enclosed(&chars, index + 1, '(', ')')?;
        output.push_str(&compile_extglob('?', &inner, options)?);
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::Group;
        index = next_index;
        continue;
      }

      if matches!(
        last_token_kind,
        TokenKind::Bracket | TokenKind::Group | TokenKind::RegexEscape
      ) {
        output.push('?');
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::None;
        index += 1;
        continue;
      }

      if segment_start && !options.dot {
        output.push_str(segment_leader(options));
        output.push_str(qmark_no_dot(options));
      } else {
        output.push_str(qmark(options));
      }
      segment_start = false;
      last_was_wildcard = false;
      last_token_kind = TokenKind::Wildcard;
      index += 1;
      continue;
    }

    if ch == '+' {
      if chars.get(index + 1) == Some(&'(')
        && chars.get(index + 2) != Some(&'?')
        && !options.noextglob
      {
        let (inner, next_index) = collect_enclosed(&chars, index + 1, '(', ')')?;
        output.push_str(&compile_extglob('+', &inner, options)?);
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::Group;
        index = next_index;
        continue;
      }

      if matches!(
        last_token_kind,
        TokenKind::Bracket | TokenKind::Group | TokenKind::RegexEscape
      ) || ((options.regex || options.literal_plus_quantifier)
        && matches!(last_token_kind, TokenKind::Literal))
      {
        output.push('+');
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::None;
      } else {
        output.push_str(r"\+");
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::Literal;
      }
      index += 1;
      continue;
    }

    if ch == '*' {
      if chars.get(index + 1) == Some(&'(')
        && chars.get(index + 2) != Some(&'?')
        && !options.noextglob
      {
        let (inner, next_index) = collect_enclosed(&chars, index + 1, '(', ')')?;
        output.push_str(&compile_extglob('*', &inner, options)?);
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::Group;
        index = next_index;
        continue;
      }

      if options.regex
        && matches!(
          last_token_kind,
          TokenKind::Bracket | TokenKind::Group | TokenKind::RegexEscape
        )
      {
        output.push('*');
        segment_start = false;
        last_was_wildcard = false;
        last_token_kind = TokenKind::None;
        index += 1;
        continue;
      }

      let mut stars = 1;
      while index + stars < chars.len() && chars[index + stars] == '*' {
        stars += 1;
      }

      if stars >= 2 && chars.get(index + 1) == Some(&'*') && chars.get(index + 2) == Some(&'(') {
        stars = 1;
      }

      let prev_is_sep = index == 0 || is_separator(chars[index - 1], options);
      let next = chars.get(index + stars).copied();
      let next_is_sep = match next {
        None => true,
        Some(value) => is_separator(value, options),
      };
      let next_is_groupish = match next {
        Some('{') | Some('(') => true,
        Some('@') => chars.get(index + stars + 1) == Some(&'(') && !options.noextglob,
        _ => false,
      };

      if stars >= 2 && prev_is_sep && next_is_sep && !options.noglobstar {
        let globstar = globstar_segment(options);
        if next.is_some() {
          let prefix = if index == 0 {
            format!("(?:{}|)", slash_literal(options))
          } else {
            String::new()
          };
          output.push_str(&format!(
            "{}(?:{}{})*",
            prefix,
            globstar,
            slash_literal(options)
          ));
          last_segment_token_kind = TokenKind::Wildcard;
          segment_start = true;
          last_was_wildcard = true;
          last_token_kind = TokenKind::Wildcard;
          index += stars + 1;
        } else {
          if prev_is_sep
            && !matches!(last_segment_token_kind, TokenKind::Wildcard)
            && !output.is_empty()
          {
            let slash = slash_literal(options);
            if options.bash {
              let new_len = output.len().saturating_sub(slash.len());
              output.truncate(new_len);
              if options.strict_slashes {
                output.push_str(&format!("{}{}", slash, extglob_slashy_body(options)));
              } else {
                output.push_str(&format!(
                  "(?:|(?:{}{}))",
                  slash,
                  extglob_slashy_body(options)
                ));
              }
            } else if options.strict_slashes {
              output.push_str(&format!("(?:{}(?:{}{})*)?", globstar, slash, globstar));
            } else {
              let new_len = output.len().saturating_sub(slash.len());
              output.truncate(new_len);
              output.push_str(&format!(
                "(?:{slash}+(?:{globstar}(?:{slash}+{globstar})*)?)?",
              ));
            }
          } else {
            let slash = slash_literal(options);
            let leading = if index == 0 {
              format!("(?:{slash}+)?")
            } else {
              String::new()
            };
            output.push_str(&format!("{leading}(?:{globstar}(?:{slash}+{globstar})*)?",));
          }
          segment_start = false;
          last_was_wildcard = true;
          last_token_kind = TokenKind::Wildcard;
          index += stars;
        }
        continue;
      }

      if stars >= 2 && prev_is_sep && next_is_groupish && !options.noglobstar {
        let slash = slash_literal(options);
        let globstar = globstar_segment(options);
        if index == 0 {
          output.push_str(&format!(
            "(?:|{}|{}(?:{}{})*)",
            slash, globstar, slash, globstar
          ));
        } else {
          output.push_str(&format!("(?:{}(?:{}{})*)?", globstar, slash, globstar));
        }
        last_segment_token_kind = TokenKind::Wildcard;
        segment_start = true;
        last_was_wildcard = true;
        last_token_kind = TokenKind::Wildcard;
        index += stars;
        continue;
      }

      if segment_start {
        output.push_str(segment_leader(options));
        if !options.bash {
          output.push_str(one_char());
        }
      }
      output.push_str(&star(options));
      segment_start = false;
      last_was_wildcard = true;
      last_token_kind = TokenKind::Wildcard;
      index += stars;
      continue;
    }

    let escaped = escape_regex_char(ch);
    if escaped.is_empty() {
      output.push(ch);
    } else {
      output.push_str(escaped);
    }

    segment_start = false;
    last_was_wildcard = false;
    last_token_kind = TokenKind::Literal;
    index += 1;
  }

  if last_was_wildcard && !options.strict_slashes && !pattern.is_empty() {
    output.push_str(&format!("(?:{}+)?", slash_literal(options)));
  }

  if optional_trailing_slash {
    output.push_str(&format!("(?:|{})", slash_literal(options)));
  }

  if optional_descendants {
    if options.bash {
      output.push_str(&format!(
        "(?:|(?:{}{}))",
        slash_literal(options),
        extglob_slashy_body(options)
      ));
    } else {
      let globstar = globstar_segment(options);
      output.push_str(&format!(
        "(?:|(?:{}(?:{}(?:{}{})*)?)?)",
        slash_literal(options),
        globstar,
        slash_literal(options),
        globstar
      ));
    }
  }

  Some(output)
}

fn compile_body(input: &str, options: &CompileOptions) -> Option<String> {
  compile_body_with_context(input, options, true)
}

fn push_parse_token(tokens: &mut Vec<ParseToken>, token: ParseToken) {
  if let Some(prev) = tokens.last_mut() {
    if prev.kind == "text" && token.kind == "text" {
      let merged_output = format!(
        "{}{}",
        prev.output.clone().unwrap_or_else(|| prev.value.clone()),
        token.value
      );
      prev.output = Some(merged_output);
      prev.value.push_str(&token.value);
      return;
    }
  }

  tokens.push(token);
}

fn parse_tokens(input: &str, options: &CompileOptions) -> Vec<ParseToken> {
  let chars = input.chars().collect::<Vec<_>>();
  let mut tokens = vec![ParseToken {
    kind: "bos".to_string(),
    value: String::new(),
    output: Some(String::new()),
  }];
  let mut braces = 0usize;
  let mut parens = 0usize;
  let mut index = 0usize;

  while index < chars.len() {
    let ch = chars[index];
    let next = chars.get(index + 1).copied();
    let prev_kind = tokens
      .last()
      .map(|token| token.kind.as_str())
      .unwrap_or("bos");

    match ch {
      '{' if !options.nobrace => {
        braces += 1;
        push_parse_token(
          &mut tokens,
          ParseToken {
            kind: "brace".to_string(),
            value: "{".to_string(),
            output: Some("(".to_string()),
          },
        );
      }
      '}' if !options.nobrace && braces > 0 => {
        braces -= 1;
        push_parse_token(
          &mut tokens,
          ParseToken {
            kind: "brace".to_string(),
            value: "}".to_string(),
            output: Some(")".to_string()),
          },
        );
      }
      ',' => {
        let output = if !options.nobrace && braces > 0 {
          "|".to_string()
        } else {
          ",".to_string()
        };
        push_parse_token(
          &mut tokens,
          ParseToken {
            kind: "comma".to_string(),
            value: ",".to_string(),
            output: Some(output),
          },
        );
      }
      '(' => {
        parens += 1;
        push_parse_token(
          &mut tokens,
          ParseToken {
            kind: "paren".to_string(),
            value: "(".to_string(),
            output: None,
          },
        );
      }
      ')' => {
        let output = if parens > 0 { ")" } else { r"\)" }.to_string();
        parens = parens.saturating_sub(1);
        push_parse_token(
          &mut tokens,
          ParseToken {
            kind: "paren".to_string(),
            value: ")".to_string(),
            output: Some(output),
          },
        );
      }
      '.' => {
        let token = if braces + parens == 0 && prev_kind != "bos" && prev_kind != "slash" {
          ParseToken {
            kind: "text".to_string(),
            value: ".".to_string(),
            output: Some(r"\.".to_string()),
          }
        } else {
          ParseToken {
            kind: "dot".to_string(),
            value: ".".to_string(),
            output: Some(r"\.".to_string()),
          }
        };
        push_parse_token(&mut tokens, token);
      }
      '*' => {
        push_parse_token(
          &mut tokens,
          ParseToken {
            kind: "star".to_string(),
            value: "*".to_string(),
            output: Some(star(options)),
          },
        );
      }
      '?' => {
        let output = if !options.dot && matches!(prev_kind, "slash" | "bos") {
          qmark_no_dot(options).to_string()
        } else {
          qmark(options).to_string()
        };
        push_parse_token(
          &mut tokens,
          ParseToken {
            kind: "qmark".to_string(),
            value: "?".to_string(),
            output: Some(output),
          },
        );
      }
      '/' => {
        push_parse_token(
          &mut tokens,
          ParseToken {
            kind: "slash".to_string(),
            value: "/".to_string(),
            output: Some(slash_literal(options).to_string()),
          },
        );
      }
      '+' => {
        push_parse_token(
          &mut tokens,
          ParseToken {
            kind: "plus".to_string(),
            value: "+".to_string(),
            output: Some(r"\+".to_string()),
          },
        );
      }
      '|' => {
        push_parse_token(
          &mut tokens,
          ParseToken {
            kind: "text".to_string(),
            value: "|".to_string(),
            output: None,
          },
        );
      }
      '\\' => {
        let mut value = String::from("\\");
        if let Some(next_char) = next {
          value.push(next_char);
          index += 1;
        }

        push_parse_token(
          &mut tokens,
          ParseToken {
            kind: "text".to_string(),
            value,
            output: None,
          },
        );
      }
      _ => {
        let mut value = ch.to_string();
        while index + 1 < chars.len()
          && !matches!(
            chars[index + 1],
            '{'
              | '}'
              | ','
              | '('
              | ')'
              | '.'
              | '*'
              | '?'
              | '/'
              | '+'
              | '|'
              | '\\'
              | '!'
              | '@'
              | '['
              | ']'
          )
        {
          index += 1;
          value.push(chars[index]);
        }

        push_parse_token(
          &mut tokens,
          ParseToken {
            kind: "text".to_string(),
            value,
            output: None,
          },
        );
      }
    }

    index += 1;
  }

  if !options.strict_slashes
    && tokens
      .last()
      .is_some_and(|token| matches!(token.kind.as_str(), "star" | "bracket"))
  {
    push_parse_token(
      &mut tokens,
      ParseToken {
        kind: "maybe_slash".to_string(),
        value: String::new(),
        output: Some(format!("{}?", slash_literal(options))),
      },
    );
  }

  tokens
}

pub fn parse(input: &str, options: &CompileOptions) -> Option<ParseState> {
  let mut pattern = input;
  let mut negated = false;

  if !options.nonegate {
    let chars: Vec<char> = input.chars().collect();
    let mut count = 0usize;
    while count < chars.len() && chars[count] == '!' {
      if chars.get(count + 1) == Some(&'(') && chars.get(count + 2) != Some(&'?') {
        break;
      }
      count += 1;
    }

    if count > 0 {
      negated = count % 2 == 1;
      pattern = &input[count..];
    }
  }

  let output = compile_body(pattern, options)?;
  Some(ParseState {
    input: input.to_string(),
    output,
    negated,
    fastpaths: false,
    tokens: Some(parse_tokens(pattern, options)),
  })
}

pub fn make_re(
  input: &str,
  options: &CompileOptions,
  return_state: bool,
) -> Option<RegexDescriptor> {
  let state = parse(input, options)?;
  let prepend = if options.contains { "" } else { "^" };
  let append = if options.contains { "" } else { "$" };
  let flags = if options.flags.is_empty() {
    if options.nocase {
      "i".to_string()
    } else {
      String::new()
    }
  } else {
    options.flags.clone()
  };
  let output = state.output.clone();
  let guard = if contains_magic(input)
    && (input.contains('/') || input.contains('\\'))
    && !has_explicit_dot_segment(input, options)
  {
    dot_segment_guard(options)
  } else {
    ""
  };
  let mut source = format!(
    "{prepend}{guard}(?:{}){append}",
    regex_output_for_engine(&output)
  );
  if state.negated {
    source = format!("^(?!{source}).*$");
  }

  Some(RegexDescriptor {
    source,
    flags,
    output,
    state: return_state.then_some(state),
  })
}
