#![deny(clippy::all)]

use base64::{Engine, engine::general_purpose};
use farmfe_core::{
  config::{Config, config_regex::ConfigRegex},
  context::CompilationContext,
  module::ModuleType,
  plugin::{Plugin, PluginLoadHookResult, PluginTransformHookResult},
  serde_json,
};

use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{fs::read_file_raw, plugin_utils::path_filter::PathFilter};
use mime_guess::{from_path, mime::IMAGE};
use std::{fs::metadata, path::Path};

#[farm_plugin]
pub struct FarmfePluginUrl {
  options: Options,
}

#[derive(Debug, serde::Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Options {
  pub limit: Option<u64>,
  pub public_path: Option<String>,
  pub emit_files: Option<bool>,
  pub filename: Option<String>,
  pub dest_dir: Option<String>,
  pub source_dir: Option<String>,
  pub include: Option<Vec<ConfigRegex>>,
  pub exclude: Option<Vec<ConfigRegex>>,
}

pub fn get_file_size(file_path: &str) -> u64 {
  match metadata(file_path) {
    Ok(metadata) => metadata.len(),
    Err(_e) => 0,
  }
}

fn to_data_url(resolved_path: &str, raw_bytes: &[u8]) -> String {
  let mime_type = from_path(resolved_path).first_or_octet_stream();
  let file_base64 = general_purpose::STANDARD.encode(raw_bytes);
  format!("data:{mime_type};base64,{file_base64}")
}

impl FarmfePluginUrl {
  fn new(_config: &Config, options: String) -> Self {
    let mut options: Options = serde_json::from_str(&options).unwrap();
    let include = [
      r".*\.svg$",
      r".*\.png$",
      r".*\.jp(e)?g$",
      r".*\.gif$",
      r".*\.webp$",
    ]
    .map(ConfigRegex::new)
    .to_vec();
    if options.include.is_none() {
      options.include = Some(include);
    }
    Self { options }
  }
}

impl Plugin for FarmfePluginUrl {
  fn name(&self) -> &str {
    "FarmfePluginUrl"
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    let options: Options = self.options.clone();
    let include = options.include.unwrap_or_default();
    let exclude = options.exclude.unwrap_or_default();

    let filter = PathFilter::new(&include, &exclude);
    if !filter.execute(&param.module_id) || param.query.iter().any(|(k, _)| k == "url") {
      return Ok(None);
    }

    Ok(Some(PluginLoadHookResult {
      content: String::new(),
      module_type: ModuleType::Asset,
      source_map: None,
    }))
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &std::sync::Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<PluginTransformHookResult>> {
    let options: Options = self.options.clone();
    let include = options.include.unwrap_or_default();
    let exclude = options.exclude.unwrap_or_default();
    let filter = PathFilter::new(&include, &exclude);

    if !filter.execute(&param.module_id) || !matches!(param.module_type, ModuleType::Asset) {
      return Ok(None);
    }

    let raw_bytes = read_file_raw(param.resolved_path).unwrap_or_default();
    let limit = options.limit.unwrap_or(14 * 1024);
    let mut res = String::new();

    if get_file_size(param.resolved_path) > limit {
      if self.options.emit_files.unwrap_or(true) {
        use farmfe_core::{context::EmitFileParams, resource::ResourceType};
        let filename = Path::new(param.resolved_path)
          .file_name()
          .and_then(|v| v.to_str())
          .unwrap_or_default();
        let ext = Path::new(param.resolved_path)
          .extension()
          .and_then(|e| e.to_str())
          .unwrap_or("asset")
          .to_string();
        context.emit_file(EmitFileParams {
          resolved_path: param.module_id.to_string(),
          name: filename.to_string(),
          content: raw_bytes.clone(),
          resource_type: ResourceType::Asset(ext),
        });
        let public_path = context.config.output.public_path.clone();
        return Ok(Some(PluginTransformHookResult {
          content: format!("export default \"{public_path}{filename}\""),
          module_type: Some(ModuleType::Js),
          source_map: None,
          ignore_previous_source_map: false,
        }));
      } else {
        res = to_data_url(param.resolved_path, &raw_bytes);
      }
    } else {
      let mime_type = from_path(param.resolved_path).first_or_octet_stream();
      if mime_type.type_() == IMAGE {
        res = to_data_url(param.resolved_path, &raw_bytes);
      }
    }

    Ok(Some(PluginTransformHookResult {
      content: format!("export default \"{res}\""),
      module_type: Some(ModuleType::Js),
      source_map: None,
      ignore_previous_source_map: false,
    }))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use farmfe_core::{config::persistent_cache::PersistentCacheConfig, resource::ResourceOrigin};
  use std::sync::Arc;

  #[test]
  fn emitted_asset_keeps_query_qualified_module_owner() {
    let file = std::env::temp_dir().join(format!("farm-url-owner-{}.svg", std::process::id()));
    let bytes = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><path d=\"M0 0\"/></svg>";
    std::fs::write(&file, bytes).unwrap();
    let resolved_path = file.to_string_lossy().to_string();
    let module_id = format!("{resolved_path}?media");
    let context = Arc::new(
      CompilationContext::new(
        Config {
          persistent_cache: Box::new(PersistentCacheConfig::Bool(false)),
          ..Default::default()
        },
        vec![],
      )
      .unwrap(),
    );
    let plugin = FarmfePluginUrl::new(&context.config, r#"{"limit":0,"include":[".*"]}"#.into());
    let result = plugin
      .transform(
        &farmfe_core::plugin::PluginTransformHookParam {
          module_id: module_id.clone(),
          content: String::new(),
          module_type: ModuleType::Asset,
          resolved_path: &resolved_path,
          query: vec![("media".into(), String::new())],
          meta: Default::default(),
          source_map_chain: vec![],
        },
        &context,
      )
      .unwrap()
      .unwrap();
    let resources = context.resources_map.lock();
    let resource = resources
      .get(file.file_name().unwrap().to_str().unwrap())
      .unwrap();
    assert_eq!(resource.bytes, bytes);
    match &resource.origin {
      ResourceOrigin::Module(owner) => assert_eq!(*owner, context.str_to_module_id(&module_id)),
      _ => panic!("asset must belong to its importing module"),
    }
    assert!(
      result
        .content
        .contains(file.file_name().unwrap().to_str().unwrap())
    );
    std::fs::remove_file(file).unwrap();
  }
}
