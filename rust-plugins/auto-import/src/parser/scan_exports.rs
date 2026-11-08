use std::{
  fs::{metadata, read_to_string},
  path::Path,
};

use farmfe_toolkit::plugin_utils::normalize_path::normalize_path;

use super::parse::{DeclarationType, ESMExport, ExportType, parse_esm_exports};

const FILE_EXTENSION_LOOKUP: [&str; 8] =
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
    s.split(['-', '_'])
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

pub fn scan_exports(
  file_path: &str,
  content: Option<&str>,
) -> farmfe_core::error::Result<Vec<Import>> {
  let exports = parse_esm_exports(Some(file_path), content)?;
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
          let index_path = resolve_export_path(&specifier_path)?;
          let index_content = read_to_string(&index_path).map_err(|error| {
            farmfe_core::error::CompilationError::LoadError {
              resolved_path: index_path.clone(),
              source: Some(Box::new(error)),
            }
          })?;
          exports_names.extend(scan_exports(&index_path, Some(&index_content))?);
        }
      }
      _ => {
        // do nothing
      }
    }
  }
  Ok(exports_names)
}

fn resolve_export_path(specifier_path: &str) -> farmfe_core::error::Result<String> {
  use std::io::{Error, ErrorKind};
  let load_error = |path: &str, error: Error| farmfe_core::error::CompilationError::LoadError {
    resolved_path: path.to_owned(),
    source: Some(Box::new(error)),
  };
  let directory = match metadata(specifier_path) {
    Ok(meta) if meta.is_file() => return Ok(specifier_path.to_owned()),
    Ok(meta) => meta.is_dir(),
    Err(error) if error.kind() == ErrorKind::NotFound => false,
    Err(error) => return Err(load_error(specifier_path, error)),
  };
  for extension in FILE_EXTENSION_LOOKUP {
    let candidate = if directory {
      format!("{specifier_path}/index{extension}")
    } else {
      format!("{specifier_path}{extension}")
    };
    match metadata(&candidate) {
      Ok(meta) if meta.is_file() => return Ok(candidate),
      Ok(_) => {}
      Err(error) if error.kind() == ErrorKind::NotFound => {}
      Err(error) => return Err(load_error(&candidate, error)),
    }
  }
  Err(load_error(
    specifier_path,
    Error::new(ErrorKind::NotFound, "Re-export source not found"),
  ))
}

#[cfg(test)]
mod export_path_tests {
  use super::*;
  use std::sync::atomic::{AtomicUsize, Ordering};
  static NEXT: AtomicUsize = AtomicUsize::new(0);

  #[test]
  fn extensionless_directory_and_retry_keep_errors_visible() {
    let root = std::env::temp_dir().join(format!(
      "farm-export-path-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).unwrap();
    let entry = root.join("entry.ts");
    std::fs::write(&entry, "export * from './foo';").unwrap();
    std::fs::write(root.join("foo.ts"), "export const first = 1;").unwrap();
    let exports = scan_exports(entry.to_str().unwrap(), None).unwrap();
    assert!(exports.iter().any(|item| item.name == "first"));
    std::fs::write(root.join("foo.ts"), "export const second = 2;").unwrap();
    let exports = scan_exports(entry.to_str().unwrap(), None).unwrap();
    assert!(exports.iter().any(|item| item.name == "second"));
    assert!(!exports.iter().any(|item| item.name == "first"));
    std::fs::remove_file(root.join("foo.ts")).unwrap();
    assert!(scan_exports(entry.to_str().unwrap(), None).is_err());
    std::fs::create_dir(root.join("foo")).unwrap();
    std::fs::write(root.join("foo/index.ts"), "export const nested = 3;").unwrap();
    assert!(
      scan_exports(entry.to_str().unwrap(), None)
        .unwrap()
        .iter()
        .any(|item| item.name == "nested")
    );
    // A file used as a directory produces an I/O error, never an empty success.
    assert!(resolve_export_path(entry.join("child").to_str().unwrap()).is_err());
    std::fs::remove_dir_all(root).unwrap();
  }
}
