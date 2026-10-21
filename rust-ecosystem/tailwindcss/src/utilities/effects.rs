use crate::{
  candidate::ParsedCandidate,
  theme::{Theme, ThemeOptions},
};

pub(super) const FILTER: &str = "var(--tw-blur,) var(--tw-invert,)";

fn valid(candidate: &ParsedCandidate) -> bool {
  !candidate.negative && candidate.modifier.is_none()
}

pub(super) fn content(c: &ParsedCandidate, _: &Theme) -> Option<Vec<(String, String)>> {
  if !valid(c) {
    return None;
  }
  let value = c
    .arbitrary_value
    .clone()
    .or_else(|| match c.utility_value.as_deref() {
      Some("none") => Some("none".into()),
      _ => None,
    })?;
  Some(vec![("content".into(), value)])
}

pub(super) fn blur(c: &ParsedCandidate, theme: &Theme) -> Option<Vec<(String, String)>> {
  if !valid(c) {
    return None;
  }
  let value = if let Some(value) = &c.arbitrary_value {
    value.clone()
  } else if c.utility_value.as_deref() == Some("none") {
    "0".into()
  } else {
    theme.resolve(c.utility_value.as_deref(), &["--blur"], ThemeOptions::NONE)?
  };
  Some(vec![
    ("--tw-blur".into(), format!("blur({value})")),
    ("filter".into(), FILTER.into()),
  ])
}

pub(super) fn invert(c: &ParsedCandidate, _: &Theme) -> Option<Vec<(String, String)>> {
  if !valid(c) {
    return None;
  }
  let value = if let Some(value) = &c.arbitrary_value {
    value.clone()
  } else if let Some(value) = &c.utility_value {
    let number = value.parse::<f64>().ok()?;
    if !number.is_finite() || !(0.0..=100.0).contains(&number) {
      return None;
    }
    format!("{value}%")
  } else {
    "100%".into()
  };
  Some(vec![
    ("--tw-invert".into(), format!("invert({value})")),
    ("filter".into(), FILTER.into()),
  ])
}

pub(super) fn mix_blend(c: &ParsedCandidate, _: &Theme) -> Option<Vec<(String, String)>> {
  if !valid(c) {
    return None;
  }
  let value = c.utility_value.as_deref()?;
  if ![
    "normal",
    "multiply",
    "screen",
    "overlay",
    "darken",
    "lighten",
    "color-dodge",
    "color-burn",
    "hard-light",
    "soft-light",
    "difference",
    "exclusion",
    "hue",
    "saturation",
    "color",
    "luminosity",
    "plus-lighter",
    "plus-darker",
  ]
  .contains(&value)
  {
    return None;
  }
  Some(vec![("mix-blend-mode".into(), value.into())])
}

pub(super) fn will_change(c: &ParsedCandidate, _: &Theme) -> Option<Vec<(String, String)>> {
  if !valid(c) {
    return None;
  }
  let value = c
    .arbitrary_value
    .clone()
    .or_else(|| match c.utility_value.as_deref() {
      Some(value @ ("auto" | "scroll-position" | "contents" | "transform")) => Some(value.into()),
      _ => None,
    })?;
  Some(vec![("will-change".into(), value)])
}
