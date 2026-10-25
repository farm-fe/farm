use fancy_regex::Regex;

use crate::{make_re, CompileOptions};

#[derive(Debug)]
pub enum MatchError {
  EmptyPattern,
  UnsupportedPattern(String),
  InvalidRegex(String),
}

pub struct Matcher {
  glob: String,
  options: CompileOptions,
  regex: Regex,
}

impl Matcher {
  pub fn is_match(&self, input: &str) -> Result<bool, MatchError> {
    if input.is_empty() {
      return Ok(false);
    }

    let output = normalize_input(input, &self.options);

    if input == self.glob || output == self.glob {
      return Ok(true);
    }

    let candidate = if self.options.match_base || self.options.basename {
      basename(input, self.options.windows)
    } else {
      output
    };

    self
      .regex
      .is_match(&candidate)
      .map_err(|err| MatchError::InvalidRegex(err.to_string()))
  }
}

pub fn compile_matcher(pattern: &str, options: &CompileOptions) -> Result<Matcher, MatchError> {
  if pattern.is_empty() {
    return Err(MatchError::EmptyPattern);
  }
  if pattern.encode_utf16().count() > options.max_length.unwrap_or(65536).min(65536) {
    return Err(MatchError::UnsupportedPattern(
      "pattern length exceeds limit".to_string(),
    ));
  }

  let descriptor = make_re(pattern, options, false)
    .ok_or_else(|| MatchError::UnsupportedPattern(pattern.to_string()))?;
  let regex = Regex::new(&regex_source(&descriptor.source, &descriptor.flags))
    .map_err(|err| MatchError::InvalidRegex(err.to_string()))?;

  Ok(Matcher {
    glob: pattern.to_string(),
    options: options.clone(),
    regex,
  })
}

fn regex_source(source: &str, flags: &str) -> String {
  if flags.is_empty() {
    return source.to_string();
  }

  let mut inline = String::new();
  if flags.contains('i') {
    inline.push('i');
  }

  if inline.is_empty() {
    source.to_string()
  } else {
    format!("(?{inline}){source}")
  }
}

pub fn is_match(input: &str, pattern: &str, options: &CompileOptions) -> Result<bool, MatchError> {
  compile_matcher(pattern, options)?.is_match(input)
}

pub fn is_match_any<'a, I>(
  input: &str,
  patterns: I,
  options: &CompileOptions,
) -> Result<bool, MatchError>
where
  I: IntoIterator<Item = &'a str>,
{
  for pattern in patterns {
    if is_match(input, pattern, options)? {
      return Ok(true);
    }
  }

  Ok(false)
}

fn normalize_input(input: &str, options: &CompileOptions) -> String {
  let _ = options;
  input.to_string()
}

fn basename(input: &str, windows: bool) -> String {
  let parts: Vec<&str> = if windows {
    input.split(['/', '\\']).collect()
  } else {
    input.split('/').collect()
  };

  match parts.last().copied() {
    Some("") => parts
      .get(parts.len().saturating_sub(2))
      .copied()
      .unwrap_or_default()
      .to_string(),
    Some(value) => value.to_string(),
    None => String::new(),
  }
}

#[cfg(test)]
mod tests {
  use super::{basename, is_match, is_match_any};
  use crate::CompileOptions;

  #[test]
  fn matches_windows_literals() {
    let options = CompileOptions {
      windows: true,
      ..CompileOptions::default()
    };

    assert!(is_match("aaa\\bbb", "aaa/bbb", &options).unwrap());
    assert!(is_match("aaa/bbb", "aaa/bbb", &options).unwrap());
  }

  #[test]
  fn matches_against_any_pattern() {
    assert!(is_match_any("ab", ["*b", "foo"], &CompileOptions::default()).unwrap());
    assert!(!is_match_any("ab", ["foo", "bar"], &CompileOptions::default()).unwrap());
  }

  #[test]
  fn extracts_basename() {
    assert_eq!(basename("a/b/c.md", false), "c.md");
    assert_eq!(basename("a\\b\\c.md", true), "c.md");
    assert_eq!(basename("a/b/", false), "b");
  }

  // Covers the "should basename paths" test from test/regex-features.js
  // (tests/regex_features.rs references this via comment since basename is private)
  #[test]
  fn should_basename_paths() {
    assert_eq!(basename("/a/b/c", false), "c");
    assert_eq!(basename("/a/b/c/", false), "c");
    assert_eq!(basename("/a\\b/c", true), "c");
    assert_eq!(basename("/a\\b/c\\", true), "c");
    assert_eq!(basename("\\a/b\\c", true), "c");
    assert_eq!(basename("\\a/b\\c/", true), "c");
  }

  #[test]
  fn honors_case_insensitive_flag() {
    let options = CompileOptions {
      flags: "i".to_string(),
      ..CompileOptions::default()
    };

    assert!(is_match("A/B/C.MD", "a/b/*.md", &options).unwrap());
  }
}
