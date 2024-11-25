use std::{collections::HashMap, path::PathBuf};

use farmfe_core::config::{bool_or_obj::BoolOrObj, config_regex::ConfigRegex, Mode, TargetEnv};
mod common;
use crate::common::{
  assert_compiler_result_with_config, create_compiler_with_args, AssertCompilerResultConfig,
};

#[allow(dead_code)]
#[cfg(test)]
fn test(file: String, crate_path: String) {
  use std::fs;

  use common::get_config_field;
  use farmfe_core::config::partial_bundling::PartialBundlingEnforceResourceConfig;

  use crate::common::try_read_config_from_json;

  let file_path_buf = PathBuf::from(file.clone());
  let create_path_buf = PathBuf::from(crate_path);
  let cwd = file_path_buf.parent().unwrap();
  println!("testing test case: {cwd:?}");

  let entry_name = "index".to_string();

  let runtime_entry = cwd.to_path_buf().join("runtime.ts");
  let mut files = fs::read_dir(cwd)
    .unwrap()
    .filter_map(|item| item.map(Some).unwrap_or(None))
    .filter_map(|item| {
      let filename = item
        .path()
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

      if filename.starts_with("config") || filename.ends_with(".json") {
        Some((
          item.path(),
          filename
            .trim_start_matches("config")
            .trim_start_matches('.')
            .trim_end_matches("json")
            .trim_end_matches('.')
            .to_string(),
        ))
      } else {
        None
      }
    })
    .collect::<Vec<_>>();

  if files.is_empty() {
    files.push((cwd.join("config.json"), "".to_string()));
  }

  for (config_entry, config_named) in files {
    let config_from_file = try_read_config_from_json(config_entry);

    let compiler = create_compiler_with_args(
      cwd.to_path_buf(),
      create_path_buf.clone(),
      |mut config, plugins| {
        config.mode = Mode::Production;

        if runtime_entry.is_file() {
          let runtime_entry = runtime_entry.to_string_lossy().to_string();
          config.runtime.path = runtime_entry;
        }

        config.input = HashMap::from_iter(vec![(entry_name.clone(), file.clone())]);

        config.minify = Box::new(BoolOrObj::Bool(false));
        config.tree_shaking = Box::new(BoolOrObj::Bool(false));

        config.external = vec![ConfigRegex::new("(^node:.*)"), ConfigRegex::new("^fs$")];
        config.output.target_env = TargetEnv::Custom("library-browser".to_string());
        // config.output.format = ModuleFormat::CommonJs;

        // TODO: multiple bundle
        config.partial_bundling.enforce_resources = vec![PartialBundlingEnforceResourceConfig {
          test: vec![ConfigRegex::new(".+")],
          name: "index".to_string(),
        }];

        if let Some(config_from_file) = config_from_file {
          if let Some(mode) = get_config_field(&config_from_file, &["mode"]) {
            config.mode = mode;
          }

          if let Some(format) = get_config_field(&config_from_file, &["output", "format"]) {
            config.output.format = format;
          }

          if let Some(target_env) = get_config_field(&config_from_file, &["output", "targetEnv"]) {
            config.output.target_env = target_env;
          }
        }

        (config, plugins)
      },
    );

    compiler.compile().unwrap();

    assert_compiler_result_with_config(
      &compiler,
      AssertCompilerResultConfig {
        entry_name: Some(entry_name.clone()),
        ignore_emitted_field: false,
        output_file: Some(format!(
          "output.{}js",
          if config_named.is_empty() {
            "".into()
          } else {
            format!("{config_named}.")
          }
        )),
      },
    );
  }
}

farmfe_testing::testing! {"tests/fixtures/bundle/library/**/index.ts", test}
// farmfe_testing::testing! {"tests/fixtures/bundle/library/hybrid/esm_export_cjs/**/index.ts", test}
