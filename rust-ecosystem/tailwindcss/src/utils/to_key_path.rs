//! Path string → key array, ported from
//! `packages/tailwindcss/src/utils/to-key-path.ts`.

use super::segment::segment;

/// Parse a path string into key segments.
///
/// Square bracket notation `a[b]` may be used to "escape" dots that
/// would otherwise be interpreted as path separators.
///
/// Examples:
/// - `a` → `["a"]`
/// - `a.b.c` → `["a", "b", "c"]`
/// - `a[b].c` → `["a", "b", "c"]`
/// - `a[b.c].e.f` → `["a", "b.c", "e", "f"]`
/// - `a[b][c][d]` → `["a", "b", "c", "d"]`
pub fn to_key_path(path: &str) -> Vec<String> {
  let mut keypath: Vec<String> = Vec::new();

  for part in segment(path, '.') {
    if !part.contains('[') {
      keypath.push(part);
      continue;
    }

    let bytes = part.as_bytes();
    let mut current = 0usize;

    loop {
      let Some(bracket_l) = find_byte_from(bytes, b'[', current) else {
        break;
      };
      let Some(bracket_r) = find_byte_from(bytes, b']', bracket_l) else {
        break;
      };

      if bracket_l > current {
        keypath.push(part[current..bracket_l].to_string());
      }
      keypath.push(part[bracket_l + 1..bracket_r].to_string());
      current = bracket_r + 1;
    }

    // Tail after the last bracket.
    if current < part.len() {
      keypath.push(part[current..].to_string());
    }
  }

  keypath
}

fn find_byte_from(haystack: &[u8], needle: u8, start: usize) -> Option<usize> {
  if start >= haystack.len() {
    return None;
  }
  haystack[start..]
    .iter()
    .position(|&b| b == needle)
    .map(|p| p + start)
}
