use std::sync::Arc;

use farmfe_core::HashMap;
use farmfe_core::{
  config::Config, context::CompilationContext, deserialize, parking_lot::Mutex, plugin::Plugin,
};
use farmfe_macro_cache_item::cache_item;
use farmfe_toolkit::lazy_static::lazy_static;
use farmfe_toolkit::regex::Regex;
use rkyv::Deserialize;

pub const FARM_CSS_MODULES: &str = "farm_lightning_css_modules";

lazy_static! {
  pub static ref FARM_CSS_MODULES_SUFFIX: Regex =
    Regex::new(&format!("(?:\\?|&){FARM_CSS_MODULES}")).unwrap();
}

#[cache_item]
struct LightningCssModulesCache {
  content_map: HashMap<String, String>,
  sourcemap_map: HashMap<String, String>,
}

pub struct FarmPluginLightningCss {
  css_modules_paths: Vec<Regex>,
  content_map: Mutex<HashMap<String, String>>,
  sourcemap_map: Mutex<HashMap<String, String>>,
}

impl Plugin for FarmPluginLightningCss {
  fn name(&self) -> &str {
    "FarmPluginLightningCss"
  }

  fn priority(&self) -> i32 {
    -99
  }

  fn plugin_cache_loaded(
    &self,
    cache: &Vec<u8>,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let cache = deserialize!(cache, LightningCssModulesCache);
    let mut content_map = self.content_map.lock();

    for (k, v) in cache.content_map {
      content_map.insert(k, v);
    }

    let mut sourcemap_map = self.sourcemap_map.lock();

    for (k, v) in cache.sourcemap_map {
      sourcemap_map.insert(k, v);
    }

    Ok(Some(()))
  }
}

impl FarmPluginLightningCss {
  pub fn new(config: &Config) -> Self {
    Self {
      css_modules_paths: config
        .css
        .modules
        .as_ref()
        .map(|item| {
          item
            .paths
            .iter()
            .map(|item| Regex::new(item).expect("Config `css.modules.paths` is not valid Regex"))
            .collect()
        })
        .unwrap_or_default(),
      content_map: Mutex::new(Default::default()),
      sourcemap_map: Mutex::new(Default::default()),
    }
  }

  pub fn is_path_match_css_modules(&self, path: &str) -> bool {
    self
      .css_modules_paths
      .iter()
      .any(|regex| regex.is_match(path))
  }
}
