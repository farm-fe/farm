use std::{collections::HashMap, path::PathBuf};

use common::{
  assert_compiler_result_with_config, create_compiler_with_args, get_config_field,
  try_read_config_from_json, AssertCompilerResultConfig,
};

mod common;

fn script_test(file: String, crate_path: String) {
  let file_path_buf = PathBuf::from(file.clone());
  let create_path_buf = PathBuf::from(crate_path);
  let cwd = file_path_buf.parent().unwrap();
  println!("testing test case: {cwd:?}");

  let entry_name = "index".to_string();

  let config_entry = cwd.to_path_buf().join("config.json");

  let config_from_file = try_read_config_from_json(config_entry);

  let compiler =
    create_compiler_with_args(cwd.to_path_buf(), create_path_buf, |mut config, plugins| {
      config.input = HashMap::from_iter(vec![(entry_name.clone(), file)]);

      if let Some(config_form_file) = config_from_file {
        if let Some(str) = get_config_field(&config_form_file, &["output", "publicPath"]) {
          config.output.public_path = str;
        }

        if let Some(enable) =
          get_config_field(&config_form_file, &["script", "nativeTopLevelAwait"])
        {
          config.script.native_top_level_await = enable;
        }
      }

      (config, plugins)
    });

  compiler.compile().unwrap();

  assert_compiler_result_with_config(
    &compiler,
    AssertCompilerResultConfig {
      entry_name: Some(entry_name),
      ignore_emitted_field: false,
      ..Default::default()
    },
  );
}

farmfe_testing::testing!("tests/fixtures/script/**/index.ts", script_test);
