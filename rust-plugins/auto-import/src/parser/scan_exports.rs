use std::{
  fs::{metadata, read_to_string},
  path::Path,
};

use farmfe_toolkit::plugin_utils::normalize_path::normalize_path;

use super::parse::{parse_esm_exports, DeclarationType, ESMExport, ExportType};

const FILE_EXTENSION_LOOKUP: [&'static str; 8] =
  [".mts", ".cts", ".ts", ".mjs", ".cjs", ".js", ".jsx", ".tsx"];

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Import {
  pub from: String,
  pub name: String,
  pub priority: usize,
  pub export_type: ExportType,
  pub tp: Option<bool>,
  pub disabled: Option<bool>,
  pub dts_disabled: Option<bool>,
  pub declaration_type: Option<DeclarationType>,
  pub type_from: Option<String>,
  pub as_name: Option<String>,
}

impl Import {
  pub fn stringify_import(&self) -> String {
    match self.export_type {
      ExportType::DefaultDecl => format!("import {} from '{}';\n", self.name, self.from),
      ExportType::Declaration | ExportType::Named => {
        format!("import {{ {} }} from '{}';\n", self.name, self.from)
      }
      ExportType::Namespace => {
        if let Some(as_name) = &self.as_name {
          if as_name == &self.name {
            format!("import {{ {} }} from '{}';\n", self.name, self.from)
          } else {
            format!(
              "import {{ {} as {} }} from '{}';\n",
              self.name, as_name, self.from
            )
          }
        } else {
          format!("import {{ {} }} from '{}';\n", self.name, self.from)
        }
      }
      ExportType::Type => format!("import {{ type {} }} from '{}';\n", self.name, self.from),
      _ => String::new(),
    }
  }
}

fn to_pascal_case(s: &str) -> String {
  if s.contains('-') || s.contains('_') {
    s.split(|c| c == '-' || c == '_')
      .filter(|part| !part.is_empty())
      .map(|part| {
        let mut chars = part.chars();
        chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str()
      })
      .collect()
  } else {
    let chars = s.chars();
    chars.as_str().to_string()
  }
}

fn get_filename_by_path(file_path: &str) -> String {
  let path = Path::new(file_path);
  let filename = path
    .file_stem()
    .and_then(|filename_osstr| filename_osstr.to_str())
    .map(|filename_str| filename_str.to_owned())
    .unwrap();
  to_pascal_case(&filename)
}

pub fn scan_exports(file_path: &str, content: Option<&str>) -> Vec<Import> {
  let exports = parse_esm_exports(Some(file_path), content);
  let filename = get_filename_by_path(file_path);
  let mut exports_names = Vec::new();
  for export in exports {
    let ESMExport {
      name,
      named_exports,
      export_type,
      type_named_exports,
      specifier,
      ..
    } = export;
    match export_type {
      ExportType::Type => {
        if let Some(type_named_export) = type_named_exports {
          for (_k, v) in type_named_export {
            exports_names.push(Import {
              from: file_path.to_string(),
              tp: Some(true),
              export_type: export_type.clone(),
              name: v,
              priority: 0,
              disabled: None,
              dts_disabled: None,
              declaration_type: None,
              type_from: None,
              as_name: None,
            });
          }
        }
      }
      ExportType::DefaultDecl => {
        exports_names.push(Import {
          from: file_path.to_string(),
          name: filename.clone(),
          export_type,
          priority: 0,
          disabled: None,
          dts_disabled: None,
          declaration_type: None,
          tp: None,
          type_from: None,
          as_name: None,
        });
      }
      ExportType::Declaration => {
        exports_names.push(Import {
          from: file_path.to_string(),
          name: name.unwrap(),
          export_type,
          priority: 0,
          disabled: None,
          dts_disabled: None,
          declaration_type: None,
          tp: None,
          type_from: None,
          as_name: None,
        });
      }
      ExportType::Namespace => exports_names.push(Import {
        from: file_path.to_string(),
        name: name.clone().unwrap(),
        export_type,
        priority: 0,
        disabled: None,
        dts_disabled: None,
        declaration_type: None,
        tp: None,
        type_from: None,
        as_name: name,
      }),
      ExportType::Named => {
        if let Some(named_export) = named_exports {
          for (_k, v) in named_export {
            exports_names.push(Import {
              from: file_path.to_string(),
              name: v,
              export_type: export_type.clone(),
              priority: 0,
              disabled: None,
              dts_disabled: None,
              declaration_type: None,
              tp: None,
              type_from: None,
              as_name: None,
            });
          }
        }
      }
      ExportType::All => {
        // file_path is a file , need to get the file parent dir path
        let file_path = Path::new(file_path);
        if let Some(parent_dir_path) = file_path.parent() {
          let specifier_path = parent_dir_path.join(specifier.unwrap());
          let specifier_path = normalize_path(specifier_path.to_str().unwrap());
          let file_exts = FILE_EXTENSION_LOOKUP.to_vec();
          if metadata(&specifier_path).unwrap().is_dir() {
            // check if specifier_path has index.tsx ...
            for ext in &file_exts {
              let index_path = format!("{}/index{}", specifier_path, ext);
              if metadata(&index_path).is_ok() {
                let index_content = read_to_string(&index_path).unwrap();
                let index_exports = scan_exports(&index_path, Some(&index_content));
                exports_names.extend(index_exports);
                break;
              }
            }
          } else {
            // check if specifier_path is a file
            for ext in &file_exts {
              let index_path = format!("{}{}", specifier_path, ext);
              if metadata(&index_path).is_ok() {
                let index_content = read_to_string(&index_path).unwrap();
                let index_exports = scan_exports(&index_path, Some(&index_content));
                exports_names.extend(index_exports);
                break;
              }
            }
          }
        }
      }
      _ => {
        // do nothing
      }
    }
  }
  exports_names
}
