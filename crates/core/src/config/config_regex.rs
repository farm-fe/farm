#[derive(Debug, Clone)]
pub struct ConfigRegex(pub regex::Regex);

impl Default for ConfigRegex {
  fn default() -> Self {
    Self(regex::Regex::new("node_modules/").unwrap())
  }
}

// implement serde::Deserialize for ConfigRegex
impl<'de> serde::Deserialize<'de> for ConfigRegex {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    let regex = regex::Regex::new(&s).map_err(serde::de::Error::custom)?;
    Ok(Self(regex))
  }
}

// implement serde::Serialize for ConfigRegex
impl serde::Serialize for ConfigRegex {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(self.0.as_str())
  }
}

impl ConfigRegex {
  pub fn is_match(&self, s: &str) -> bool {
    self.0.is_match(s)
  }

  pub fn new(s: &str) -> Self {
    Self(regex::Regex::new(s).unwrap())
  }
}
