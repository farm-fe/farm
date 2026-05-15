//! Data-type inference, ported from
//! `packages/tailwindcss/src/utils/infer-data-type.ts`.
//!
//! Upstream has no dedicated `.test.ts` for this module (only a bench), so
//! the integration tests here are derived from documented behaviour.

use super::is_color::is_color;
use super::math_operators::has_math_fn;
use super::segment::segment;
use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
  Color,
  Length,
  Percentage,
  Ratio,
  Number,
  Integer,
  Url,
  Position,
  BgSize,
  LineWidth,
  Image,
  FamilyName,
  GenericName,
  AbsoluteSize,
  RelativeSize,
  Angle,
  Vector,
}

/// Determine the type of `value` against an ordered list of candidate types.
pub fn infer_data_type(value: &str, types: &[DataType]) -> Option<DataType> {
  if value.starts_with("var(") {
    return None;
  }
  for ty in types {
    if check(*ty, value) {
      return Some(*ty);
    }
  }
  None
}

fn check(ty: DataType, value: &str) -> bool {
  match ty {
    DataType::Color => is_color(value),
    DataType::Length => is_length(value),
    DataType::Percentage => is_percentage(value),
    DataType::Ratio => is_fraction(value),
    DataType::Number => is_number(value),
    DataType::Integer => is_positive_integer(value),
    DataType::Url => is_url(value),
    DataType::Position => is_background_position(value),
    DataType::BgSize => is_background_size(value),
    DataType::LineWidth => is_line_width(value),
    DataType::Image => is_image(value),
    DataType::FamilyName => is_family_name(value),
    DataType::GenericName => is_generic_name(value),
    DataType::AbsoluteSize => is_absolute_size(value),
    DataType::RelativeSize => is_relative_size(value),
    DataType::Angle => is_angle(value),
    DataType::Vector => is_vector(value),
  }
}

// ----- Number-shaped sub-regexes ----------------------------------------------

const HAS_NUMBER_SRC: &str = r"[+-]?\d*\.?\d+(?:[eE][+-]?\d+)?";

fn is_number_re() -> &'static Regex {
  static RE: OnceLock<Regex> = OnceLock::new();
  RE.get_or_init(|| Regex::new(&format!("^{HAS_NUMBER_SRC}$")).unwrap())
}
fn is_percentage_re() -> &'static Regex {
  static RE: OnceLock<Regex> = OnceLock::new();
  RE.get_or_init(|| Regex::new(&format!("^{HAS_NUMBER_SRC}%$")).unwrap())
}
fn is_fraction_re() -> &'static Regex {
  static RE: OnceLock<Regex> = OnceLock::new();
  RE.get_or_init(|| {
    Regex::new(&format!(r"^{HAS_NUMBER_SRC}\s*/\s*{HAS_NUMBER_SRC}$")).unwrap()
  })
}

const LENGTH_UNITS: &[&str] = &[
  "cm", "mm", "Q", "in", "pc", "pt", "px", "em", "ex", "ch", "rem", "lh",
  "rlh", "vw", "vh", "vmin", "vmax", "vb", "vi", "svw", "svh", "lvw", "lvh",
  "dvw", "dvh", "cqw", "cqh", "cqi", "cqb", "cqmin", "cqmax",
];

fn is_length_re() -> &'static Regex {
  static RE: OnceLock<Regex> = OnceLock::new();
  RE.get_or_init(|| {
    Regex::new(&format!("^{}({})$", HAS_NUMBER_SRC, LENGTH_UNITS.join("|"))).unwrap()
  })
}

const ANGLE_UNITS: &[&str] = &["deg", "rad", "grad", "turn"];

fn is_angle_re() -> &'static Regex {
  static RE: OnceLock<Regex> = OnceLock::new();
  RE.get_or_init(|| {
    Regex::new(&format!("^{}({})$", HAS_NUMBER_SRC, ANGLE_UNITS.join("|"))).unwrap()
  })
}

fn is_vector_re() -> &'static Regex {
  static RE: OnceLock<Regex> = OnceLock::new();
  RE.get_or_init(|| {
    Regex::new(&format!(
      r"^{} +{} +{}$",
      HAS_NUMBER_SRC, HAS_NUMBER_SRC, HAS_NUMBER_SRC
    ))
    .unwrap()
  })
}

fn is_url_re() -> &'static Regex {
  static RE: OnceLock<Regex> = OnceLock::new();
  RE.get_or_init(|| Regex::new(r"^url\(.*\)$").unwrap())
}

fn is_image_fn_re() -> &'static Regex {
  static RE: OnceLock<Regex> = OnceLock::new();
  RE.get_or_init(|| Regex::new(r"^(?:element|image|cross-fade|image-set)\(").unwrap())
}

fn is_gradient_fn_re() -> &'static Regex {
  static RE: OnceLock<Regex> = OnceLock::new();
  RE.get_or_init(|| Regex::new(r"^(?:repeating-)?(?:conic|linear|radial)-gradient\(").unwrap())
}

