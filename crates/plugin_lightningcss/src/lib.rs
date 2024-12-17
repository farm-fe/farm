use std::sync::Arc;

use farmfe_core::config::css::NameConversion;
use farmfe_core::config::custom::get_config_css_modules_local_conversion;
use farmfe_core::module::{ModuleId, ModuleType};
use farmfe_core::plugin::{PluginLoadHookResult, PluginTransformHookResult};
use farmfe_core::HashMap;
use farmfe_core::{
  config::Config, context::CompilationContext, deserialize, parking_lot::Mutex, plugin::Plugin,
};
use farmfe_macro_cache_item::cache_item;
use farmfe_toolkit::common::load_source_original_source_map;
use farmfe_toolkit::fs::read_file_utf8;
use farmfe_toolkit::lazy_static::lazy_static;
use farmfe_toolkit::regex::Regex;
use farmfe_toolkit::script::module_type_from_id;
use farmfe_utils::stringify_query;
use lightningcss::stylesheet::StyleSheet;
use rkyv::Deserialize;
mod parse;
pub const FARM_CSS_MODULES: &str = "farm_lightning_css_modules";

lazy_static! {
  pub static ref FARM_CSS_MODULES_SUFFIX: Regex =
    Regex::new(&format!("(?:\\?|&){FARM_CSS_MODULES}")).unwrap();
}

fn is_farm_css_modules(path: &str) -> bool {
  FARM_CSS_MODULES_SUFFIX.is_match(path)
}

fn is_farm_css_modules_type(module_type: &ModuleType) -> bool {
  if let ModuleType::Custom(c) = module_type {
    return c.as_str() == FARM_CSS_MODULES;
  }

  false
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
  locals_conversion: NameConversion,
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

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    if is_farm_css_modules(&param.module_id) {
      return Ok(Some(PluginLoadHookResult {
        content: self
          .content_map
          .lock()
          .get(&param.module_id)
          .unwrap()
          .clone(),
        module_type: ModuleType::Custom(FARM_CSS_MODULES.to_string()),
        source_map: None,
      }));
    }
    let module_type = module_type_from_id(param.resolved_path);

    if let Some(module_type) = module_type {
      if matches!(module_type, ModuleType::Css) {
        let content = read_file_utf8((param.resolved_path))?;

        let map =
          load_source_original_source_map(&content, param.resolved_path, "/*# sourceMappingURL");
        return Ok(Some(PluginLoadHookResult {
          content,
          module_type,
          source_map: map,
        }));
      }
    }

    Ok(None)
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<PluginTransformHookResult>> {
    if is_farm_css_modules_type(&param.module_type) {
      return Ok(Some(PluginTransformHookResult {
        content: param.content.clone(),
        module_type: Some(ModuleType::Css),
        source_map: self.sourcemap_map.lock().get(&param.module_id).cloned(),
        ignore_previous_source_map: false,
      }));
    }
    if matches!(param.module_type, ModuleType::Css) {
      let enable_css_modules = context.config.css.modules.is_some();

      if enable_css_modules && self.is_path_match_css_modules(&param.module_id) {}

      let mut query = param.query.clone();
      query.push((FARM_CSS_MODULES.to_string(), "".to_string()));
      let query_string = stringify_query(&query);

      let css_modules_module_id =
        ModuleId::new(param.resolved_path, &query_string, &context.config.root);

      let mut dynamic_import_of_composes:HashMap<String, String> = HashMap::default();

      let ast = StyleSheet::parse(&param.content, Default::default()).unwrap();
      let stylesheet = ast.to_css(Default::default()).unwrap();

      let mut export_names = Vec::new();

      if let Some(exports) = stylesheet.exports {
        for export in exports {
          let (name, exports) = export;
          export_names.push((
            self.locals_conversion.transform(&name),
            exports.name.clone(),
          ));
        }
      }

      export_names.sort_by_key(|e| e.0.to_string());

      let code = format!(
        r#"
  import "{}";
  {}
  export default {{{}}}
  "#,
        css_modules_module_id.to_string(),
        dynamic_import_of_composes
          .into_iter()
          .fold(Vec::new(), |mut acc, (from, name)| {
            acc.push(format!("import {name} from \"{from}\""));
            acc
          })
          .join(";\n"),
        export_names
          .iter()
          .map(|(name, classes)| format!("\"{}\": `{}`", name, classes.trim()))
          .collect::<Vec<String>>()
          .join(",")
      );

      return Ok(Some(PluginTransformHookResult {
        content: code,
        module_type: Some(ModuleType::Js),
        source_map: None,
        ignore_previous_source_map: true,
      }));
    }

    Ok(None)
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
      locals_conversion: get_config_css_modules_local_conversion(config),
    }
  }

  pub fn is_path_match_css_modules(&self, path: &str) -> bool {
    self
      .css_modules_paths
      .iter()
      .any(|regex| regex.is_match(path))
  }
}
