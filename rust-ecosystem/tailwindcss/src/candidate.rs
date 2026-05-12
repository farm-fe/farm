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
      let value = inner[colon_idx + 1..].to_string();

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
        is_static: false,
        raw,
      });
    }
    // No colon — treat as invalid arbitrary property
    return None;
  }

  // Arbitrary value: something-[value]
  if base_no_modifier.ends_with(']') {
    if let Some(bracket_start) = find_opening_bracket(&base_no_modifier) {
      let root = base_no_modifier[..bracket_start].trim_end_matches('-').to_string();
      let raw_value = &base_no_modifier[bracket_start + 1..base_no_modifier.len() - 1];

      // Split type hint if present: "color:var(--x)" -> hint="color", value="var(--x)"
      let (type_hint, value) = extract_type_hint(raw_value);

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
      '[' => {
        depth += 1;
        current.push(ch);
      }
      ']' => {
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
