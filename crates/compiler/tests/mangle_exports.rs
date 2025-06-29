use std::path::PathBuf;
use std::sync::Arc;

use farmfe_core::config::{bool_or_obj::BoolOrObj, config_regex::ConfigRegex, Mode, TargetEnv};
use farmfe_core::HashMap;

mod common;
use crate::common::{
  assert_compiler_result_with_config, create_compiler_with_args, generate_runtime,
  AssertCompilerResultConfig,
};

#[allow(dead_code)]
fn test(file_path_buf: PathBuf, crate_path_buf: PathBuf) {
  use common::{format_output_name, get_dir_config_files, try_merge_config_file};

  let cwd = file_path_buf.parent().unwrap();
  println!("testing test case: {cwd:?}");

  let entry_name = "index".to_string();

  let files = get_dir_config_files(cwd);

  for (name, config_entry) in files {
    let compiler = create_compiler_with_args(
      cwd.to_path_buf(),
      crate_path_buf.clone(),
      |mut config, mut plugins| {
        config.mode = Mode::Production;
        config.input = HashMap::from_iter(vec![(
          entry_name.clone(),
          file_path_buf.to_string_lossy().to_string().clone(),
        )]);
        config.concatenate_modules = false;
        config.minify = Box::new(BoolOrObj::Bool(false));
        config.tree_shaking = Box::new(BoolOrObj::Bool(false));
        config.external = vec![ConfigRegex::new("(^node:.*)")];
        config.output.target_env = TargetEnv::Browser;
        config.resolve.auto_external_failed_resolve = true;
        config.output.show_file_size = false;

        config = try_merge_config_file(config, config_entry);

        if config.output.target_env.is_library() {
          config.runtime = generate_runtime(crate_path_buf.clone(), true);
        }

        // push mangle_exports plugin
        plugins.push(
          Arc::new(farmfe_plugin_mangle_exports::FarmPluginMangleExports::new(
            &config,
          )) as _,
        );

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

#[test]
fn mangle_exports_test() {
  use farmfe_testing_helpers::fixture;

  fixture!("tests/fixtures/mangle_exports/**/index.ts", test);
  // fixture!(
  //   "tests/fixtures/mangle_exports/import-default/**/index.ts",
  //   test
  // );
}
