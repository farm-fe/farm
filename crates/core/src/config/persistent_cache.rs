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

  pub fn get_default_config(root: &str, config_file_path: &str) -> Self {
    let cache_dir = RelativePath::new("node_modules/.farm/cache")
      .to_logical_path(&root)
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
      build_dependencies: vec![config_file_path.to_string()],
    })
  }

  pub fn as_obj(&self, root: &str, config_file_path: &str) -> PersistentCacheConfigObj {
    match self {
      PersistentCacheConfig::Bool(true) => {
        Self::get_default_config(root, config_file_path).as_obj(root, config_file_path)
      }
      PersistentCacheConfig::Bool(false) => {
        panic!("should not call as_obj when PersistentCacheConfig is false")
      }
      PersistentCacheConfig::Obj(obj) => {
        let mut cloned_obj = obj.clone();
        let config_file_path = config_file_path.to_string();

        if !cloned_obj.build_dependencies.contains(&config_file_path) {
          cloned_obj.build_dependencies.push(config_file_path);
        }

        return cloned_obj;
      }
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
  pub module_cache_key_strategy: PersistentModuleCacheKeyStrategy,
  /// If the build dependencies changed, the cache need to be invalidated. The value must be absolute path.
  /// It's absolute paths of farm.config by default. Farm will use their timestamp and hash to invalidate cache.
  /// Note that farm will resolve the config file dependencies from node side
  pub build_dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PersistentModuleCacheKeyStrategy {
  pub timestamp: bool,
  pub hash: bool,
}
