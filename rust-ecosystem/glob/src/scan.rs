use serde::{Deserialize, Serialize};

use crate::constants::{
  CHAR_ASTERISK, CHAR_AT, CHAR_BACKWARD_SLASH, CHAR_COMMA, CHAR_DOT, CHAR_EXCLAMATION_MARK,
  CHAR_FORWARD_SLASH, CHAR_LEFT_CURLY_BRACE, CHAR_LEFT_PARENTHESES, CHAR_LEFT_SQUARE_BRACKET,
  CHAR_PLUS, CHAR_QUESTION_MARK, CHAR_RIGHT_CURLY_BRACE, CHAR_RIGHT_PARENTHESES,
  CHAR_RIGHT_SQUARE_BRACKET,
};
use crate::utils::{is_path_separator, remove_backslashes};

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct ScanOptions {
  pub parts: bool,
  pub tokens: bool,
  pub scan_to_end: bool,
  pub noext: bool,
  pub nonegate: bool,
  pub noparen: bool,
  pub unescape: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScanToken {
  pub value: String,
  pub depth: f64,
  pub is_glob: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub is_globstar: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub is_brace: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub is_bracket: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub is_extglob: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub negated: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub backslashes: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub is_prefix: Option<bool>,
}

impl Default for ScanToken {
  fn default() -> Self {
    Self {
      value: String::new(),
      depth: 0.0,
      is_glob: false,
      is_globstar: None,
      is_brace: None,
      is_bracket: None,
      is_extglob: None,
      negated: None,
      backslashes: None,
      is_prefix: None,
    }
  }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScanState {
  pub prefix: String,
  pub input: String,
  pub start: usize,
  pub base: String,
  pub glob: String,
  pub is_brace: bool,
  pub is_bracket: bool,
  pub is_glob: bool,
  pub is_extglob: bool,
  pub is_globstar: bool,
  pub negated: bool,
  pub negated_extglob: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_depth: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tokens: Option<Vec<ScanToken>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub slashes: Option<Vec<usize>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parts: Option<Vec<String>>,
}

struct InputView<'a> {
  raw: &'a str,
  chars: Vec<char>,
  offsets: Vec<usize>,
}

impl<'a> InputView<'a> {
  fn new(raw: &'a str) -> Self {
    let chars: Vec<char> = raw.chars().collect();
    let mut offsets = raw.char_indices().map(|(idx, _)| idx).collect::<Vec<_>>();
    offsets.push(raw.len());
    Self {
      raw,
      chars,
      offsets,
    }
  }

  fn len(&self) -> usize {
    self.chars.len()
  }

  fn char_at(&self, index: usize) -> Option<char> {
    self.chars.get(index).copied()
  }

  fn slice(&self, start: usize, end: usize) -> String {
    if start >= end {
      return String::new();
    }
    self.raw[self.offsets[start]..self.offsets[end]].to_string()
  }

  fn slice_from(&self, start: usize) -> String {
    self.slice(start, self.len())
  }
}

fn depth(token: &mut ScanToken) {
  if token.is_prefix != Some(true) {
    token.depth = if token.is_globstar == Some(true) {
      f64::INFINITY
    } else {
      1.0
    };
  }
}

fn is_extglob_char(ch: char) -> bool {
  matches!(
    ch,
    CHAR_PLUS | CHAR_AT | CHAR_ASTERISK | CHAR_QUESTION_MARK | CHAR_EXCLAMATION_MARK
  )
}

pub fn scan(input: &str, options: &ScanOptions) -> ScanState {
  let view = InputView::new(input);
  let scan_to_end = options.parts || options.scan_to_end;
  let mut slashes = Vec::new();
  let mut tokens = Vec::new();
  let mut parts = Vec::new();

  let length = view.len().saturating_sub(1);
  let mut index: isize = -1;
  let mut start = 0usize;
  let mut last_index = 0usize;
  let mut is_brace = false;
  let mut is_bracket = false;
  let mut is_glob = false;
  let mut is_extglob = false;
  let mut is_globstar = false;
  let mut brace_escaped = false;
  let mut backslashes = false;
  let mut negated = false;
  let mut negated_extglob = false;
  let mut finished = false;
  let mut braces = 0usize;
  let mut prev = '\0';
  let mut code = '\0';
  let mut token = ScanToken::default();

  let eos = |idx: isize| idx >= length as isize;
  let peek = |idx: isize| view.char_at((idx + 1) as usize);
  let next_char = |idx: &mut isize, prev_code: &mut char, current_code: char| -> Option<char> {
    *prev_code = current_code;
    *idx += 1;
    view.char_at(*idx as usize)
  };

  while index < length as isize {
    let Some(current) = next_char(&mut index, &mut prev, code) else {
      break;
    };
    code = current;

    if code == CHAR_BACKWARD_SLASH {
      backslashes = true;
      token.backslashes = Some(true);

      let Some(next) = next_char(&mut index, &mut prev, code) else {
        break;
      };
      code = next;

      if code == CHAR_LEFT_CURLY_BRACE {
        brace_escaped = true;
      }
      continue;
    }

    if brace_escaped || code == CHAR_LEFT_CURLY_BRACE {
      braces += 1;

      while !eos(index) {
        let Some(next) = next_char(&mut index, &mut prev, code) else {
          break;
        };
        code = next;

        if code == CHAR_BACKWARD_SLASH {
          backslashes = true;
          token.backslashes = Some(true);
          if let Some(escaped) = next_char(&mut index, &mut prev, code) {
            code = escaped;
          }
          continue;
        }

        if code == CHAR_LEFT_CURLY_BRACE {
          braces += 1;
          continue;
        }

        if !brace_escaped && code == CHAR_DOT {
          if let Some(next_code) = next_char(&mut index, &mut prev, code) {
            code = next_code;
            if code == CHAR_DOT {
              is_brace = true;
              token.is_brace = Some(true);
              is_glob = true;
              token.is_glob = true;
              finished = true;

              if scan_to_end {
                continue;
              }

              break;
            }
          } else {
            break;
          }
        }

        if !brace_escaped && code == CHAR_COMMA {
          is_brace = true;
          token.is_brace = Some(true);
          is_glob = true;
          token.is_glob = true;
          finished = true;

          if scan_to_end {
            continue;
          }

          break;
        }

        if code == CHAR_RIGHT_CURLY_BRACE {
          braces = braces.saturating_sub(1);

          if braces == 0 {
            brace_escaped = false;
            is_brace = true;
            token.is_brace = Some(true);
            finished = true;
            break;
          }
        }
      }

      if scan_to_end {
        continue;
      }

      break;
    }

    if code == CHAR_FORWARD_SLASH {
      slashes.push(index as usize);
      tokens.push(token);
      token = ScanToken::default();

      if finished {
        continue;
      }

      if prev == CHAR_DOT && index as usize == start + 1 {
        start += 2;
        continue;
      }

      last_index = index as usize + 1;
      continue;
    }

    if !options.noext && is_extglob_char(code) && peek(index) == Some(CHAR_LEFT_PARENTHESES) {
      is_glob = true;
      token.is_glob = true;
      is_extglob = true;
      token.is_extglob = Some(true);
      finished = true;

      if code == CHAR_EXCLAMATION_MARK && index as usize == start {
        negated_extglob = true;
      }

      if scan_to_end {
        while !eos(index) {
          let Some(next) = next_char(&mut index, &mut prev, code) else {
            break;
          };
          code = next;

          if code == CHAR_BACKWARD_SLASH {
            backslashes = true;
            token.backslashes = Some(true);
            if let Some(escaped) = next_char(&mut index, &mut prev, code) {
              code = escaped;
            }
            continue;
          }

          if code == CHAR_RIGHT_PARENTHESES {
            is_glob = true;
            token.is_glob = true;
            finished = true;
            break;
          }
        }
        continue;
      }

      break;
    }

    if code == CHAR_ASTERISK {
      if prev == CHAR_ASTERISK {
        is_globstar = true;
        token.is_globstar = Some(true);
      }
      is_glob = true;
      token.is_glob = true;
      finished = true;

      if scan_to_end {
        continue;
      }
      break;
    }

    if code == CHAR_QUESTION_MARK {
      is_glob = true;
      token.is_glob = true;
      finished = true;

      if scan_to_end {
        continue;
      }
      break;
    }

    if code == CHAR_LEFT_SQUARE_BRACKET {
      while !eos(index) {
        let Some(next) = next_char(&mut index, &mut prev, code) else {
          break;
        };

        if next == CHAR_BACKWARD_SLASH {
          backslashes = true;
          token.backslashes = Some(true);
          let _ = next_char(&mut index, &mut prev, next);
          continue;
        }

        if next == CHAR_RIGHT_SQUARE_BRACKET {
          is_bracket = true;
          token.is_bracket = Some(true);
          is_glob = true;
          token.is_glob = true;
          finished = true;
          break;
        }
      }

      if scan_to_end {
        continue;
      }

      break;
    }

    if !options.nonegate && code == CHAR_EXCLAMATION_MARK && index as usize == start {
      negated = true;
      token.negated = Some(true);
      start += 1;
      continue;
    }

    if !options.noparen && code == CHAR_LEFT_PARENTHESES {
      is_glob = true;
      token.is_glob = true;

      if scan_to_end {
        while !eos(index) {
          let Some(next) = next_char(&mut index, &mut prev, code) else {
            break;
          };
          code = next;

          if code == CHAR_LEFT_PARENTHESES {
            backslashes = true;
            token.backslashes = Some(true);
            if let Some(escaped) = next_char(&mut index, &mut prev, code) {
              code = escaped;
            }
            continue;
          }

          if code == CHAR_RIGHT_PARENTHESES {
            finished = true;
            break;
          }
        }
        continue;
      }
      break;
    }

    if is_glob {
      finished = true;

      if scan_to_end {
        continue;
      }

      break;
    }
  }

  if options.noext {
    is_extglob = false;
    is_glob = false;
  }

  let mut str = input.to_string();
  let mut prefix = String::new();
  let mut base;
  let mut glob = String::new();

  if start > 0 {
    prefix = view.slice(0, start);
    str = view.slice_from(start);
    last_index = last_index.saturating_sub(start);
  }

  base = str.clone();

  let str_view = InputView::new(&str);
  if !base.is_empty() && is_glob && last_index > 0 {
    base = str_view.slice(0, last_index);
    glob = str_view.slice_from(last_index);
  } else if is_glob {
    base.clear();
    glob = str.clone();
  }

  if !base.is_empty()
    && base != "/"
    && base != str
    && base.chars().last().is_some_and(is_path_separator)
  {
    base.pop();
  }

  if options.unescape {
    if !glob.is_empty() {
      glob = remove_backslashes(&glob);
    }

    if !base.is_empty() && backslashes {
      base = remove_backslashes(&base);
    }
  }

  let mut state = ScanState {
    prefix,
    input: input.to_string(),
    start,
    base,
    glob,
    is_brace,
    is_bracket,
    is_glob,
    is_extglob,
    is_globstar,
    negated,
    negated_extglob,
    max_depth: None,
    tokens: None,
    slashes: None,
    parts: None,
  };

  if options.tokens {
    state.max_depth = Some(0.0);
    if !is_path_separator(code) {
      tokens.push(token);
    }
  }

  if options.parts || options.tokens {
    let mut prev_index: Option<usize> = None;

    for (idx, slash_index) in slashes.iter().copied().enumerate() {
      let n = match prev_index {
        Some(prev_index_value) if prev_index_value != 0 => prev_index_value + 1,
        _ => start,
      };
      let value = view.slice(n, slash_index);

      if options.tokens {
        if idx == 0 && start != 0 {
          tokens[idx].is_prefix = Some(true);
          tokens[idx].value = state.prefix.clone();
        } else {
          tokens[idx].value = value.clone();
        }
        depth(&mut tokens[idx]);
        if let Some(max_depth) = state.max_depth.as_mut() {
          *max_depth += tokens[idx].depth;
        }
      }

      if idx != 0 || !value.is_empty() {
        parts.push(value);
      }
      prev_index = Some(slash_index);
    }

    if let Some(prev_index_value) = prev_index {
      if prev_index_value != 0 && prev_index_value + 1 < view.len() {
        let value = view.slice(prev_index_value + 1, view.len());
        parts.push(value.clone());

        if options.tokens {
          let last = tokens.len() - 1;
          tokens[last].value = value;
          depth(&mut tokens[last]);
          if let Some(max_depth) = state.max_depth.as_mut() {
            *max_depth += tokens[last].depth;
          }
        }
      }
    }

    state.slashes = Some(slashes);
    state.parts = Some(parts);
  }

  if options.tokens {
    state.tokens = Some(tokens);
  }

  state
}
