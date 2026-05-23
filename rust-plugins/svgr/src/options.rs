#![deny(clippy::all)]

use farmfe_core::{serde, serde_json};
use serde_json::Value;

#[derive(Debug, serde::Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Options {
  pub include: Option<Vec<String>>,
  pub exclude: Option<Vec<String>>,
  pub default_style: Option<Value>,
  pub default_class: Option<String>,
}
