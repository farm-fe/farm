/// Parsed candidate from a raw string like "hover:bg-red-500/50".
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedCandidate {
  /// The utility root, e.g. "flex", "bg", "text"
  pub utility_root: String,
  /// The utility value, e.g. "red-500" for "bg-red-500"
  pub utility_value: Option<String>,
  /// Whether this is an arbitrary property like "[color:red]"
  pub arbitrary_property: Option<(String, String)>,
  /// Whether this is an arbitrary value like "bg-[#0088cc]"
  pub arbitrary_value: Option<String>,
  /// Optional type hint for arbitrary values, e.g. "color" in "bg-[color:var(--x)]"
  pub type_hint: Option<String>,
  /// Variant stack, e.g. ["hover", "focus"] for "focus:hover:bg-red-500"
  pub variants: Vec<String>,
  /// Modifier, e.g. "50" for "bg-red-500/50"
  pub modifier: Option<String>,
  /// Whether the modifier is arbitrary (e.g., /[50%])
  pub modifier_is_arbitrary: bool,
  /// Whether the utility is important (!)
  pub important: bool,
  /// Whether the utility is negated (leading `-` after variants, e.g. `-mt-4`)
  pub negative: bool,
  /// Whether this is a static utility (no dash-separated value)
  pub is_static: bool,
  /// The raw input string
  pub raw: String,
}

