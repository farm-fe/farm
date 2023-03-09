use std::path::Path;

use base64::engine::{general_purpose, Engine};
use farmfe_core::{config::Config, module::ModuleType, plugin::Plugin};
use farmfe_toolkit::{
  fs::{read_file_raw, read_file_utf8},
  lazy_static::lazy_static,
};

const VIRTUAL_ASSET_PREFIX: &str = "virtual:ASSETS:";

// Default supported static assets: png, jpg, jpeg, gif, svg, webp, mp4, webm, wav, mp3, wma, m4a, aac, ico, ttf, woff, woff2
lazy_static! {
  static ref DEFAULT_STATIC_ASSETS: Vec<&'static str> = vec![
    "png", "jpg", "jpeg", "gif", "svg", "webp", "mp4", "webm", "wav", "mp3", "wma", "m4a", "aac",
    "ico", "ttf", "woff", "woff2",
  ];
}

pub struct FarmPluginStaticAssets {}

impl FarmPluginStaticAssets {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginStaticAssets {
  fn name(&self) -> &str {
    "FarmPluginStaticAssets"
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    let path = Path::new(param.resolved_path);
    let extension = path.extension().and_then(|s| s.to_str());

    if let Some(ext) = extension {
      if DEFAULT_STATIC_ASSETS
        .iter()
        .any(|a| a.eq_ignore_ascii_case(ext))
        || context
          .config
          .assets
          .include
          .iter()
          .any(|a| a.eq_ignore_ascii_case(ext))
      {
        // let file_raw = read_file_raw(param.resolved_path)?;
        // let file_base64 = general_purpose::STANDARD.encode(&file_raw);

        return Ok(Some(farmfe_core::plugin::PluginLoadHookResult {
          content: String::new(), // just return empty string, we don't need to load the file content, we will handle it in the transform hook
          module_type: ModuleType::Asset,
        }));
      }
    }

    Ok(None)
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    if matches!(param.module_type, ModuleType::Asset) {
      if param.query.contains_key("inline") {
        let file_raw = read_file_raw(param.resolved_path)?;
        let file_base64 = general_purpose::STANDARD.encode(&file_raw);
        let path = Path::new(param.resolved_path);
        let ext = path.extension().and_then(|s| s.to_str()).unwrap();

        let content = format!(
          "export default \"data:image/{};base64,{}\"",
          ext, file_base64
        );

        return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
          content,
          module_type: Some(ModuleType::Js),
          source_map: None,
        }));
      } else if param.query.contains_key("raw") {
        let file_utf8 = read_file_utf8(param.resolved_path)?;
        let content = format!("export default \"{}\"", file_utf8);

        return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
          content,
          module_type: Some(ModuleType::Js),
          source_map: None,
        }));
      } else {
        let content = format!(
          "import '{}{}';\nexport default \"{}\"",
          VIRTUAL_ASSET_PREFIX, param.resolved_path, param.resolved_path
        );

        return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
          content,
          module_type: Some(ModuleType::Js),
          source_map: None,
        }));
      }
    }

    Ok(None)
  }
}
