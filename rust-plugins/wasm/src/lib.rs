pub mod utils;

use farmfe_core::{
  config::Config,
  context::EmitFileParams,
  error::CompilationError,
  module::ModuleType,
  plugin::{Plugin, PluginLoadHookResult, PluginResolveHookResult},
  resource::ResourceType,
};
use std::{fs, path::Path};
use utils::generate_glue_code;

use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::fs::{transform_output_filename, TransformOutputFileNameParams};

const WASM_HELPER_ID_FARM: &str = "farm/wasm-helper.js";

#[farm_plugin]
pub struct FarmfePluginWasm {}

impl FarmfePluginWasm {
  fn new(_config: &Config, _options: String) -> Self {
    Self {}
  }
}

impl Plugin for FarmfePluginWasm {
  fn name(&self) -> &str {
    "FarmfePluginWasm"
  }
  fn resolve(
    &self,
    param: &farmfe_core::plugin::PluginResolveHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginResolveHookResult>> {
    let id = &param.source;
    if id == WASM_HELPER_ID_FARM {
      return Ok(Some(PluginResolveHookResult {
        resolved_path: id.to_string(),
        ..Default::default()
      }));
    }

    Ok(None)
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    let wasm_file_path = param.resolved_path;
    if wasm_file_path == WASM_HELPER_ID_FARM {
      return Ok(Some(PluginLoadHookResult {
        content: include_str!("wasm_runtime.js").to_string(),
        module_type: ModuleType::Js,
        source_map: None,
      }));
    }

    if wasm_file_path.ends_with(".wasm") {
      let init = param.query.iter().any(|(k, _)| k == "init");
      let content = fs::read(wasm_file_path).map_err(|e| CompilationError::LoadError {
        resolved_path: wasm_file_path.to_string(),
        source: Some(Box::new(e)),
      })?;

      let file_name_ext = Path::new(wasm_file_path)
        .file_name()
        .map(|x| x.to_string_lossy().to_string())
        .unwrap();
      let (file_name, ext) = file_name_ext.split_once('.').unwrap();
      let assets_filename_config = context.config.output.assets_filename.clone();
      let transform_output_file_name_params = TransformOutputFileNameParams {
        filename_config: assets_filename_config,
        name: file_name,
        name_hash: "",
        bytes: &param.module_id.as_bytes(),
        ext,
        special_placeholders: &Default::default(),
      };
      let output_file_name = transform_output_filename(transform_output_file_name_params);
      let params = EmitFileParams {
        name: output_file_name,
        content,
        resource_type: ResourceType::Asset("wasm".to_string()),
        resolved_path: param.module_id.to_string(),
      };
      context.emit_file(params);

      // let wasm_url = if !context.config.output.public_path.is_empty() {
      //   let normalized_public_path = context.config.output.public_path.trim_end_matches('/');
      //   format!("{}/{}", normalized_public_path, resolved_path)
      // } else {
      //   format!("/{}", resolved_path)
      // };

      let content = if init {
        format!(
          r#"import initWasm from "{WASM_HELPER_ID_FARM}";
          import wasmUrl from "{wasm_file_path}?url";
          export default opts => initWasm(opts, wasmUrl)"#,
        )
      } else {
        format!(
          r#"import initWasm from "{WASM_HELPER_ID_FARM}";
          import wasmUrl from "{wasm_file_path}?url";
          {}
          "#,
          generate_glue_code(wasm_file_path, "initWasm", "wasmUrl")?
        )
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