/// Parse a raw candidate string into structured components.
/// Returns `None` if the candidate is clearly invalid.
pub fn parse_candidate(input: &str) -> Option<ParsedCandidate> {
  if input.is_empty() {
    return None;
  }

  let raw = input.to_string();

  // Handle arbitrary variants like [&_p]:flex — find the first ':' that is
  // not inside brackets.
  let segments = split_at_colon(input);

  if segments.is_empty() {
    return None;
  }

  let base = segments.last().unwrap();
  let variants: Vec<String> = segments[..segments.len() - 1]
    .iter()
    .map(|s| s.to_string())
    .collect();

  // Sanity limit
  if variants.len() > 10 {
    return None;
  }

  // Check for important
  let (base, important) = if base.ends_with('!') {
    (&base[..base.len() - 1], true)
  } else if base.starts_with('!') {
    (&base[1..], true)
  } else {
    (base as &str, false)
  };

  // Check for leading `-` (negative utility). Only meaningful when the
  // remainder is a plain (non-arbitrary, non-bracketed) utility.
  let (base, negative) = if let Some(stripped) = base.strip_prefix('-') {
    // Don't consume `-` for arbitrary properties / arbitrary values starting
    // with `[`, and never produce an empty base.
    if stripped.is_empty() || stripped.starts_with('[') {
      (base, false)
    } else {
      (stripped, true)
    }
  } else {
    (base, false)
  };

  // Split base into utility + modifier by '/' (outside brackets)
  let (base_no_modifier, modifier) = split_modifier(base);

  let modifier_is_arbitrary = modifier
    .as_ref()
    .map_or(false, |m| m.starts_with('[') && m.ends_with(']'));

  // Arbitrary property: [property:value]
  if base_no_modifier.starts_with('[') && base_no_modifier.ends_with(']') {
    let inner = &base_no_modifier[1..base_no_modifier.len() - 1];
    // Find colon outside nested brackets/parens
    if let Some(colon_idx) = find_colon_outside_brackets(inner) {
      if colon_idx == 0 || colon_idx == inner.len() - 1 {
        return None;
      }
      let property = inner[..colon_idx].to_string();
      let value = normalize_arbitrary_value(&inner[colon_idx + 1..]);

      let first = property.as_bytes()[0];
      if !first.is_ascii_lowercase() && first != b'-' {
        return None;
      }

      return Some(ParsedCandidate {
        utility_root: String::new(),
        utility_value: None,
        arbitrary_property: Some((property, value)),
        arbitrary_value: None,
        type_hint: None,
        variants,
        modifier,
        modifier_is_arbitrary,
        important,
        negative,
        is_static: false,
        raw,
      });
    }
    // No colon — treat as invalid arbitrary property
    return None;
  }

  // Arbitrary value with parens (CSS-variable shorthand): something-(--my-var)
  // The inner value MUST start with `--`; the result is `var(<inner>)`.
  if base_no_modifier.ends_with(')') {
    if let Some(paren_start) = find_opening_paren(&base_no_modifier) {
      let root = base_no_modifier[..paren_start].trim_end_matches('-').to_string();
      let raw_inner = &base_no_modifier[paren_start + 1..base_no_modifier.len() - 1];

      // Optional type hint inside the parens, e.g. `bg-(color:--my-color)`
      let (type_hint, inner) = extract_type_hint(raw_inner);
      // Decode escaped underscores (`\_` -> `_`) but DO NOT convert plain `_`
      // to spaces inside the paren-shorthand: this form represents a CSS
      // variable reference where underscores are meaningful (e.g. `--_foo`).
      let inner = decode_escaped_underscores(&inner);

      // Must be non-empty and start with `--` to be a valid var shorthand.
      if inner.trim().is_empty() || !inner.starts_with("--") {
        return None;
      }

      return Some(ParsedCandidate {
        utility_root: root,
        utility_value: None,
        arbitrary_property: None,
        arbitrary_value: Some(format!("var({inner})")),
        type_hint,
        variants,
        modifier,
        modifier_is_arbitrary,
        important,
        negative,
        is_static: false,
        raw,
      });
    }
  }

  // Arbitrary value: something-[value]
  if base_no_modifier.ends_with(']') {
    if let Some(bracket_start) = find_opening_bracket(&base_no_modifier) {
      let root = base_no_modifier[..bracket_start].trim_end_matches('-').to_string();
      let raw_value = &base_no_modifier[bracket_start + 1..base_no_modifier.len() - 1];

      // Split type hint if present: "color:var(--x)" -> hint="color", value="var(--x)"
      let (type_hint, value) = extract_type_hint(raw_value);
      let value = normalize_arbitrary_value(&value);

      if value.trim().is_empty() {
        return None;
      }

      return Some(ParsedCandidate {
        utility_root: root,
        utility_value: None,
        arbitrary_property: None,
        arbitrary_value: Some(value),
        type_hint,
        variants,
        modifier,
        modifier_is_arbitrary,
        important,
        negative,
        is_static: false,
        raw,
      });
    }
  }

  // Named utility: "flex", "bg-red-500", etc.
  if base_no_modifier
    .chars()
    .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
  {
    let (root, value) = split_root(&base_no_modifier);
    let is_static = value.is_none();

    return Some(ParsedCandidate {
      utility_root: root,
      utility_value: value,
      arbitrary_property: None,
      arbitrary_value: None,
      type_hint: None,
      variants,
      modifier,
      modifier_is_arbitrary,
      important,
      negative,
      is_static,
      raw,
    });
  }

  None
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Split "hover:focus:flex" at ':' that are outside brackets.
fn split_at_colon(input: &str) -> Vec<String> {
  let mut segments = Vec::new();
  let mut current = String::new();
  let mut depth: i32 = 0;

  for ch in input.chars() {
    match ch {
      '[' | '(' => {
        depth += 1;
        current.push(ch);
      }
      ']' | ')' => {
        depth -= 1;
        current.push(ch);
      }
      ':' if depth == 0 => {
        segments.push(current.clone());
        current.clear();
      }
      _ => {
        current.push(ch);
      }
    }
  }
  segments.push(current);
  segments
}

/// Split "bg-red-500/50" into ("bg-red-500", Some("50")).
fn split_modifier(input: &str) -> (String, Option<String>) {
  let mut depth: i32 = 0;
  let mut slash_pos: Option<usize> = None;

  for (i, ch) in input.char_indices() {
    match ch {
      '[' => depth += 1,
      ']' => depth -= 1,
      '/' if depth == 0 => {
        slash_pos = Some(i);
        break;
      }
      _ => {}
    }
  }

  match slash_pos {
    Some(pos) => (input[..pos].to_string(), Some(input[pos + 1..].to_string())),
    None => (input.to_string(), None),
  }
}

/// Find a ':' outside brackets/parens in an arbitrary property value.
fn find_colon_outside_brackets(input: &str) -> Option<usize> {
  let mut depth: i32 = 0;
  for (i, ch) in input.char_indices() {
    match ch {
      '[' | '(' => depth += 1,
      ']' | ')' => depth -= 1,
      ':' if depth == 0 => return Some(i),
      _ => {}
    }
  }
  None
}

/// Find the `[` that starts an arbitrary value suffix like `-[value]`.
fn find_opening_bracket(input: &str) -> Option<usize> {
  // Walk backwards to find the '[' that closes with ']' at end
  let bytes = input.as_bytes();
  let mut depth = 0i32;
  for i in (0..bytes.len()).rev() {
    if bytes[i] == b']' {
      depth += 1;
    } else if bytes[i] == b'[' {
      depth -= 1;
      if depth == 0 {
        return Some(i);
      }
    }
  }
  None
}

/// Walk backwards to find the `(` matching the trailing `)`.
fn find_opening_paren(input: &str) -> Option<usize> {
  let bytes = input.as_bytes();
  let mut depth = 0i32;
  for i in (0..bytes.len()).rev() {
    if bytes[i] == b')' {
      depth += 1;
    } else if bytes[i] == b'(' {
      depth -= 1;
      if depth == 0 {
        return Some(i);
      }
    }
  }
  None
}

/// Decode `\_` escapes to literal underscores. Unlike
/// [`normalize_arbitrary_value`], lone underscores are preserved as-is — used
/// for the paren-arbitrary CSS-variable shorthand where `_` is part of the
/// identifier (e.g. `--_foo`).
fn decode_escaped_underscores(input: &str) -> String {
  let bytes = input.as_bytes();
  let mut out = String::with_capacity(bytes.len());
  let mut i = 0;
  while i < bytes.len() {
    if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'_' {
      out.push('_');
      i += 2;
    } else {
      let n = utf8_char_len(bytes[i]);
      out.push_str(&input[i..i + n]);
      i += n;
    }
  }
  out
}

/// Extract optional type hint from an arbitrary value like "color:var(--x)".
fn extract_type_hint(raw: &str) -> (Option<String>, String) {
  if let Some(colon) = raw.find(':') {
    if colon > 0 {
      let hint_candidate = &raw[..colon];
      if hint_candidate
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b == b'-')
      {
        return (
          Some(hint_candidate.to_string()),
          raw[colon + 1..].to_string(),
        );
      }
    }
  }
  (None, raw.to_string())
}

