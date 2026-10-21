use crate::{ConfigError, NotFoundConfig, NotFoundLabels};
use serde::{de::Error, Deserialize};
use std::collections::BTreeMap;

impl<'de> Deserialize<'de> for NotFoundConfig {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    // Derived struct visitors also accept sequences. Public configuration must
    // match the object-only JSON schema, including each locale's label object.
    let value = serde_json::Value::deserialize(deserializer)?;
    if !value.is_object() {
      return Err(D::Error::custom("notFound must be an object"));
    }
    if let Some(labels) = value.get("labels") {
      let labels = labels
        .as_object()
        .ok_or_else(|| D::Error::custom("notFound.labels must be an object"))?;
      for (locale, labels) in labels {
        if !labels.is_object() {
          return Err(D::Error::custom(format!(
            "notFound.labels.{locale} must be an object"
          )));
        }
      }
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Wire {
      #[serde(default)]
      labels: BTreeMap<String, NotFoundLabels>,
    }
    let wire: Wire = serde_json::from_value(value).map_err(D::Error::custom)?;
    Ok(Self {
      labels: wire.labels,
    })
  }
}

pub(crate) fn resolve(
  config: &NotFoundConfig,
  locales: &[String],
) -> Result<NotFoundConfig, ConfigError> {
  for (locale, labels) in &config.labels {
    if !locales.contains(locale) {
      return Err(ConfigError::new(
        "DOCS_CONFIG_NOT_FOUND",
        &format!("notFound.labels.{locale}"),
        "must name a declared locale",
      ));
    }
    for (field, text) in [
      ("title", &labels.title),
      ("description", &labels.description),
      ("homeLabel", &labels.home_label),
    ] {
      if text.trim().is_empty() {
        return Err(ConfigError::new(
          "DOCS_CONFIG_NOT_FOUND",
          &format!("notFound.labels.{locale}.{field}"),
          "require non-empty text",
        ));
      }
    }
  }
  Ok(NotFoundConfig {
    labels: locales
      .iter()
      .map(|locale| {
        (
          locale.clone(),
          config.labels.get(locale).cloned().unwrap_or_default(),
        )
      })
      .collect(),
  })
}
