use crate::{AtomConfig, ConfigError, OpenSearchConfig, SearchConfig, SearchProvider, SeoConfig};
use chrono::Datelike;
use serde::{
  de::{DeserializeOwned, Error, MapAccess, Visitor},
  Deserialize, Deserializer,
};

// Only object maps are accepted, and duplicates must be rejected before a Value
// can collapse them. Both syndication objects have scalar leaf fields.
fn object<'de, D: Deserializer<'de>, T: DeserializeOwned>(deserializer: D) -> Result<T, D::Error> {
  struct Object;
  impl<'de> Visitor<'de> for Object {
    type Value = serde_json::Value;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a syndication configuration object")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
      let mut result = serde_json::Map::new();
      while let Some(key) = map.next_key::<String>()? {
        if result.contains_key(&key) {
          return Err(M::Error::custom(format!("duplicate field {key}")));
        }
        result.insert(key, map.next_value()?);
      }
      Ok(serde_json::Value::Object(result))
    }
  }
  serde_json::from_value(deserializer.deserialize_map(Object)?).map_err(D::Error::custom)
}

pub(crate) fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
  deserializer: D,
) -> Result<Option<T>, D::Error> {
  T::deserialize(deserializer).map(Some)
}

impl<'de> Deserialize<'de> for AtomConfig {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Wire {
      updated: String,
      author: String,
    }
    let wire: Wire = object(deserializer)?;
    Ok(Self {
      updated: wire.updated,
      author: wire.author,
    })
  }
}
impl<'de> Deserialize<'de> for OpenSearchConfig {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Wire {
      page_id: String,
      short_name: String,
      description: String,
    }
    let wire: Wire = object(deserializer)?;
    Ok(Self {
      page_id: wire.page_id,
      short_name: wire.short_name,
      description: wire.description,
    })
  }
}

fn error(field: &str, message: &str) -> ConfigError {
  ConfigError::new("DOCS_CONFIG_SYNDICATION", field, message)
}
fn text(value: &str, field: &str) -> Result<(), ConfigError> {
  if value.trim().is_empty()
    || value.chars().any(|ch| {
      (ch < '\u{20}' && !matches!(ch, '\n' | '\r' | '\t')) || matches!(ch, '\u{FFFE}' | '\u{FFFF}')
    })
  {
    return Err(error(field, "require nonempty XML-compatible text"));
  }
  Ok(())
}
pub fn validate(seo: &SeoConfig, search: Option<&SearchConfig>) -> Result<(), ConfigError> {
  if let Some(atom) = &seo.atom {
    text(&atom.author, "seo.atom.author")?;
    let date = chrono::DateTime::parse_from_rfc3339(&atom.updated).map_err(|_| {
      error(
        "seo.atom.updated",
        "require RFC3339 timestamp with timezone",
      )
    })?;
    if !(1..=9999).contains(&date.year()) {
      return Err(error("seo.atom.updated", "year must be within 0001..9999"));
    }
  }
  if let Some(search) = search {
    if let Some(open) = &search.open_search {
      if !matches!(search.provider, SearchProvider::Algolia) {
        return Err(error("search.openSearch", "require the algolia provider"));
      }
      for (field, value, maximum) in [
        ("shortName", &open.short_name, 16),
        ("description", &open.description, 1024),
      ] {
        let field = format!("search.openSearch.{field}");
        text(value, &field)?;
        if value.chars().count() > maximum {
          return Err(error(&field, "text exceeds the OpenSearch character limit"));
        }
      }
      text(&open.page_id, "search.openSearch.pageId")?;
      if open.page_id.contains(['?', '#', '\\', ':'])
        || open.page_id.chars().any(char::is_control)
        || open
          .page_id
          .split('/')
          .any(|part| part.is_empty() || matches!(part, "." | ".."))
      {
        return Err(error(
          "search.openSearch.pageId",
          "require a relative logical page id, not a URL",
        ));
      }
    }
  }
  Ok(())
}
