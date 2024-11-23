use std::path::PathBuf;

use farmfe_core::config::{bool_or_obj::BoolOrObj, config_regex::ConfigRegex, Mode, TargetEnv};
use farmfe_core::HashMap;
mod common;
use crate::common::{
  assert_compiler_result_with_config, create_compiler_with_args, AssertCompilerResultConfig,
};

#[allow(dead_code)]
#[cfg(test)]
fn test(file: String, crate_path: String) {
  use common::{format_output_name, get_dir_config_files, try_merge_config_file};

  let file_path_buf = PathBuf::from(file.clone());
  let create_path_buf = PathBuf::from(crate_path);
  let cwd = file_path_buf.parent().unwrap();
  println!("testing test case: {cwd:?}");

  let entry_name = "index".to_string();

  let files = get_dir_config_files(cwd);

  for (name, config_entry) in files {
    let runtime_entry = cwd.to_path_buf().join("runtime.ts");

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
        config.output.target_env = TargetEnv::Node;

        config = try_merge_config_file(config, config_entry);

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

farmfe_testing::testing! {"tests/fixtures/runtime/bundle/**/index.ts", test}
// farmfe_testing::testing! {"tests/fixtures/runtime/bundle/cjs/export/entryExportStar/**/index.ts", test}
// farmfe_testing::testing! {"tests/fixtures/runtime/bundle/external/import/namespace/**/index.ts", test}
