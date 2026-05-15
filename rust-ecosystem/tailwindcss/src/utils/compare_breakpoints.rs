//! Breakpoint comparator, ported from
//! `packages/tailwindcss/src/utils/compare-breakpoints.ts`.

use std::cmp::Ordering;

/// Sort direction for [`compare_breakpoints`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
  Asc,
  Desc,
}

/// Compare two breakpoint values like `40rem`, `calc(100% - 1rem)`.
///
/// Bucketing rules mirror upstream:
/// - If neither side contains `(`, bucket by unit (digits/dots removed).
/// - Otherwise bucket by the function name (everything before `(`).
///
/// Within a bucket, numeric sort by the leading integer, with a string
/// fallback when both values lack parseable numbers (e.g. CSS function
/// expressions).
pub fn compare_breakpoints(a: &str, z: &str, direction: Direction) -> Ordering {
  if a == z {
    return Ordering::Equal;
  }

  let a_paren = a.find('(');
  let z_paren = z.find('(');

  let a_bucket = match a_paren {
    None => strip_digits_and_dots(a),
    Some(idx) => a[..idx].to_string(),
  };
  let z_bucket = match z_paren {
    None => strip_digits_and_dots(z),
    Some(idx) => z[..idx].to_string(),
  };

  match a_bucket.cmp(&z_bucket) {
    Ordering::Equal => {}
    other => return other,
  }

  let a_num = parse_leading_int(a);
  let z_num = parse_leading_int(z);

  match (a_num, z_num) {
    (Some(av), Some(zv)) => match direction {
      Direction::Asc => av.cmp(&zv),
      Direction::Desc => zv.cmp(&av),
    },
    // NaN fallback: alphabetical compare.
    _ => a.cmp(z),
  }
}

fn strip_digits_and_dots(s: &str) -> String {
  s.chars()
    .filter(|c| !c.is_ascii_digit() && *c != '.')
    .collect()
}

/// JS `parseInt` parity: leading optional sign + leading digits; ignores
/// trailing non-digit content. Returns `None` if no digits are found.
fn parse_leading_int(s: &str) -> Option<i64> {
  let bytes = s.as_bytes();
  let mut i = 0usize;
  // Optional leading whitespace, then optional sign.
  while i < bytes.len() && (bytes[i] as char).is_ascii_whitespace() {
    i += 1;
  }
  let neg = if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+') {
    let n = bytes[i] == b'-';
    i += 1;
    n
  } else {
    false
  };
  let start = i;
  while i < bytes.len() && bytes[i].is_ascii_digit() {
    i += 1;
  }
  if i == start {
    return None;
  }
  let mag: i64 = std::str::from_utf8(&bytes[start..i]).ok()?.parse().ok()?;
  Some(if neg { -mag } else { mag })
}
