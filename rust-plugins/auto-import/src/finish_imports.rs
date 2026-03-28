use std::sync::{Arc, Mutex};

use farmfe_core::config::config_regex::ConfigRegex;

use crate::parser::generate_dts::{generate_dts, GenerateDtsOption};
use crate::parser::scan_dirs_exports::scan_dirs_exports;
use crate::parser::scan_exports::Import;
use crate::presets::{resolve_presets, PresetItem};
use crate::Dts;

pub struct FinishImportsParams<'a> {
  pub root_path: String,
  pub presets: Vec<PresetItem>,
  pub dirs: Vec<ConfigRegex>,
  pub dts: Dts,
  pub ignore: Vec<ConfigRegex>,
  pub context_imports: &'a Arc<Mutex<Vec<Import>>>,
}

fn maybe_has_new_or_removed_imports(
  old_imports: &Vec<Import>,
  local_imports: &Vec<Import>,
  resolvers_imports: &Vec<Import>,
) -> bool {
  let old_len = old_imports.len();
  let new_len = local_imports.len() + resolvers_imports.len();
  old_len != new_len
    || local_imports
      .iter()
      .any(|import| !old_imports.contains(import))
    || resolvers_imports
      .iter()
      .any(|import| !old_imports.contains(import))
}

pub fn finish_imports(params: FinishImportsParams) {
  let FinishImportsParams {
    root_path,
    presets,
    dirs,
    dts,
    ignore,
    context_imports,
  } = params;

  let mut presets_imports = resolve_presets(&presets)
    .into_iter()
    .filter(|import| !ignore.iter().any(|ignore| ignore.is_match(&import.name)))
    .collect::<Vec<_>>();
  let local_imports = if dirs.len() > 0 {
    scan_dirs_exports(&root_path, &dirs.clone())
      .into_iter()
      .filter(|import| !ignore.iter().any(|ignore| ignore.is_match(&import.name)))
      .collect::<Vec<_>>()
  } else {
    vec![]
  };
  let mut context_imports_guard = match context_imports.lock() {
    Ok(guard) => guard,
    Err(poisoned) => poisoned.into_inner(),
  };
  let has_new_or_removed_imports =
    maybe_has_new_or_removed_imports(&context_imports_guard, &local_imports, &presets_imports);
  let filename = match dts {
    Dts::Filename(filename) => filename,
    Dts::Bool(b) => {
      if b {
        "auto_import.d.ts".to_string()
      } else {
        "".to_string()
      }
    }
  };

  if has_new_or_removed_imports && !filename.is_empty() {
    let generate_dts_option = GenerateDtsOption {
      filename,
      root_path,
      imports: &local_imports.iter().collect::<Vec<_>>(),
      presets_imports: &presets_imports.iter().collect::<Vec<_>>(),
    };
    generate_dts(generate_dts_option);
  }
  if has_new_or_removed_imports {
    presets_imports.extend(local_imports);
    let mut new_imports = vec![];
    for import in presets_imports {
      if !new_imports.iter().any(|i: &Import| i.name == import.name) {
        new_imports.push(import);
      }
    }
    *context_imports_guard = new_imports;
  }
}
