//! CSS identifier escape/unescape, ported from
//! `packages/tailwindcss/src/utils/escape.ts`.
//!
//! Implements <https://drafts.csswg.org/cssom/#serialize-an-identifier>
//! and the inverse unescape used by upstream tailwindcss.

/// Serialize `value` as a CSS identifier per CSSOM rules.
pub fn escape(value: &str) -> String {
  // Operate on UTF-16 code units (like JS `charCodeAt`) so behavior
  // matches upstream byte-for-byte for BMP characters and uses surrogate
  // pairs for astral code points.
  let units: Vec<u16> = value.encode_utf16().collect();
  let length = units.len();
  let mut result = String::new();

  if length == 0 {
    return result;
  }

  let first = units[0];

  // If the only character is `-`, prefix with backslash.
  if length == 1 && first == 0x002d {
    return format!("\\{}", value);
  }

  let mut index = 0usize;
  while index < length {
    let code_unit = units[index];

    // NULL → REPLACEMENT CHARACTER.
    if code_unit == 0x0000 {
      result.push('\u{FFFD}');
      index += 1;
      continue;
    }

    // Control chars [\1-\1F] or DEL, leading digit, or second-char
    // digit when first is `-`.
    if (0x0001..=0x001f).contains(&code_unit)
      || code_unit == 0x007f
      || (index == 0 && (0x0030..=0x0039).contains(&code_unit))
      || (index == 1 && (0x0030..=0x0039).contains(&code_unit) && first == 0x002d)
    {
      result.push_str(&format!("\\{:x} ", code_unit));
      index += 1;
      continue;
    }

    // The character itself for safe ranges.
    if code_unit >= 0x0080
      || code_unit == 0x002d
      || code_unit == 0x005f
      || (0x0030..=0x0039).contains(&code_unit)
      || (0x0041..=0x005a).contains(&code_unit)
      || (0x0061..=0x007a).contains(&code_unit)
    {
      let consumed = push_code_unit(&mut result, &units, index);
      index += consumed;
      continue;
    }

    // Otherwise, escape the literal character with a backslash.
    result.push('\\');
    let consumed = push_code_unit(&mut result, &units, index);
    index += consumed;
  }

  result
}

/// Push the code unit at `index` (and its low-surrogate partner if it's
/// a high surrogate followed by a low one). Returns how many code units
/// were consumed.
fn push_code_unit(out: &mut String, units: &[u16], index: usize) -> usize {
  let unit = units[index];
  if (0xd800..=0xdbff).contains(&unit) && index + 1 < units.len() {
    let next = units[index + 1];
    if (0xdc00..=0xdfff).contains(&next) {
      let code = 0x10000 + (((unit as u32 - 0xd800) << 10) | (next as u32 - 0xdc00));
      if let Some(c) = char::from_u32(code) {
        out.push(c);
        return 2;
      }
    }
  }
  if let Some(c) = char::from_u32(unit as u32) {
    out.push(c);
  } else {
    out.push('\u{FFFD}');
  }
  1
}

/// Inverse of [`escape`], matching upstream `unescape`.
pub fn unescape(escaped: &str) -> String {
  let bytes = escaped.as_bytes();
  let len = bytes.len();
  let mut out = String::with_capacity(len);
  let mut i = 0usize;

  while i < len {
    let ch = bytes[i];
    if ch != b'\\' {
      let next = next_char_boundary(escaped, i);
      out.push_str(&escaped[i..next]);
      i = next;
      continue;
    }

    if i + 1 >= len {
      out.push('\\');
      i += 1;
      continue;
    }

    // Try hex escape: 1..=6 hex digits, optionally followed by one ws.
    let j = i + 1;
    let mut hex_end = j;
    while hex_end < len && hex_end - j < 6 && (bytes[hex_end] as char).is_ascii_hexdigit() {
      hex_end += 1;
    }

    if hex_end > j {
      // Optional trailing whitespace per spec ([\t\n\f\r ]).
      let mut consumed_end = hex_end;
      if consumed_end < len {
        let b = bytes[consumed_end];
        if matches!(b, 0x09 | 0x0a | 0x0c | 0x0d | 0x20) {
          consumed_end += 1;
        }
      }
      let hex_str = &escaped[j..hex_end];
      let code_point = u32::from_str_radix(hex_str, 16).unwrap_or(0);
      if code_point == 0 || code_point > 0x10ffff || (0xd800..=0xdfff).contains(&code_point) {
        out.push('\u{FFFD}');
      } else if let Some(c) = char::from_u32(code_point) {
        out.push(c);
      } else {
        out.push('\u{FFFD}');
      }
      i = consumed_end;
      continue;
    }

    // Non-hex escape: copy the next character verbatim.
    let next = next_char_boundary(escaped, i + 1);
    out.push_str(&escaped[i + 1..next]);
    i = next;
  }

  out
}

fn next_char_boundary(s: &str, byte_index: usize) -> usize {
  let mut i = byte_index + 1;
  while i < s.len() && !s.is_char_boundary(i) {
    i += 1;
  }
  i
}
