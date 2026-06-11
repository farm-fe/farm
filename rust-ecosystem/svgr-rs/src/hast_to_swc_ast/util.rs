use lazy_static::lazy_static;
use regex::Regex;

pub fn is_numeric(s: &str) -> bool {
  lazy_static! {
    static ref NUMERIC_REGEX: Regex = Regex::new(r#"^(\-|\+)?\d+(\.\d+)?$"#).unwrap();
  }
  NUMERIC_REGEX.is_match(s)
}
