#![feature(path_file_prefix)]

use std::{collections::HashMap, path::Path};

use base64::engine::{general_purpose, Engine};
use farmfe_core::{
  config::Config,
  module::{ModuleId, ModuleType},
  plugin::{
    constants::PLUGIN_BUILD_STAGE_META_RESOLVE_KIND, Plugin, PluginHookContext, ResolveKind,
  },
  resource::{Resource, ResourceOrigin, ResourceType},
};
use farmfe_toolkit::{
  fs::{read_file_raw, read_file_utf8, transform_output_filename},
  lazy_static::lazy_static,
};
use farmfe_utils::stringify_query;

// Default supported static assets: png, jpg, jpeg, gif, svg, webp, mp4, webm, wav, mp3, wma, m4a, aac, ico, ttf, woff, woff2
lazy_static! {
  static ref DEFAULT_STATIC_ASSETS: Vec<&'static str> = vec![
    "png", "jpg", "jpeg", "gif", "svg", "webp", "mp4", "webm", "wav", "mp3", "wma", "m4a", "aac",
    "ico", "ttf", "woff", "woff2",
  ];
}

const PLUGIN_NAME: &str = "FarmPluginStaticAssets";

pub struct FarmPluginStaticAssets {}

impl FarmPluginStaticAssets {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginStaticAssets {
  fn name(&self) -> &str {
    PLUGIN_NAME
  }
  /// Make sure this plugin is executed last
  fn priority(&self) -> i32 {
    99
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
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    if matches!(param.module_type, ModuleType::Asset) {
      let resolve_kind = ResolveKind::from(
        param
          .meta
          .get(PLUGIN_BUILD_STAGE_META_RESOLVE_KIND)
          .unwrap()
          .as_str(),
      );

      if param.query.iter().find(|(k, _)| k == "inline").is_some() {
        let file_raw = read_file_raw(param.resolved_path)?;
        let file_base64 = general_purpose::STANDARD.encode(&file_raw);
        let path = Path::new(param.resolved_path);
        let ext = path.extension().and_then(|s| s.to_str()).unwrap();
        // TODO: recognize MIME type
        let content = format!(
          "export default \"data:image/{};base64,{}\"",
          ext, file_base64
        );

        return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
          content,
          module_type: Some(ModuleType::Js),
          source_map: None,
        }));
      } else if param.query.iter().find(|(k, _)| k == "raw").is_some() {
        let file_utf8 = read_file_utf8(param.resolved_path)?;
        let content = format!("export default \"{}\"", file_utf8);

        return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
          content,
          module_type: Some(ModuleType::Js),
          source_map: None,
        }));
      } else {
        let filename = Path::new(param.resolved_path)
          .file_prefix()
          .and_then(|s| s.to_str())
          .unwrap();
        let bytes = read_file_raw(param.resolved_path)?;
        let ext = Path::new(param.resolved_path)
          .extension()
          .and_then(|s| s.to_str())
          .unwrap();

        let resource_name = transform_output_filename(
          context.config.output.assets_filename.clone(),
          filename,
          &bytes,
          ext,
        );
        let content = format!("export default \"/{}\"", resource_name);

        let mut resources_map = context.resources_map.lock();
        resources_map.insert(
          resource_name.clone(),
          Resource {
            name: resource_name,
            bytes,
            emitted: false,
            resource_type: ResourceType::Asset(ext.to_string()),
            origin: ResourceOrigin::Module(ModuleId::new(
              param.resolved_path,
              &stringify_query(&param.query),
              &context.config.root,
            )),
            preserve_name: false,
          },
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
