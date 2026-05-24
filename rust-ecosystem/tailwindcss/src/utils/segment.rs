//! Top-level character splitting, ported from
//! `packages/tailwindcss/src/utils/segment.ts`.
//!
//! Splits a string on a single ASCII separator character, skipping over
//! balanced `()`, `[]`, `{}`, double-quoted, and single-quoted regions,
//! and honoring `\` escapes.

const BACKSLASH: u8 = 0x5c;
const OPEN_CURLY: u8 = 0x7b;
const CLOSE_CURLY: u8 = 0x7d;
const OPEN_PAREN: u8 = 0x28;
const CLOSE_PAREN: u8 = 0x29;
const OPEN_BRACKET: u8 = 0x5b;
const CLOSE_BRACKET: u8 = 0x5d;
const DOUBLE_QUOTE: u8 = 0x22;
const SINGLE_QUOTE: u8 = 0x27;

/// Split `input` on every top-level occurrence of `separator`.
///
/// `separator` must be a single ASCII character; only its first byte is
/// considered. This mirrors the upstream JS implementation, which uses
/// `separator.charCodeAt(0)`.
pub fn segment(input: &str, separator: char) -> Vec<String> {
  let bytes = input.as_bytes();
  let len = bytes.len();
  let separator_code = separator as u32;
  let separator_is_ascii = separator_code <= 0x7f;
  let separator_byte = if separator_is_ascii {
    separator_code as u8
  } else {
    0u8
  };

  let mut stack: Vec<u8> = Vec::with_capacity(16);
  let mut parts: Vec<String> = Vec::new();
  let mut last_pos = 0usize;
  let mut idx = 0usize;

  while idx < len {
    let ch = bytes[idx];

    if separator_is_ascii && stack.is_empty() && ch == separator_byte {
      parts.push(input[last_pos..idx].to_string());
      last_pos = idx + 1;
      idx += 1;
      continue;
    }

    match ch {
      BACKSLASH => {
        // Skip the next byte (escaped).
        idx += 2;
        continue;
      }
      SINGLE_QUOTE | DOUBLE_QUOTE => {
        let quote = ch;
        idx += 1;
        while idx < len {
          let next = bytes[idx];
          if next == BACKSLASH {
            idx += 2;
            continue;
          }
          if next == quote {
            break;
          }
          idx += 1;
        }
      }
      OPEN_PAREN => stack.push(CLOSE_PAREN),
      OPEN_BRACKET => stack.push(CLOSE_BRACKET),
      OPEN_CURLY => stack.push(CLOSE_CURLY),
      CLOSE_PAREN | CLOSE_BRACKET | CLOSE_CURLY => {
        if let Some(&top) = stack.last() {
          if top == ch {
            stack.pop();
          }
        }
      }
      _ => {}
    }

    idx += 1;
  }

  let tail_start = last_pos.min(len);
  parts.push(input[tail_start..].to_string());
  parts
}
