use serde::{
    de::{Error, Unexpected, Visitor},
    Deserialize, Deserializer,
};
use std::{fmt, ops::Deref};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Options {
    pub transform_on: bool,
    pub optimize: bool,
    pub custom_element_patterns: Vec<Regex>,
    pub merge_props: bool,
    pub enable_object_slots: bool,
    pub pragma: Option<String>,
    pub resolve_type: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            transform_on: false,
            optimize: false,
            custom_element_patterns: Default::default(),
            merge_props: true,
            enable_object_slots: true,
            pragma: None,
            resolve_type: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Regex(regex::Regex);

impl Regex {
    pub fn new(re: &str) -> Result<Self, regex::Error> {
        regex::Regex::new(re).map(Self)
    }
}

impl From<regex::Regex> for Regex {
    fn from(value: regex::Regex) -> Self {
        Self(value)
    }
}

impl Deref for Regex {
    type Target = regex::Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Regex {
    fn deserialize<D>(deserializer: D) -> Result<Regex, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(RegexVisitor)
    }
}

struct RegexVisitor;

impl Visitor<'_> for RegexVisitor {
    type Value = Regex;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string that represents a regex")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        regex::Regex::new(v)
            .map(Regex)
            .map_err(|_| E::invalid_value(Unexpected::Str(v), &"a valid regex"))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        regex::Regex::new(&v)
            .map(Regex)
            .map_err(|_| E::invalid_value(Unexpected::Str(&v), &"a valid regex"))
    }
}
