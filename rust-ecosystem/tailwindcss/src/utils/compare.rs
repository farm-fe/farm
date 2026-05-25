//! Alphanumeric comparison, ported from
//! `packages/tailwindcss/src/utils/compare.ts`.

use std::cmp::Ordering;

const ZERO: u8 = b'0';
const NINE: u8 = b'9';

/// Compare two strings alphanumerically, treating embedded digit runs as
/// numbers rather than character sequences.
///
/// Returns `Ordering::Less` / `Equal` / `Greater`. Suitable for use with
/// `slice::sort_by`.
pub fn compare(a: &str, z: &str) -> Ordering {
  let ab = a.as_bytes();
  let zb = z.as_bytes();
  let a_len = ab.len();
  let z_len = zb.len();
  let min_len = a_len.min(z_len);
  let mut i = 0usize;

  while i < min_len {
    let a_code = ab[i];
    let z_code = zb[i];

    if (ZERO..=NINE).contains(&a_code) && (ZERO..=NINE).contains(&z_code) {
      let a_start = i;
      let mut a_end = i + 1;
      let z_start = i;
      let mut z_end = i + 1;

      while a_end < a_len && (ZERO..=NINE).contains(&ab[a_end]) {
        a_end += 1;
      }
      while z_end < z_len && (ZERO..=NINE).contains(&zb[z_end]) {
        z_end += 1;
      }

      // Numeric comparison, then string fallback when numerically equal.
      let a_num: u128 = std::str::from_utf8(&ab[a_start..a_end])
        .unwrap()
        .parse()
        .unwrap_or(0);
      let z_num: u128 = std::str::from_utf8(&zb[z_start..z_end])
        .unwrap()
        .parse()
        .unwrap_or(0);

      match a_num.cmp(&z_num) {
        Ordering::Equal => {
          // Same numeric value, e.g. "0123" vs "123": fallback to string.
          let a_slice = &ab[a_start..a_end];
          let z_slice = &zb[z_start..z_end];
          match a_slice.cmp(z_slice) {
            Ordering::Equal => {
              i = a_end;
              continue;
            }
            other => return other,
          }
        }
        other => return other,
      }
    }

    if a_code == z_code {
      i += 1;
      continue;
    }

    return a_code.cmp(&z_code);
  }

  a_len.cmp(&z_len)
}
