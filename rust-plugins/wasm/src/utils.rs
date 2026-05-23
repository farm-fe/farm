use farmfe_core::error::{CompilationError, Result};
use std::collections::HashMap;
use std::fs;
use wasmparser::{Parser, Payload};

#[derive(Debug)]
pub struct WasmInfo {
  pub imports: Vec<ImportInfo>,
  pub exports: Vec<String>,
}

#[derive(Debug)]
pub struct ImportInfo {
  pub from: String,
  pub names: Vec<String>,
}

pub fn parse_wasm(wasm_file_path: &str) -> Result<WasmInfo> {
  let wasm_binary = fs::read(wasm_file_path).map_err(|e| CompilationError::LoadError {
    resolved_path: wasm_file_path.to_string(),
    source: Some(Box::new(e)),
  })?;
  let mut imports_map: HashMap<String, Vec<String>> = HashMap::default();
  let mut exports = Vec::new();

  for payload in Parser::new(0).parse_all(&wasm_binary) {
    match payload.map_err(|e| CompilationError::ParseError {
      resolved_path: wasm_file_path.to_string(),
      msg: format!("Failed to parse WASM: {}", e),
    })? {
      Payload::ImportSection(imports) => {
        for import in imports {
          let import = import.map_err(|e| CompilationError::ParseError {
            resolved_path: wasm_file_path.to_string(),
            msg: format!("Failed to read import: {}", e),
          })?;
          imports_map
            .entry(import.module.to_string())
            .or_default()
            .push(import.name.to_string());
        }
      }
      Payload::ExportSection(exports_section) => {
        for export in exports_section {
          let export = export.map_err(|e| CompilationError::ParseError {
            resolved_path: wasm_file_path.to_string(),
            msg: format!("Failed to read export: {}", e),
          })?;
          exports.push(export.name.to_string());
        }
      }
      _ => {}
    }
  }

  let imports = imports_map
    .into_iter()
    .map(|(from, names)| ImportInfo { from, names })
    .collect();

  Ok(WasmInfo { imports, exports })
}

pub fn generate_glue_code(
  wasm_file_path: &str,
  wasm_init_name: &str,
  wasm_url_name: &str,
) -> Result<String> {
  let wasm_info = parse_wasm(wasm_file_path)?;

  let import_statements: Vec<String> = wasm_info
    .imports
    .iter()
    .enumerate()
    .map(|(i, import)| {
      format!(
        "import * as __farm__wasmImport_{} from {:?};",
        i, import.from
      )
    })
    .collect();

  let import_object_entries: Vec<String> = wasm_info
    .imports
    .iter()
    .enumerate()
    .map(|(i, import)| {
      let import_values: Vec<String> = import
        .names
        .iter()
        .map(|name| format!("{}: __farm__wasmImport_{}[{:?}]", name, i, name))
        .collect();
      format!("{:?}: {{ {} }}", import.from, import_values.join(", "))
    })
    .collect();

  let init_code = format!(
    r#"const __farm__wasmModule = await {}({{{}}}, {});
const __farm__wasmExports = __farm__wasmModule.exports;"#,
    wasm_init_name,
    import_object_entries.join(", "),
    wasm_url_name
  );

  let export_statements: Vec<String> = wasm_info
    .exports
    .iter()
    .map(|export| {
      if export == "default" {
        "export default __farm__wasmExports.default;".to_string()
      } else {
        format!("export const {} = __farm__wasmExports.{};", export, export)
      }
    })
    .collect();

  Ok(
    [
      import_statements.join("\n"),
      init_code,
      export_statements.join("\n"),
    ]
    .join("\n"),
  )
}
