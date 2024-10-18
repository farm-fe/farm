use std::{collections::HashMap, path::PathBuf};

use farmfe_utils::hash::sha256;
use relative_path::RelativePath;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

  pub fn timestamp_enabled(&self) -> bool {
    match self {
      PersistentCacheConfig::Bool(b) => *b,
      PersistentCacheConfig::Obj(obj) => obj.module_cache_key_strategy.timestamp,
    }
  }

  pub fn hash_enabled(&self) -> bool {
    match self {
      PersistentCacheConfig::Bool(b) => *b,
      PersistentCacheConfig::Obj(obj) => obj.module_cache_key_strategy.hash,
    }
  }

  pub fn get_default_config(root: &str) -> Self {
    let cache_dir = RelativePath::new("node_modules/.farm/cache")
      .to_logical_path(root)
      .to_string_lossy()
      .to_string();

    PersistentCacheConfig::Obj(PersistentCacheConfigObj {
      namespace: "farm-cache".to_string(),
      cache_dir,
      module_cache_key_strategy: PersistentModuleCacheKeyStrategy {
        timestamp: true,
        hash: true,
      },
      // build dependencies are set by node side
      build_dependencies: vec![],
      envs: HashMap::new(),
    })
  }

  pub fn as_raw_object(&self) -> &PersistentCacheConfigObj {
    if let PersistentCacheConfig::Obj(obj) = self {
      obj
    } else {
      panic!("PersistentCacheConfig is not a object");
    }
  }

  pub fn as_obj(&self, root: &str) -> PersistentCacheConfigObj {
    match self {
      PersistentCacheConfig::Bool(true) => Self::get_default_config(root).as_obj(root),
      PersistentCacheConfig::Bool(false) => {
        panic!("should not call as_obj when PersistentCacheConfig is false")
      }
      PersistentCacheConfig::Obj(obj) => {
        let mut cloned_obj = obj.clone();
        let default_config = Self::get_default_config(root);

        if cloned_obj.cache_dir.is_empty() {
          cloned_obj
            .cache_dir
            .clone_from(&default_config.as_raw_object().cache_dir);
        }

        let mut keys: Vec<_> = cloned_obj.envs.keys().collect();
        keys.sort();
        let config_str = keys
            .into_iter()
            .map(|k| {
                let v = cloned_obj.envs.get(k).unwrap();
                format!("{k}={v}")
            })
            .collect::<Vec<_>>()
            .join("&");
        let config_hash = sha256(config_str.as_bytes(), 32);

        cloned_obj.build_dependencies.push(config_hash);

        if !cloned_obj.build_dependencies.is_empty() {
          cloned_obj.build_dependencies.sort();

          let mut content = String::new();

          for dep in &cloned_obj.build_dependencies {
            if !PathBuf::from(dep).exists()
              || !PathBuf::from(dep).is_file()
              || dep.ends_with(".farm")
            {
              content.push_str(dep);
            } else {
              let c = std::fs::read_to_string(dep).unwrap();
              content.push_str(&c);
            }
          }

          let hash = sha256(content.as_bytes(), 32);
          let mut cache_dir = PathBuf::from(&cloned_obj.cache_dir);
          cache_dir.push(hash);
          cloned_obj.cache_dir = cache_dir.to_string_lossy().to_string();
        }

        if cloned_obj.namespace.is_empty() {
          cloned_obj
            .namespace
            .clone_from(&default_config.as_raw_object().namespace);
        }

        cloned_obj
      }
    }
  }
}

impl Default for PersistentCacheConfig {
  fn default() -> Self {
    PersistentCacheConfig::Bool(true)
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PersistentCacheConfigObj {
  pub namespace: String,
  pub cache_dir: String,
  pub module_cache_key_strategy: PersistentModuleCacheKeyStrategy,
  /// If the build dependencies changed, the cache need to be invalidated. The value must be absolute path.
  /// It's absolute paths of farm.config by default. Farm will use their timestamp and hash to invalidate cache.
  /// Note that farm will resolve the config file dependencies from node side
  pub build_dependencies: Vec<String>,
  pub envs: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct PersistentModuleCacheKeyStrategy {
  pub timestamp: bool,
  pub hash: bool,
}

impl Default for PersistentModuleCacheKeyStrategy {
  fn default() -> Self {
    Self {
      timestamp: true,
      hash: true,
    }
  }
}
