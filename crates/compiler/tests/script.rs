use farmfe_core::HashMap;
use std::path::PathBuf;

use common::{
  assert_compiler_result_with_config, create_compiler_with_args, try_merge_config_file,
  AssertCompilerResultConfig,
};

mod common;

fn script_test(file: String, crate_path: String) {
  let file_path_buf = PathBuf::from(file.clone());
  let create_path_buf = PathBuf::from(crate_path);
  let cwd = file_path_buf.parent().unwrap();
  println!("testing test case: {cwd:?}");

  let entry_name = "index".to_string();

  let config_entry = cwd.to_path_buf().join("config.json");

  let compiler =
    create_compiler_with_args(cwd.to_path_buf(), create_path_buf, |mut config, plugins| {
      config.input = HashMap::from_iter(vec![(entry_name.clone(), file.clone())]);

      config = try_merge_config_file(config, config_entry);

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
