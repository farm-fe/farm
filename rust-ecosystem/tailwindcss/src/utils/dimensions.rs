//! Dimension parsing helper, ported from
//! `packages/tailwindcss/src/utils/dimensions.ts`.

use regex::Regex;
use std::sync::OnceLock;

/// Result of parsing a dimension: `(value, optional_unit)`.
///
/// - `("64rem")` → `Some((64.0, Some("rem")))`
/// - `("100%")` → `Some((100.0, Some("%")))`
/// - `("0.5")` → `Some((0.5, None))`
/// - `("abc")` → `None`
#[derive(Debug, Clone, PartialEq)]
pub struct Dimension {
  pub value: f64,
  pub unit: Option<String>,
}

fn re() -> &'static Regex {
  static RE: OnceLock<Regex> = OnceLock::new();
  RE.get_or_init(|| Regex::new(r"^(?P<value>[-+]?(?:\d*\.)?\d+)(?P<unit>[a-zA-Z]+|%)?$").unwrap())
}

/// Parse a dimension such as `64rem` into a [`Dimension`].
pub fn parse_dimension(input: &str) -> Option<Dimension> {
  let caps = re().captures(input)?;
  let value_str = caps.name("value")?.as_str();
  let value: f64 = value_str.parse().ok()?;
  let unit = caps.name("unit").map(|m| m.as_str().to_string());
  Some(Dimension { value, unit })
}
