#![deny(clippy::all)]

use base64::engine::general_purpose;
use base64::Engine;
use farmfe_core::config::config_regex::ConfigRegex;
use farmfe_core::module::ModuleType;
use farmfe_core::plugin::PluginLoadHookResult;
use farmfe_core::serde_json;
use farmfe_core::{config::Config, plugin::Plugin};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::fs::read_file_raw;
use farmfe_toolkit::plugin_utils::path_filter::PathFilter;
use mime_guess::from_path;
use mime_guess::mime::IMAGE;

#[derive(Debug, serde::Deserialize, Default, Clone)]
pub struct Options {
  dom: Option<bool>,
  pub include: Option<Vec<ConfigRegex>>,
  pub exclude: Option<Vec<ConfigRegex>>,
}

#[farm_plugin]
pub struct FarmfePluginImage {
  options: Options,
}

impl FarmfePluginImage {
  fn new(_config: &Config, options: String) -> Self {
    let options: Options = serde_json::from_str(&options).unwrap();
    Self { options }
  }
}

impl Plugin for FarmfePluginImage {
  fn name(&self) -> &str {
    "FarmfePluginImage"
  }
  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    let options: Options = self.options.clone();
    let include = options.include.unwrap_or(vec![]);
    let exclude = options.exclude.unwrap_or(vec![]);
    let filter = PathFilter::new(&include, &exclude);
    if !filter.execute(&param.module_id) {
      return Ok(None);
    }
    let mime_type = from_path(&param.resolved_path).first_or_octet_stream();
    if mime_type.type_() == IMAGE {
      let dom = options.dom.unwrap_or(false);
      let file_base64 =
        general_purpose::STANDARD.encode(read_file_raw(param.resolved_path).unwrap_or(vec![]));
      let data_uri = format!("data:{};base64,{}", mime_type.to_string(), file_base64);
      let content = if dom {
        format!(
          "var img = new Image();
          img.src = \"{}\";
          export default img;",
          data_uri
        )
      } else {
        format!("export default \"{}\"", data_uri)
      };
      return Ok(Some(PluginLoadHookResult {
        content,
        module_type: ModuleType::Js,
        source_map: None,
      }));
    }
    Ok(None)
  }
}
