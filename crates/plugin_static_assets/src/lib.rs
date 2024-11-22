#![feature(path_file_prefix)]

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::Arc,
};

use base64::engine::{general_purpose, Engine};
use farmfe_core::{
  cache_item,
  config::{asset::AssetFormatMode, custom::get_config_assets_mode, Config},
  context::{CompilationContext, EmitFileParams},
  deserialize,
  module::ModuleType,
  plugin::{Plugin, PluginResolveHookResult},
  relative_path::RelativePath,
  resource::{Resource, ResourceOrigin, ResourceType},
  rkyv::Deserialize,
  serialize,
  swc_common::sync::OnceCell,
};
use farmfe_toolkit::{
  fs::{read_file_raw, read_file_utf8, transform_output_filename},
  lazy_static::lazy_static,
};
use farmfe_utils::{hash::sha256, stringify_query, FARM_IGNORE_ACTION_COMMENT};

// Default supported static assets: png, jpg, jpeg, gif, svg, webp, mp4, webm, wav, mp3, wma, m4a, aac, ico, ttf, woff, woff2
lazy_static! {
  static ref DEFAULT_STATIC_ASSETS: Vec<&'static str> = vec![
    "png", "jpg", "jpeg", "gif", "svg", "webp", "mp4", "webm", "wav", "mp3", "wma", "m4a", "aac",
    "ico", "ttf", "woff", "woff2", "txt", "eot"
  ];
}

const PLUGIN_NAME: &str = "FarmPluginStaticAssets";
const PUBLIC_ASSET_PREFIX: &str = "virtual:__FARM_PUBLIC_ASSET__:";

fn is_asset_query(query: &Vec<(String, String)>) -> bool {
  let query_map = query.iter().cloned().collect::<HashMap<_, _>>();

  query_map.contains_key("raw") || query_map.contains_key("inline") || query_map.contains_key("url")
}

pub struct FarmPluginStaticAssets {
  asset_format_mode: OnceCell<AssetFormatMode>,
}

impl FarmPluginStaticAssets {
  pub fn new(_: &Config) -> Self {
    Self {
      asset_format_mode: OnceCell::new(),
    }
  }

  fn is_asset(&self, ext: &str, context: &Arc<CompilationContext>) -> bool {
    DEFAULT_STATIC_ASSETS
      .iter()
      .any(|a| a.eq_ignore_ascii_case(ext))
      || context
        .config
        .assets
        .include
        .iter()
        .any(|a| a.eq_ignore_ascii_case(ext))
  }

  fn get_resource_name(name: &str, module_id: &str) -> String {
    let last_dot = name.rfind('.').unwrap_or(0);
    if last_dot == 0 {
      format!("{}-{}", name, sha256(module_id.as_bytes(), 6))
    } else {
      format!(
        "{}-{}{}",
        &name[..last_dot],
        sha256(module_id.to_string().as_bytes(), 6),
        &name[last_dot..]
      )
    }
  }
}

impl Plugin for FarmPluginStaticAssets {
  fn name(&self) -> &str {
    PLUGIN_NAME
  }
  /// Make sure this plugin is executed last
  fn priority(&self) -> i32 {
    -99
  }

