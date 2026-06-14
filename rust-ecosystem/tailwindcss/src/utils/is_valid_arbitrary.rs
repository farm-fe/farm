//! Arbitrary-value validation, ported from
//! `packages/tailwindcss/src/utils/is-valid-arbitrary.ts`.

const BACKSLASH: u8 = 0x5c;
const OPEN_CURLY: u8 = 0x7b;
const CLOSE_CURLY: u8 = 0x7d;
const OPEN_PAREN: u8 = 0x28;
const CLOSE_PAREN: u8 = 0x29;
const OPEN_BRACKET: u8 = 0x5b;
const CLOSE_BRACKET: u8 = 0x5d;
const DOUBLE_QUOTE: u8 = 0x22;
const SINGLE_QUOTE: u8 = 0x27;
const SEMICOLON: u8 = 0x3b;

/// Determine if a string could be a valid arbitrary value.
///
/// Unbalanced parens, brackets, and braces are not allowed. Additionally a
/// top-level `;` is not allowed. Mirrors the upstream JS implementation.
pub fn is_valid_arbitrary(input: &str) -> bool {
  let bytes = input.as_bytes();
  let len = bytes.len();
  let mut stack: Vec<u8> = Vec::with_capacity(8);

  let mut i = 0usize;
  while i < len {
    let c = bytes[i];
    match c {
      BACKSLASH => {
        // Skip escaped character.
        i += 2;
        continue;
      }
      SINGLE_QUOTE | DOUBLE_QUOTE => {
        // Consume until matching quote.
        let quote = c;
        i += 1;
        while i < len {
          let next = bytes[i];
          if next == BACKSLASH {
            i += 2;
            continue;
          }
          if next == quote {
            break;
          }
          i += 1;
        }
      }
      OPEN_PAREN => stack.push(CLOSE_PAREN),
      OPEN_BRACKET => stack.push(CLOSE_BRACKET),
      OPEN_CURLY => {
        // Intentionally not pushed: candidates like `[&{color:red}]:flex`
        // must be rejected.
      }
      CLOSE_BRACKET | CLOSE_CURLY | CLOSE_PAREN => {
        if stack.is_empty() {
          return false;
        }
        if *stack.last().unwrap() == c {
          stack.pop();
        }
      }
      SEMICOLON if stack.is_empty() => {
        return false;
      }
      _ => {}
    }
    i += 1;
  }

  true
}
