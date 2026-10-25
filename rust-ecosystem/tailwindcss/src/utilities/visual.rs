//! Composable visual utilities used by normal application layouts.
use super::{maybe_neg, resolve_spacing};
use crate::{
  candidate::ParsedCandidate,
  theme::{Theme, ThemeOptions},
};
type Declarations = Option<Vec<(String, String)>>;
pub(super) const TRANSFORM: &str =
  "var(--tw-transform-gpu,) translate(var(--tw-translate-x, 0), var(--tw-translate-y, 0))";
fn valid(c: &ParsedCandidate) -> bool {
  !c.negative && c.modifier.is_none()
}
fn value(c: &ParsedCandidate, t: &Theme, namespace: &str) -> Option<String> {
  c.arbitrary_value
    .clone()
    .or_else(|| t.resolve(c.utility_value.as_deref(), &[namespace], ThemeOptions::NONE))
}
pub(super) fn backdrop_blur(c: &ParsedCandidate, t: &Theme) -> Declarations {
  if !valid(c) {
    return None;
  }
  let v = if c.utility_value.as_deref() == Some("none") {
    "0".into()
  } else {
    value(c, t, "--blur")?
  };
  Some(vec![
    ("--tw-backdrop-blur".into(), format!("blur({v})")),
    (
      "-webkit-backdrop-filter".into(),
      "var(--tw-backdrop-blur,)".into(),
    ),
    ("backdrop-filter".into(), "var(--tw-backdrop-blur,)".into()),
  ])
}
pub(super) fn shadow(c: &ParsedCandidate, t: &Theme) -> Declarations {
  if !valid(c) {
    return None;
  }
  let v = if c.utility_value.as_deref() == Some("none") {
    "0 0 #0000".into()
  } else {
    value(c, t, "--shadow")?
  };
  Some(vec![("--tw-shadow".into(),v),("box-shadow".into(),"var(--tw-ring-offset-shadow, 0 0 #0000), var(--tw-ring-shadow, 0 0 #0000), var(--tw-shadow, 0 0 #0000)".into())])
}
pub(super) fn transition(c: &ParsedCandidate, _: &Theme) -> Declarations {
  if !valid(c) {
    return None;
  }
  let property = if let Some(v) = &c.arbitrary_value {
    v.as_str()
  } else {
    match c.utility_value.as_deref() {
 None=>"color, background-color, border-color, text-decoration-color, fill, stroke, opacity, box-shadow, transform, filter, backdrop-filter, -webkit-backdrop-filter",
 Some("all")=>"all",Some("shadow")=>"box-shadow",Some("transform")=>"transform",Some("none")=>"none",_=>return None}
  };
  Some(vec![
    ("transition-property".into(), property.into()),
    (
      "transition-duration".into(),
      "var(--tw-duration, 150ms)".into(),
    ),
    (
      "transition-timing-function".into(),
      "var(--tw-ease, cubic-bezier(0.4, 0, 0.2, 1))".into(),
    ),
  ])
}
pub(super) fn duration(c: &ParsedCandidate, t: &Theme) -> Declarations {
  if !valid(c) {
    return None;
  }
  let v = if let Some(v) = &c.arbitrary_value {
    v.clone()
  } else if let Some(v) = c
    .utility_value
    .as_deref()
    .filter(|v| v.parse::<u32>().is_ok())
  {
    format!("{v}ms")
  } else {
    value(c, t, "--duration")?
  };
  Some(vec![
    ("--tw-duration".into(), v.clone()),
    ("transition-duration".into(), v),
  ])
}
pub(super) fn ease(c: &ParsedCandidate, t: &Theme) -> Declarations {
  if !valid(c) {
    return None;
  }
  let v = value(c, t, "--ease")?;
  Some(vec![
    ("--tw-ease".into(), v.clone()),
    ("transition-timing-function".into(), v),
  ])
}
pub(super) fn translate(c: &ParsedCandidate, t: &Theme) -> Declarations {
  if c.modifier.is_some() {
    return None;
  }
  let v = resolve_spacing(c, t)?;
  let axis = c.utility_root.strip_prefix("translate-")?;
  Some(vec![
    (format!("--tw-translate-{axis}"), v),
    ("transform".into(), TRANSFORM.into()),
  ])
}
pub(super) fn tracking(c: &ParsedCandidate, t: &Theme) -> Declarations {
  if c.modifier.is_some() {
    return None;
  }
  let mut v = value(c, t, "--tracking")?;
  if c.negative {
    v = maybe_neg(v, true);
  }
  Some(vec![("letter-spacing".into(), v)])
}
pub(super) fn space(c: &ParsedCandidate, t: &Theme) -> Declarations {
  if c.modifier.is_some() {
    return None;
  }
  let v = resolve_spacing(c, t)?;
  let axis = c.utility_root.strip_prefix("space-")?;
  let (start, end) = if axis == "x" {
    ("margin-left", "margin-right")
  } else {
    ("margin-top", "margin-bottom")
  };
  Some(vec![
    (format!("--tw-space-{axis}-reverse"), "0".into()),
    (
      start.into(),
      format!("calc({v} * (1 - var(--tw-space-{axis}-reverse)))"),
    ),
    (
      end.into(),
      format!("calc({v} * var(--tw-space-{axis}-reverse))"),
    ),
  ])
}
