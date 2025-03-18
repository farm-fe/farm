use compact_str::ToCompactString;
use regex::Regex;

use crate::String;

/// Farm config regex inner filter.
#[derive(Debug, Clone)]
pub enum InnerFilter {
  Contains(String),
  StartsWith(String),
  EndsWith(String),
  Regex(Regex),
}

/// Farm config regex.
///
/// # Example
///
/// ```rust
/// use farmfe_core::config::config_regex::ConfigRegex;
///
/// let regex = ConfigRegex::new("node_modules/");
/// assert!(regex.is_match("node_modules/"));
/// assert!(regex.is_match("node_modules/abc"));
/// assert!(!regex.is_match("node_modules"));
/// ```
#[derive(Debug, Clone)]
pub struct ConfigRegex(InnerFilter, bool);

impl Default for ConfigRegex {
  fn default() -> Self {
    Self(
      InnerFilter::StartsWith(String::new_inline("node_modules/")),
      false,
    )
  }
}

impl AsRef<str> for ConfigRegex {
  fn as_ref(&self) -> &str {
    match &self.0 {
      InnerFilter::Contains(s) => s.as_str(),
      InnerFilter::StartsWith(s) => s.as_str(),
      InnerFilter::EndsWith(s) => s.as_str(),
      InnerFilter::Regex(regex) => regex.as_str(),
    }
  }
}

impl<'de> serde::Deserialize<'de> for ConfigRegex {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let ss = String::deserialize(deserializer)?;
    let (s, is_not) = Self::parse_str(&ss);

    let regex = regex::Regex::new(s).map_err(serde::de::Error::custom)?;
    Ok(Self(InnerFilter::Regex(regex), is_not))
  }
}

impl serde::Serialize for ConfigRegex {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let is_not = self.1;
    match &self.0 {
      InnerFilter::Contains(s) => {
        let pattern = format!(".*{}.*", regex::escape(s));
        serializer.serialize_str(if is_not {
          format!("!{}", pattern).leak()
        } else {
          &pattern
        })
      }
      InnerFilter::StartsWith(s) => {
        let pattern = regex::escape(s);
        serializer.serialize_str(if is_not {
          format!("!{}", pattern).leak()
        } else {
          &pattern
        })
      }
      InnerFilter::EndsWith(s) => {
        let pattern = format!("{}$", regex::escape(s));
        serializer.serialize_str(if is_not {
          format!("!{}", pattern).leak()
        } else {
          &pattern
        })
      }
      InnerFilter::Regex(regex) => {
        let pattern = regex.as_str();
        serializer.serialize_str(if is_not {
          format!("!{}", pattern).leak()
        } else {
          pattern
        })
      }
    }
  }
}

impl ConfigRegex {
  /// Create a new regex.
  pub fn new(regex: &str) -> Self {
    let (s, is_not) = Self::parse_str(regex);
    Self(InnerFilter::Regex(regex::Regex::new(s).unwrap()), is_not)
  }

  /// Create a new regex, alias of `new`.
  pub fn new_regex(haystack: &str) -> Self {
    Self::new(haystack)
  }

  /// Create a [ConfigRegex] that matches the beginning of the string.
  pub fn new_starts_with(haystack: &str) -> Self {
    let (s, is_not) = Self::parse_str(haystack);
    Self(InnerFilter::StartsWith(s.to_compact_string()), is_not)
  }

  /// Create a [ConfigRegex] that matches the end of the string.
  pub fn new_ends_with(haystack: &str) -> Self {
    let (s, is_not) = Self::parse_str(haystack);
    Self(InnerFilter::EndsWith(s.to_compact_string()), is_not)
  }

  /// Create a [ConfigRegex] that matches the string.
  pub fn new_contains(haystack: &str) -> Self {
    let (s, is_not) = Self::parse_str(haystack);
    Self(InnerFilter::Contains(s.to_compact_string()), is_not)
  }

  /// Check if the regex matches the haystack.
  ///
  /// # Example
  ///
  /// Using regex:
  ///
  /// ```rust
  /// use farmfe_core::config::config_regex::ConfigRegex;
  ///
  /// let regex = ConfigRegex::new("node_modules/");
  /// // or `let regex = ConfigRegex::new_regex("node_modules/");`
  /// assert!(regex.is_match("node_modules/"));
  /// assert!(regex.is_match("node_modules/abc"));
  /// assert!(!regex.is_match("node_modules"));
  /// ```
  ///
  /// Using starts with:
  ///
  /// ```rust
  /// use farmfe_core::config::config_regex::ConfigRegex;
  ///
  /// let regex = ConfigRegex::new_starts_with("node_modules/");
  /// assert!(regex.is_match("node_modules/"));
  /// assert!(regex.is_match("node_modules/abc"));
  /// assert!(!regex.is_match("node_modules"));
  /// ```
  pub fn is_match(&self, haystack: &str) -> bool {
    (match &self.0 {
      InnerFilter::Contains(s) => haystack.contains(s.as_str()),
      InnerFilter::StartsWith(s) => haystack.starts_with(s.as_str()),
      InnerFilter::EndsWith(s) => haystack.ends_with(s.as_str()),
      InnerFilter::Regex(regex) => regex.is_match(haystack),
    }) ^ self.1
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
    let regex = ConfigRegex::new_regex("node_modules/");
    assert!(regex.is_match("node_modules/"));
    assert!(regex.is_match("node_modules/abc"));
    assert!(!regex.is_match("node_modules"));
    assert!(!regex.is_match("/path/to/node_modules"));
  }

  #[test]
  fn test_config_starts_with() {
    let regex = ConfigRegex::new_starts_with("node_modules/");
    assert!(regex.is_match("node_modules/"));
    assert!(regex.is_match("node_modules/abc"));
    assert!(!regex.is_match("node_modules"));
    assert!(!regex.is_match("/path/to/node_modules"));
  }

  #[test]
  fn test_config_ends_with() {
    let regex = ConfigRegex::new_ends_with(".json");
    assert!(regex.is_match("package.json"));
    assert!(!regex.is_match("json.config"));
  }

  #[test]
  fn test_config_regex_not() {
    let regex = ConfigRegex::new("!node_modules/");
    assert!(!regex.is_match("node_modules/"));
    assert!(!regex.is_match("node_modules/abc"));
    assert!(regex.is_match("node_modules"));
    assert!(regex.is_match("/path/to/node_modules"));
  }

  #[test]
  fn test_serialization() {
    let contains = ConfigRegex::new_contains("middle*");
    assert_eq!(
      serde_json::to_string(&contains).unwrap(),
      r#"".*middle\\*.*""#
    );

    let starts_with = ConfigRegex::new_starts_with("node_");
    assert_eq!(serde_json::to_string(&starts_with).unwrap(), r#""node_""#);

    let ends_with = ConfigRegex::new_ends_with(".json");
    assert_eq!(serde_json::to_string(&ends_with).unwrap(), r#""\\.json$""#);

    let not_contains = ConfigRegex::new_contains("!middle");
    assert_eq!(
      serde_json::to_string(&not_contains).unwrap(),
      r#""!.*middle.*""#
    );
  }
}
