use std::{collections::HashMap, path::PathBuf};

use farmfe_core::config::{
  bool_or_obj::BoolOrObj, config_regex::ConfigRegex,
  partial_bundling::PartialBundlingEnforceResourceConfig, Mode, TargetEnv,
};
mod common;
use crate::common::{
  assert_compiler_result_with_config, create_compiler_with_args, AssertCompilerResultConfig,
};
use farmfe_core::config::Config;

#[allow(dead_code)]
#[cfg(test)]
fn test(file: String, crate_path: String, f: Option<impl Fn(&mut Config)>) {
  use common::{format_output_name, get_config_field, get_dir_config_files};
  use farmfe_core::config::partial_bundling::PartialBundlingEnforceResourceConfig;

  use crate::common::try_read_config_from_json;

  let file_path_buf = PathBuf::from(file.clone());
  let create_path_buf = PathBuf::from(crate_path);
  let cwd = file_path_buf.parent().unwrap();
  println!("testing test case: {cwd:?}");

  let entry_name = "index".to_string();

  let files = get_dir_config_files(cwd);
  let runtime_entry = cwd.to_path_buf().join("runtime.ts");

  for (name, config_entry) in files {
    // let config_entry = cwd.to_path_buf().join("config.json");

    let config_from_file = try_read_config_from_json(config_entry);

    let compiler = create_compiler_with_args(
      cwd.to_path_buf(),
      create_path_buf.clone(),
      |mut config, plugins| {
        config.mode = Mode::Development;

        if runtime_entry.is_file() {
          let runtime_entry = runtime_entry.to_string_lossy().to_string();
          config.runtime.path = runtime_entry;
        }

        config.input = HashMap::from_iter(vec![(entry_name.clone(), file.clone())]);

        config.minify = Box::new(BoolOrObj::Bool(false));
        config.tree_shaking = Box::new(BoolOrObj::Bool(false));

        config.external = vec![ConfigRegex::new("(^node:.*)"), ConfigRegex::new("^fs$")];
        config.output.target_env = TargetEnv::Custom("library-node".to_string());
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

        if let Some(f) = f.as_ref() {
          f(&mut config);
        }

        (config, plugins)
      },
    );

    compiler.compile().unwrap();

    assert_compiler_result_with_config(
      &compiler,
      AssertCompilerResultConfig {
        entry_name: Some(entry_name.clone()),
        output_file: Some(format_output_name(name)),
        ignore_emitted_field: false,
        ..Default::default()
      },
    );
  }
}

fn single_bundle_test(file: String, crate_path: String) {
  test(file, crate_path, None::<fn(&mut Config)>);
}

fn multiple_bundle_test(file: String, crate_path: String) {
  test(
    file,
    crate_path,
    Some(|config: &mut Config| {
      config.partial_bundling.enforce_resources = vec![PartialBundlingEnforceResourceConfig {
        name: "bundle1".to_string(),
        test: vec![ConfigRegex::new("^bundle2.+")],
      }];
    }),
  );
}

// farmfe_testing::testing! {"tests/fixtures/bundle/library/reexport/reexport_hybrid_cjs/namespace/**/index.ts", single_bundle_test}

farmfe_testing::testing! {
  "tests/fixtures/bundle/**/index.ts",
  multiple_bundle_test
}
// farmfe_testing::testing! {"tests/fixtures/runtime/bundle/cjs/export/entryExportStar/**/index.ts", test}
// farmfe_testing::testing! {"tests/fixtures/runtime/bundle/external/import/namespace/**/index.ts", test}
