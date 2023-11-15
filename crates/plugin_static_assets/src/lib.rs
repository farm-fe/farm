#![feature(path_file_prefix)]

use std::{path::Path, sync::Arc};

use base64::engine::{general_purpose, Engine};
use farmfe_core::{
  cache_item,
  config::Config,
  context::{CompilationContext, EmitFileParams},
  deserialize,
  module::ModuleType,
  // plugin::{constants::PLUGIN_BUILD_STAGE_META_RESOLVE_KIND, Plugin, ResolveKind},
  plugin::{Plugin, PluginResolveHookResult},
  relative_path::RelativePath,
  resource::{Resource, ResourceOrigin, ResourceType},
  rkyv::Deserialize,
  serialize,
};
use farmfe_toolkit::{
  fs::{read_file_raw, read_file_utf8, transform_output_filename},
  lazy_static::lazy_static,
};

// Default supported static assets: png, jpg, jpeg, gif, svg, webp, mp4, webm, wav, mp3, wma, m4a, aac, ico, ttf, woff, woff2
lazy_static! {
  static ref DEFAULT_STATIC_ASSETS: Vec<&'static str> = vec![
    "png", "jpg", "jpeg", "gif", "svg", "webp", "mp4", "webm", "wav", "mp3", "wma", "m4a", "aac",
    "ico", "ttf", "woff", "woff2",
  ];
}

const PLUGIN_NAME: &str = "FarmPluginStaticAssets";
const PUBLIC_ASSET_PREFIX: &str = "virtual:__FARM_PUBLIC_ASSET__:";

pub struct FarmPluginStaticAssets {}

impl FarmPluginStaticAssets {
  pub fn new(_: &Config) -> Self {
    Self {}
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

    if let Some(source) = param.resolved_path.strip_prefix(PUBLIC_ASSET_PREFIX) {
      return Ok(Some(farmfe_core::plugin::PluginLoadHookResult {
        content: format!("export default '{source}';"),
        module_type: ModuleType::Js,
      }));
    } else if let Some(ext) = extension {
      if self.is_asset(ext, context) {
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
      // let resolve_kind = ResolveKind::from(
      //   param
      //     .meta
      //     .get(PLUGIN_BUILD_STAGE_META_RESOLVE_KIND)
      //     .unwrap()
      //     .as_str(),
      // );

      if param.query.iter().any(|(k, _)| k == "inline") {
        let file_raw = read_file_raw(param.resolved_path)?;
        let file_base64 = general_purpose::STANDARD.encode(file_raw);
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
          ignore_previous_source_map: false,
        }));
      } else if param.query.iter().any(|(k, _)| k == "raw") {
        let file_utf8 = read_file_utf8(param.resolved_path)?;
        let content = format!("export default `{}`", file_utf8);

        return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
          content,
          module_type: Some(ModuleType::Js),
          source_map: None,
          ignore_previous_source_map: false,
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
        let content = if !context.config.output.public_path.is_empty() {
          let normalized_public_path = context
            .config
            .output
            .public_path
            .trim_start_matches("/")
            .trim_end_matches("/");

          if normalized_public_path.is_empty() {
            format!("export default \"/{}\"", resource_name)
          } else {
            format!(
              "export default \"/{}/{}\"",
              normalized_public_path, resource_name
            )
          }
        } else {
          format!("export default \"/{}\"", resource_name)
        };
        // TODO refactor this to support cache
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
      if let ResourceOrigin::Module(_) = resource.origin {
        list.push(resource.clone());
      }
    }

    let cached_static_assets = CachedStaticAssets { list };

    Ok(Some(serialize!(&cached_static_assets)))
  }
}

#[cache_item]
struct CachedStaticAssets {
  list: Vec<Resource>,
}
