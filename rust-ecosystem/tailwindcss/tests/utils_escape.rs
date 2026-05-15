//! Ported from upstream `packages/tailwindcss/src/utils/escape.test.ts`.

use farmfe_ecosystem_tailwindcss::utils::escape::{escape, unescape};

#[test]
fn escape_adds_backslashes() {
  // Input: `red-1/2` → `red-1\/2`
  assert_eq!(escape("red-1/2"), r"red-1\/2");
}

#[test]
fn unescape_removes_backslashes() {
  assert_eq!(unescape(r"red-1\/2"), "red-1/2");
}

#[test]
fn unescape_replaces_out_of_range_escaped_code_points() {
  let input =
    r"--Coding-Projects-CharacterMapper-Master-Workspace\d8819554-4725-4235-9d22-2d0ed572e924";
  // \d88195 is in surrogate range when parsed as 6-hex-digit code-point;
  // upstream behavior: replace with U+FFFD, keep "54-..." remainder.
  assert_eq!(
    unescape(input),
    "--Coding-Projects-CharacterMapper-Master-Workspace\u{FFFD}54-4725-4235-9d22-2d0ed572e924"
  );
}