/// Split a named utility into root and optional value.
/// "flex" -> ("flex", None), "bg-red-500" -> ("bg", Some("red-500"))
fn split_root(input: &str) -> (String, Option<String>) {
  if let Some(idx) = input.find('-') {
    (input[..idx].to_string(), Some(input[idx + 1..].to_string()))
  } else {
    (input.to_string(), None)
  }
}

/// Normalize an arbitrary value the way Tailwind v4 does: convert lone
/// underscores to spaces, leave `\_` as a literal `_`, and never substitute
/// underscores that appear inside `url(...)` function calls (which are
/// commonly used for unquoted URLs containing underscores).
fn normalize_arbitrary_value(input: &str) -> String {
  let bytes = input.as_bytes();
  let mut out = String::with_capacity(bytes.len());
  let mut i = 0;
  while i < bytes.len() {
    let b = bytes[i];
    // Escaped underscore: emit literal '_'
    if b == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'_' {
      out.push('_');
      i += 2;
      continue;
    }
    // Preserve everything inside url(...) verbatim (handles nested parens).
    if (b == b'u' || b == b'U')
      && bytes[i..].len() >= 4
      && bytes[i + 1].eq_ignore_ascii_case(&b'r')
      && bytes[i + 2].eq_ignore_ascii_case(&b'l')
      && bytes[i + 3] == b'('
    {
      let start = i;
      i += 4;
      let mut depth = 1i32;
      while i < bytes.len() && depth > 0 {
        match bytes[i] {
          b'(' => depth += 1,
          b')' => depth -= 1,
          _ => {}
        }
        i += 1;
      }
      out.push_str(&input[start..i]);
      continue;
    }
    if b == b'_' {
      out.push(' ');
    } else {
      // Push a single UTF-8 character starting at i.
      let ch_len = utf8_char_len(b);
      out.push_str(&input[i..i + ch_len]);
      i += ch_len;
      continue;
    }
    i += 1;
  }
  out
}

#[inline]
fn utf8_char_len(first_byte: u8) -> usize {
  match first_byte {
    0x00..=0x7F => 1,
    0xC0..=0xDF => 2,
    0xE0..=0xEF => 3,
    0xF0..=0xF7 => 4,
    _ => 1,
  }
}
