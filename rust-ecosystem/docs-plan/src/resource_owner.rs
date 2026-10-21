//! Internal resource provenance; contains no source paths or Farm resource types.
use serde::{Deserialize, Serialize};

pub const META_KEY: &str = "farm-docs:resource-owner";

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Stamp {
  version: u8,
  owner: String,
  name: String,
  digest: String,
}

pub fn stamp(owner: &str, name: &str, bytes: &[u8]) -> Result<String, String> {
  serde_json::to_string(&Stamp {
    version: 1,
    owner: owner.into(),
    name: name.into(),
    digest: crate::hash(bytes),
  })
  .map_err(|error| error.to_string())
}

pub fn is_owned(value: Option<&str>, owner: &str, name: &str, bytes: &[u8]) -> bool {
  value
    .and_then(|value| serde_json::from_str::<Stamp>(value).ok())
    .is_some_and(|stamp| {
      stamp.version == 1
        && stamp.owner == owner
        && stamp.name == name
        && stamp.digest == crate::hash(bytes)
    })
}
