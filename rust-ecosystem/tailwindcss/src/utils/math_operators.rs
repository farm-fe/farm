//! Math-operator whitespace formatter, ported from
//! `packages/tailwindcss/src/utils/math-operators.ts`.

const LOWER_A: u8 = 0x61;
const LOWER_Z: u8 = 0x7a;
const UPPER_A: u8 = 0x41;
const UPPER_Z: u8 = 0x5a;
const LOWER_E: u8 = 0x65;
const UPPER_E: u8 = 0x45;
const ZERO: u8 = 0x30;
const NINE: u8 = 0x39;
const ADD: u8 = 0x2b;
const SUB: u8 = 0x2d;
const MUL: u8 = 0x2a;
const DIV: u8 = 0x2f;
const OPEN_PAREN: u8 = 0x28;
const CLOSE_PAREN: u8 = 0x29;
const COMMA: u8 = 0x2c;
const SPACE: u8 = 0x20;
const PERCENT: u8 = 0x25;

pub const MATH_FUNCTIONS: &[&str] = &[
  "calc", "min", "max", "clamp", "mod", "rem", "sin", "cos", "tan", "asin", "acos", "atan",
  "atan2", "pow", "sqrt", "hypot", "log", "exp", "round",
];

/// Returns true if the input contains a known CSS math function call.
pub fn has_math_fn(input: &str) -> bool {
  if !input.contains('(') {
    return false;
  }
  MATH_FUNCTIONS
    .iter()
    .any(|fn_name| input.contains(&format!("{fn_name}(")))
}

/// Add whitespace around math operators inside math function calls.
///
/// Outside of math functions, the input is returned unchanged. Inside,
/// `+ - * /` get surrounded by spaces according to the upstream rules.
pub fn add_whitespace_around_math_operators(input: &str) -> String {
  // Bail early if no math function name appears anywhere in the input.
  if !MATH_FUNCTIONS.iter().any(|fn_name| input.contains(fn_name)) {
    return input.to_string();
  }

  let bytes = input.as_bytes();
  // Pre-allocate. Result may be slightly longer due to inserted spaces.
  let mut result: Vec<u8> = Vec::with_capacity(bytes.len() + 16);
  let mut formattable: Vec<bool> = Vec::with_capacity(8);

  let mut value_pos: Option<usize> = None;
  let mut last_value_pos: Option<usize> = None;

  let is_digit = |b: u8| (ZERO..=NINE).contains(&b);
  let is_lower = |b: u8| (LOWER_A..=LOWER_Z).contains(&b);
  let is_upper = |b: u8| (UPPER_A..=UPPER_Z).contains(&b);

  let mut i = 0usize;
  while i < bytes.len() {
    let c = bytes[i];

    // Track value positions (digit, then unit chars).
    if is_digit(c) || (value_pos.is_some() && (c == PERCENT || is_lower(c) || is_upper(c))) {
      value_pos = Some(i);
    } else {
      last_value_pos = value_pos;
      value_pos = None;
    }

    if c == OPEN_PAREN {
      result.push(c);

      // Scan backwards for fn name (lowercase alphanumerics).
      let mut start = i;
      let mut j = i;
      while j > 0 {
        j -= 1;
        let inner = bytes[j];
        if is_digit(inner) || is_lower(inner) {
          start = j;
        } else {
          break;
        }
      }
      let fn_name = std::str::from_utf8(&bytes[start..i]).unwrap_or("");

      if MATH_FUNCTIONS.contains(&fn_name) {
        formattable.insert(0, true);
      } else if formattable.first().copied().unwrap_or(false) && fn_name.is_empty() {
        // Nested parens inside a math function: keep formatting.
        formattable.insert(0, true);
      } else {
        formattable.insert(0, false);
      }
      i += 1;
      continue;
    } else if c == CLOSE_PAREN {
      result.push(c);
      if !formattable.is_empty() {
        formattable.remove(0);
      }
      i += 1;
      continue;
    } else if c == COMMA && formattable.first().copied().unwrap_or(false) {
      result.extend_from_slice(b", ");
      i += 1;
      continue;
    } else if c == SPACE
      && formattable.first().copied().unwrap_or(false)
      && result.last().copied() == Some(SPACE)
    {
      i += 1;
      continue;
    } else if (c == ADD || c == MUL || c == DIV || c == SUB)
      && formattable.first().copied().unwrap_or(false)
    {
      // Determine context.
      let trimmed_len = trim_end_len(&result);
      let prev = if trimmed_len > 0 {
        result[trimmed_len - 1]
      } else {
        0
      };
      let prev_prev = if trimmed_len > 1 {
        result[trimmed_len - 2]
      } else {
        0
      };
      let next = bytes.get(i + 1).copied().unwrap_or(0);

      // Scientific notation: `-3.4e-2`.
      if (prev == LOWER_E || prev == UPPER_E) && is_digit(prev_prev) {
        result.push(c);
        i += 1;
        continue;
      }

      // Preceded by an operator: no spaces.
      if prev == ADD || prev == MUL || prev == DIV || prev == SUB {
        result.push(c);
        i += 1;
        continue;
      }

      // Beginning of an argument: no spaces.
      if prev == OPEN_PAREN || prev == COMMA {
        result.push(c);
        i += 1;
        continue;
      }

      // Already had a space before: only add one after.
      if i > 0 && bytes[i - 1] == SPACE {
        result.push(c);
        result.push(SPACE);
        i += 1;
        continue;
      }

      // Surround with spaces if appropriate.
      let last_value_adjacent = last_value_pos.is_some_and(|p| i > 0 && p == i - 1);
      if is_digit(prev)
        || is_digit(next)
        || prev == CLOSE_PAREN
        || next == OPEN_PAREN
        || next == ADD
        || next == MUL
        || next == DIV
        || next == SUB
        || last_value_adjacent
      {
        result.push(SPACE);
        result.push(c);
        result.push(SPACE);
      } else {
        result.push(c);
      }
      i += 1;
      continue;
    } else {
      result.push(c);
      i += 1;
    }
  }

  String::from_utf8(result).expect("ASCII-safe transformation")
}

fn trim_end_len(buf: &[u8]) -> usize {
  let mut n = buf.len();
  while n > 0 && buf[n - 1].is_ascii_whitespace() {
    n -= 1;
  }
  n
}
