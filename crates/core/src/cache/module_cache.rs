use rkyv::Deserialize;
use std::path::{Path, PathBuf};

use farmfe_macro_cache_item::cache_item;

use crate::config::Mode;
use crate::module::Module;
use crate::plugin::PluginAnalyzeDepsHookResultEntry;
use crate::{deserialize, serialize};

pub const FARM_MODULE_CACHE_VERSION: &str = "0.0.1";

pub struct ModuleCacheManager {
  cache_dir: PathBuf,
}

#[cache_item]
pub struct CachedModule {
  pub module: Module,
  pub deps: Vec<PluginAnalyzeDepsHookResultEntry>,
}

impl ModuleCacheManager {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
    let mut cache_dir = Path::new(cache_dir_str).to_path_buf();
    cache_dir.push(namespace.to_string() + "-" + FARM_MODULE_CACHE_VERSION);

    if matches!(mode, Mode::Development) {
      cache_dir.push("development");
    } else {
      cache_dir.push("production");
    }

    if cache_dir_str.len() > 0 && !cache_dir.exists() {
      std::fs::create_dir_all(&cache_dir).unwrap();
    }

    Self { cache_dir }
  }

  pub fn has_module_cache(&self, code_hash: &str) -> bool {
    let path = self.cache_dir.join(code_hash);
    path.exists()
  }

  pub fn set_module_cache(&self, code_hash: &str, module: &CachedModule) {
    let bytes = serialize!(module);
    let path = self.cache_dir.join(code_hash);
    std::fs::write(path, bytes).unwrap();
  }

  pub fn get_module_cache(&self, code_hash: &str) -> CachedModule {
    let path = self.cache_dir.join(code_hash);
    let bytes = std::fs::read(path).unwrap();
    deserialize!(&bytes, CachedModule)
  }
}
