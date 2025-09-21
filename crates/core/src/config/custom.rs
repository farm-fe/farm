use serde::de::DeserializeOwned;

use super::Config;

pub fn get_field_or_default_from_custom<T: Default + DeserializeOwned>(
  config: &Config,
  field: &str,
) -> T {
  config
    .custom
    .get(field)
    .map(|val| serde_json::from_str(val).unwrap_or_default())
    .unwrap_or_default()
}