// ----- Public checkers --------------------------------------------------------

pub fn is_number(value: &str) -> bool {
  is_number_re().is_match(value) || has_math_fn(value)
}

pub fn is_percentage(value: &str) -> bool {
  is_percentage_re().is_match(value) || has_math_fn(value)
}

pub fn is_fraction(value: &str) -> bool {
  is_fraction_re().is_match(value) || has_math_fn(value)
}

pub fn is_length(value: &str) -> bool {
  is_length_re().is_match(value) || has_math_fn(value)
}

pub fn is_angle(value: &str) -> bool {
  is_angle_re().is_match(value)
}

pub fn is_vector(value: &str) -> bool {
  is_vector_re().is_match(value)
}

pub fn is_url(value: &str) -> bool {
  is_url_re().is_match(value)
}

/// JS-Number semantics: integer ≥ 0, and `String(num) === String(value)`.
pub fn is_positive_integer(value: &str) -> bool {
  is_integer_with(value, |n| n >= 0)
}

pub fn is_strict_positive_integer(value: &str) -> bool {
  is_integer_with(value, |n| n > 0)
}

fn is_integer_with(value: &str, pred: impl Fn(i64) -> bool) -> bool {
  let n: i64 = match value.parse() {
    Ok(n) => n,
    Err(_) => return false,
  };
  if !pred(n) {
    return false;
  }
  n.to_string() == value
}

pub fn is_valid_spacing_multiplier(value: &str) -> bool {
  is_multiple_of(value, 0.25)
}

pub fn is_valid_opacity_value(value: &str) -> bool {
  is_multiple_of(value, 0.25)
}

fn is_multiple_of(value: &str, divisor: f64) -> bool {
  let n: f64 = match value.parse() {
    Ok(n) => n,
    Err(_) => return false,
  };
  if n < 0.0 {
    return false;
  }
  let q = n / divisor;
  let rounded = q.round();
  if (q - rounded).abs() > f64::EPSILON * 8.0 {
    return false;
  }
  // Match JS `String(num) === String(value)` round-trip:
  // 1.0 -> "1", 1.5 -> "1.5"
  let canonical = if n == n.trunc() {
    format!("{}", n as i64)
  } else {
    format!("{n}")
  };
  canonical == value
}

// ----- Helpers ----------------------------------------------------------------

fn is_generic_name(value: &str) -> bool {
  matches!(
    value,
    "serif"
      | "sans-serif"
      | "monospace"
      | "cursive"
      | "fantasy"
      | "system-ui"
      | "ui-serif"
      | "ui-sans-serif"
      | "ui-monospace"
      | "ui-rounded"
      | "math"
      | "emoji"
      | "fangsong"
  )
}

fn is_absolute_size(value: &str) -> bool {
  matches!(
    value,
    "xx-small"
      | "x-small"
      | "small"
      | "medium"
      | "large"
      | "x-large"
      | "xx-large"
      | "xxx-large"
  )
}

fn is_relative_size(value: &str) -> bool {
  matches!(value, "larger" | "smaller")
}

fn is_line_width(value: &str) -> bool {
  segment(value, ' ').iter().all(|p| {
    let p = p.as_str();
    is_length(p) || is_number(p) || matches!(p, "thin" | "medium" | "thick")
  })
}

fn is_image(value: &str) -> bool {
  let mut count = 0usize;
  for part in segment(value, ',') {
    let part = part.trim();
    if part.starts_with("var(") {
      continue;
    }
    if is_url(part) || is_gradient_fn_re().is_match(part) || is_image_fn_re().is_match(part) {
      count += 1;
      continue;
    }
    return false;
  }
  count > 0
}

fn is_family_name(value: &str) -> bool {
  let mut count = 0usize;
  for part in segment(value, ',') {
    let part = part.trim_start();
    if let Some(b) = part.as_bytes().first() {
      if (b'0'..=b'9').contains(b) {
        return false;
      }
    }
    if part.starts_with("var(") {
      continue;
    }
    count += 1;
  }
  count > 0
}

fn is_background_position(value: &str) -> bool {
  let mut count = 0usize;
  for part in segment(value, ' ') {
    let p = part.as_str();
    if matches!(p, "center" | "top" | "right" | "bottom" | "left") {
      count += 1;
      continue;
    }
    if p.starts_with("var(") {
      continue;
    }
    if is_length(p) || is_percentage(p) {
      count += 1;
      continue;
    }
    return false;
  }
  count > 0
}

fn is_background_size(value: &str) -> bool {
  let mut count = 0usize;
  for size in segment(value, ',') {
    let size = size.trim();
    if matches!(size, "cover" | "contain") {
      count += 1;
      continue;
    }
    let values = segment(size, ' ');
    if values.len() != 1 && values.len() != 2 {
      return false;
    }
    if values
      .iter()
      .all(|v| v.as_str() == "auto" || is_length(v.as_str()) || is_percentage(v.as_str()))
    {
      count += 1;
    }
  }
  count > 0
}
