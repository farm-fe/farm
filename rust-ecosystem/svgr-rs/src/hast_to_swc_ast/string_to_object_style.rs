use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::{Captures, Regex};
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;

use super::util::*;

pub fn hyphen_to_camel_case(s: &str) -> Cow<str> {
  lazy_static! {
    static ref HYPHEN_REGEX: Regex = Regex::new(r#"-(.)"#).unwrap();
  }
  HYPHEN_REGEX.replace_all(s, |caps: &Captures| caps[1].to_uppercase())
}

// Format style key into JSX style object key.
pub fn format_key(key: &str) -> PropName {
  lazy_static! {
    static ref VAR_REGEX: Regex = Regex::new(r#"^--"#).unwrap();
    static ref MS_REGEX: Regex = Regex::new(r#"^-ms-"#).unwrap();
  }

  if VAR_REGEX.is_match(key) {
    return PropName::Str(Str {
      span: DUMMY_SP,
      value: key.into(),
      raw: None,
    });
  }

  let mut key = key.to_lowercase();
  if MS_REGEX.is_match(&key) {
    key = key[1..].into();
  }

  PropName::Ident(IdentName::new(hyphen_to_camel_case(&key).into(), DUMMY_SP))
}

fn is_convertible_pixel_value(s: &str) -> bool {
  lazy_static! {
    static ref PX_REGEX: Regex = Regex::new(r#"^\d+px$"#).unwrap();
  }
  PX_REGEX.is_match(s)
}

// Format style value into JSX style object value.
pub fn format_value(value: &str) -> Expr {
  if is_numeric(value) {
    return Expr::Lit(Lit::Num(Number {
      span: DUMMY_SP,
      value: value.parse().unwrap(),
      raw: None,
    }));
  }

  if is_convertible_pixel_value(value) {
    return Expr::Lit(Lit::Num(Number {
      span: DUMMY_SP,
      value: value[..value.len() - 2].parse().unwrap(),
      raw: None,
    }));
  }

  Expr::Lit(Lit::Str(Str {
    span: DUMMY_SP,
    value: value.into(),
    raw: None,
  }))
}

pub fn string_to_object_style(raw_style: &str) -> Expr {
  let entries = raw_style.split(';');

  let properties = entries
    .into_iter()
    .filter_map(|entry| {
      let style = entry.trim();
      if style.is_empty() {
        return None;
      }

      let first_colon = style.find(':');
      match first_colon {
        Some(i) => {
          let value = format_value(style[(i + 1)..].trim());
          let key = format_key(style[..i].trim());

          Some(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
            key,
            value: Box::new(value),
          }))))
        }
        None => None,
      }
    })
    .collect::<Vec<PropOrSpread>>();

  Expr::Object(ObjectLit {
    span: DUMMY_SP,
    props: properties,
  })
}