  fn resolve(
    &self,
    param: &farmfe_core::plugin::PluginResolveHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginResolveHookResult>> {
    let path = Path::new(&param.source);
    let extension = path.extension().and_then(|s| s.to_str());

    if let Some(ext) = extension {
      if self.is_asset(ext, context)
        && context.config.assets.public_dir.is_some()
        && param.source.starts_with('/')
      {
        let public_dir = context.config.assets.public_dir.as_ref().unwrap();
        let resolved_public_path =
          RelativePath::new(&param.source[1..]).to_logical_path(public_dir);

        if resolved_public_path.exists() {
          return Ok(Some(PluginResolveHookResult {
            resolved_path: format!("{PUBLIC_ASSET_PREFIX}{}", param.source),
            external: false,
            side_effects: false,
            ..Default::default()
          }));
        }
      }
      // TODO make css imported asset only handled by static assets to avoid issue: background: url(xx.svg),
      // and xx.svg are transformed to js module by plugin like svgr. which would lead to a error for css.
    }

    Ok(None)
  }
  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    let path = Path::new(param.resolved_path);
    let extension = path.extension().and_then(|s| s.to_str());
    let public_path = context.config.output.public_path.clone();

    if let Some(source) = param.resolved_path.strip_prefix(PUBLIC_ASSET_PREFIX) {
      // fix https://github.com/farm-fe/farm/issues/1165
      let mut base_path = PathBuf::from(public_path);
      base_path.push(source.trim_start_matches("/"));

      let base_path_str = base_path.to_string_lossy().to_string();
      return Ok(Some(farmfe_core::plugin::PluginLoadHookResult {
        content: format!("export default '{base_path_str}';"),
        module_type: ModuleType::Js,
        source_map: None,
      }));
    } else if let Some(ext) = extension {
      if self.is_asset(ext, context) {
        return Ok(Some(farmfe_core::plugin::PluginLoadHookResult {
          content: String::new(), // just return empty string, we don't need to load the file content, we will handle it in the transform hook
          module_type: ModuleType::Asset,
          source_map: None,
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
      if param.query.iter().any(|(k, _)| k == "inline") {
        let file_base64 = if param.content.is_empty() {
          general_purpose::STANDARD.encode(read_file_raw(param.resolved_path)?)
        } else {
          param.content.clone()
        };
        let path = Path::new(param.resolved_path);
        let ext = path.extension().and_then(|s| s.to_str()).unwrap();
        let mime_type = mime_guess::from_ext(ext).first_or_octet_stream();
        let mime_type_str = mime_type.to_string();

        let content = format!("export default \"data:{mime_type_str};base64,{file_base64}\"");

        return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
          content,
          module_type: Some(ModuleType::Js),
          source_map: None,
          ignore_previous_source_map: false,
        }));
      } else if param.query.iter().any(|(k, _)| k == "raw") {
        let file_utf8 = if param.content.is_empty() {
          read_file_utf8(param.resolved_path)?
        } else {
          param.content.clone()
        };
        let content = format!("export default {:?}", file_utf8.replace("\r\n", "\n"));

        return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
          content,
          module_type: Some(ModuleType::Js),
          source_map: None,
          ignore_previous_source_map: false,
        }));
      } else {
        let bytes = if param.content.is_empty() {
          read_file_raw(param.resolved_path)?
        } else {
          // if content is not empty, it means the content is already read by the load hook in other plugins
          return Ok(None);
        };

        let ext = Path::new(param.resolved_path)
          .extension()
          .and_then(|s| s.to_str())
          .unwrap();

        let filename = Path::new(param.resolved_path)
          .file_prefix()
          .and_then(|s| s.to_str())
          .unwrap();
        let resource_name = transform_output_filename(
          context.config.output.assets_filename.clone(),
          filename,
          &bytes,
          ext,
        ) + stringify_query(&param.query).as_str();

        let resource_name = Self::get_resource_name(&resource_name, &param.module_id);

        let assets_path = if !context.config.output.public_path.is_empty() {
          let normalized_public_path = context.config.output.public_path.trim_end_matches("/");

          format!("{normalized_public_path}/{resource_name}")
        } else {
          format!("/{resource_name}")
        };

        let mode = self.asset_format_mode.get_or_init(|| {
          get_config_assets_mode(&context.config)
            .unwrap_or_else(|| (context.config.output.target_env.clone().into()))
        });

        let content = match mode {
          AssetFormatMode::Node => {
            format!(
              r#"
    import {{ fileURLToPath }} from "node:url";
    export default fileURLToPath(new URL(/* {FARM_IGNORE_ACTION_COMMENT} */{assets_path:?}, import.meta.url))
                "#
            )
          }
          AssetFormatMode::Browser => {
            format!("export default {assets_path:?};")
          }
        };

        context.emit_file(EmitFileParams {
          resolved_path: param.module_id.clone(),
          name: resource_name,
          content: bytes,
          resource_type: ResourceType::Asset(ext.to_string()),
        });

        return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
          content,
          module_type: Some(ModuleType::Js),
          source_map: None,
          ignore_previous_source_map: false,
        }));
      }
    }

    Ok(None)
  }

  fn plugin_cache_loaded(
    &self,
    cache: &Vec<u8>,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let cached_static_assets = deserialize!(cache, CachedStaticAssets);

    for asset in cached_static_assets.list {
      if let ResourceOrigin::Module(m) = asset.origin {
        context.emit_file(EmitFileParams {
          resolved_path: m.to_string(),
          name: asset.name,
          content: asset.bytes,
          resource_type: asset.resource_type,
        });
      }
    }

    Ok(Some(()))
  }

  fn write_plugin_cache(
    &self,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<Vec<u8>>> {
    let mut list = vec![];
    let resources_map = context.resources_map.lock();

    for (_, resource) in resources_map.iter() {
      if let ResourceOrigin::Module(m) = &resource.origin {
        if context.cache_manager.module_cache.has_cache(m) {
          list.push(resource.clone());
        }
      }
    }

    if !list.is_empty() {
      let cached_static_assets = CachedStaticAssets { list };

      Ok(Some(serialize!(&cached_static_assets)))
    } else {
      Ok(None)
    }
  }
}

#[cache_item(farmfe_core)]
struct CachedStaticAssets {
  list: Vec<Resource>,
}

pub struct FarmPluginRaw {}

impl FarmPluginRaw {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginRaw {
  fn name(&self) -> &str {
    "FARM_PLUGIN_RAW"
  }
  /// Make sure this plugin is executed last
  fn priority(&self) -> i32 {
    101
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    if is_asset_query(&param.query) {
      return Ok(Some(farmfe_core::plugin::PluginLoadHookResult {
        content: String::new(), // just return empty string, we don't need to load the file content, we will handle it in the transform hook
        module_type: ModuleType::Asset,
        source_map: None,
      }));
    }

    Ok(None)
  }
}
