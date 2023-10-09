use farmfe_utils::transform_string_to_static_str;

#[derive(Debug, Clone)]
pub struct ConfigRegex(regex::Regex, bool);

impl Default for ConfigRegex {
  fn default() -> Self {
    Self(regex::Regex::new("node_modules/").unwrap(), false)
  }
}

// implement serde::Deserialize for ConfigRegex
impl<'de> serde::Deserialize<'de> for ConfigRegex {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let ss = String::deserialize(deserializer)?;
    let (s, is_not) = Self::parse_str(&ss);

    let regex = regex::Regex::new(s).map_err(serde::de::Error::custom)?;
    Ok(Self(regex, is_not))
  }
}

// implement serde::Serialize for ConfigRegex
impl serde::Serialize for ConfigRegex {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(if !self.1 {
      self.0.as_str()
    } else {
      transform_string_to_static_str(format!("!{}", self.0.as_str()))
    })
  }
}

impl ConfigRegex {
  pub fn is_match(&self, s: &str) -> bool {
    if self.1 {
      return !self.0.is_match(s);
    }

    self.0.is_match(s)
  }

  pub fn new(ss: &str) -> Self {
    let (s, is_not) = Self::parse_str(ss);
    Self(regex::Regex::new(s).unwrap(), is_not)
  }

  fn parse_str(ss: &str) -> (&str, bool) {
    let mut is_not = false;
    let s = if let Some(stripped) = ss.strip_prefix('!') {
      is_not = true;
      stripped
    } else {
      ss
    };
    (s, is_not)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_config_regex() {
    let regex = ConfigRegex::new("node_modules/");
    assert!(regex.is_match("node_modules/"));
    assert!(regex.is_match("node_modules/abc"));
    assert!(!regex.is_match("node_modules"));
  }

  #[test]
  fn test_config_regex_not() {
    let regex = ConfigRegex::new("!node_modules/");
    assert!(!regex.is_match("node_modules/"));
    assert!(!regex.is_match("node_modules/abc"));
    assert!(regex.is_match("node_modules"));
  }
}
