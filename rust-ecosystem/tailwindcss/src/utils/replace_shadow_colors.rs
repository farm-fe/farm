//! Shadow-color replacement, ported from
//! `packages/tailwindcss/src/utils/replace-shadow-colors.ts`.

use super::segment::segment;
use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

fn keywords() -> &'static HashSet<&'static str> {
  static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
  SET.get_or_init(|| {
    ["inset", "inherit", "initial", "revert", "unset"]
      .into_iter()
      .collect()
  })
}

fn length_re() -> &'static Regex {
  static RE: OnceLock<Regex> = OnceLock::new();
  RE.get_or_init(|| Regex::new(r"^-?(\d+|\.\d+)(.*?)$").unwrap())
}

/// Substitute shadow colours in a `box-shadow`-style value.
///
/// `replacement` is applied to whatever color is detected (or
/// `"currentcolor"` if no color was found and we have at least two length
/// offsets).
pub fn replace_shadow_colors<F>(input: &str, mut replacement: F) -> String
where
  F: FnMut(&str) -> String,
{
  let shadows: Vec<String> = segment(input, ',')
    .into_iter()
    .map(|shadow| transform_shadow(shadow.trim(), &mut replacement))
    .collect();
  shadows.join(", ")
}

fn transform_shadow<F>(shadow: &str, replacement: &mut F) -> String
where
  F: FnMut(&str) -> String,
{
  let parts: Vec<String> = segment(shadow, ' ')
    .into_iter()
    .filter(|p| !p.trim().is_empty())
    .collect();
  let mut color: Option<String> = None;
  let mut offset_x: Option<String> = None;
  let mut offset_y: Option<String> = None;

  for part in &parts {
    if keywords().contains(part.as_str()) {
      continue;
    } else if length_re().is_match(part) {
      if offset_x.is_none() {
        offset_x = Some(part.clone());
      } else if offset_y.is_none() {
        offset_y = Some(part.clone());
      }
    } else if color.is_none() {
      color = Some(part.clone());
    }
  }

  if offset_x.is_none() || offset_y.is_none() {
    return shadow.to_string();
  }

  let replacement_color = match &color {
    Some(c) => replacement(c),
    None => replacement("currentcolor"),
  };

  match &color {
    Some(c) => shadow.replacen(c.as_str(), &replacement_color, 1),
    None => format!("{shadow} {replacement_color}"),
  }
}
