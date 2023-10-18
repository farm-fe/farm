use relative_path::RelativePath;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum PersistentCacheConfig {
  Bool(bool),
  Obj(PersistentCacheConfigObj),
}

impl PersistentCacheConfig {
  pub fn enabled(&self) -> bool {
    match self {
      PersistentCacheConfig::Bool(b) => *b,
      PersistentCacheConfig::Obj(_) => true,
    }
  }

  pub fn get_default_config(root: &str) -> Self {
    let cache_dir = RelativePath::new("node_modules/.farm/cache")
      .to_logical_path(&root)
      .to_string_lossy()
      .to_string();

    PersistentCacheConfig::Obj(PersistentCacheConfigObj {
      namespace: "default_namespace".to_string(),
      cache_dir,
    })
  }

  pub fn as_obj(&self, root: &str) -> PersistentCacheConfigObj {
    match self {
      PersistentCacheConfig::Bool(true) => Self::get_default_config(root).as_obj(root),
      PersistentCacheConfig::Bool(false) => {
        panic!("should not call as_obj when PersistentCacheConfig is false")
      }
      PersistentCacheConfig::Obj(obj) => obj.clone(),
    }
  }
}

impl Default for PersistentCacheConfig {
  fn default() -> Self {
    PersistentCacheConfig::Bool(true)
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PersistentCacheConfigObj {
  pub namespace: String,
  pub cache_dir: String,
}
