#![feature(path_file_prefix)]
#![deny(clippy::all)]

use std::{
  collections::HashMap,
  fs::{copy, create_dir_all},
  io,
  path::Path,
  sync::{Arc, Mutex},
};

use base64::{engine::general_purpose, Engine};
use farmfe_core::{
  config::{config_regex::ConfigRegex, Config, Mode},
  context::EmitFileParams,
  module::ModuleType,
  plugin::{Plugin, PluginLoadHookResult},
  resource::ResourceType,
  serde_json,
};

use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{
  fs::{TransformOutputFileNameParams, read_file_raw, transform_output_filename},
  plugin_utils::path_filter::PathFilter,
};
use farmfe_utils::relative;
use mime_guess::{from_path, mime::IMAGE};
use std::fs::metadata;

fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
  if let Some(parent) = dst.as_ref().parent() {
    create_dir_all(parent)?;
  }
  copy(src, dst)?;
  Ok(())
}

#[farm_plugin]
pub struct FarmfePluginUrl {
  options: Options,
  copies: Arc<Mutex<HashMap<String, String>>>,
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
    .map(|o| ConfigRegex::new(o))
    .to_vec();
    if options.include.is_none() {
      options.include = Some(include);
    }
    let copies = Arc::new(Mutex::new(HashMap::new()));
    Self { options, copies }
  }
}

impl Plugin for FarmfePluginUrl {
  fn name(&self) -> &str {
    "FarmfePluginUrl"
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    let options: Options = self.options.clone();
    let include = options.include.unwrap_or(vec![]);
    let exclude = options.exclude.unwrap_or(vec![]);

    let filter = PathFilter::new(&include, &exclude);
    if !filter.execute(&param.module_id) {
      return Ok(None);
    }
    let mut res = String::new();
    let limit = options.limit.unwrap_or(14 * 1024);
    let raw_bytes = read_file_raw(param.resolved_path).unwrap_or(vec![]);
    let public_path = options.public_path.unwrap_or("".to_string());
    if get_file_size(param.resolved_path) > limit {
      let file_path = Path::new(param.resolved_path);
      let ext: &str = file_path.extension().and_then(|s| s.to_str()).unwrap();
      let filename = file_path.file_prefix().and_then(|s| s.to_str()).unwrap();
      let mut filename_config = options.filename.unwrap_or("[hash].[ext]".to_string());
      let relative_dir = {
        let dir_name = Path::new(param.resolved_path)
          .parent()
          .and_then(|p| p.file_name())
          .unwrap_or_else(|| std::ffi::OsStr::new(""))
          .to_string_lossy()
          .into_owned();
        if let Some(source_dir) = options.source_dir {
          format!("./{}", relative(&source_dir, &dir_name))
        } else {
          dir_name
        }
      };

      if filename_config.contains("[dirname]") {
        filename_config = filename_config.replace("[dirname]", &relative_dir);
      }
      let transform_output_file_name_params = TransformOutputFileNameParams {
        filename_config,
        name: filename,
        name_hash: "",
        bytes: &raw_bytes,
        ext,
        special_placeholders: &Default::default()
      };
      let output_file_name = transform_output_filename(transform_output_file_name_params);
      res = format!("{}{}", &public_path, &output_file_name);
      let content = read_file_raw(param.resolved_path).unwrap();
      context.emit_file(EmitFileParams {
        resolved_path: param.module_id.clone(),
        name: res.clone(),
        content,
        resource_type: ResourceType::Asset(ext.to_string()),
      });
      {
        let mut copies = self.copies.lock().unwrap();
        copies.insert(param.resolved_path.to_owned(), output_file_name);
      }
    } else {
      let mime_type = from_path(&param.resolved_path).first_or_octet_stream();
      if mime_type.type_() == IMAGE {
        let file_base64 = general_purpose::STANDARD.encode(raw_bytes);
        res = format!("data:{};base64,{}", mime_type.to_string(), file_base64);
      }
    }
    Ok(Some(PluginLoadHookResult {
      content: format!("export default \"{}\"", res),
      module_type: ModuleType::Js,
      source_map: None,
    }))
  }

  fn finalize_resources(
    &self,
    _param: &mut farmfe_core::plugin::PluginFinalizeResourcesHookParam,
    context: &Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if matches!(context.config.mode, Mode::Production) {
      if self.options.emit_files.unwrap_or(false) {
        let copies = self.copies.lock().unwrap();
        let dest_dir = &self.options.dest_dir.clone().unwrap_or("".to_string());
        let base_dir = Path::new(dest_dir);
        for (key, value) in copies.iter() {
          let base_dir = base_dir.join(Path::new(value));
          let _ = copy_file(key, base_dir);
        }
      }
    }
    Ok(None)
  }
}
