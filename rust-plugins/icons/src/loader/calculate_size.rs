use farmfe_core::regex::Regex;
use farmfe_toolkit::lazy_static::lazy_static;

lazy_static! {
  static ref UNITS_SPLIT: Regex = Regex::new(r"(-?\d*\.?\d+)([a-zA-Z%]*)").unwrap();
}

pub fn calculate_size(size: &str, ratio: f32, precision: Option<u32>) -> String {
  if ratio == 1.0 {
    return size.to_string();
  }

  let precision = precision.unwrap_or(100);

  let captures = UNITS_SPLIT.captures(size);
  if captures.is_none() {
    return size.to_string();
  }

  let captures = captures.unwrap();
  let number_part = captures.get(1).map_or("", |m| m.as_str());
  let unit_part = captures.get(2).map_or("", |m| m.as_str());

  let new_number = match number_part.parse::<f32>() {
    Ok(num) => {
      let new_num = (num * ratio * precision as f32).ceil() / precision as f32;
      new_num.to_string()
    }
    Err(_) => number_part.to_string(),
  };

  format!("{new_number}{unit_part}")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_calculate_size() {
    assert_eq!(calculate_size("1em", 2.0, Some(100)), "2em");
    assert_eq!(calculate_size("100px", 0.5, Some(100)), "50px");
    assert_eq!(calculate_size("50%", 1.5, Some(100)), "75%");
    assert_eq!(calculate_size("10.5pt", 2.0, Some(100)), "21pt");
    assert_eq!(calculate_size("1em", 1.0, Some(100)), "1em");
  }
}
