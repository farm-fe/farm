use std::path::PathBuf;

use farmfe_core::config::comments::CommentsConfig;
use farmfe_core::config::{bool_or_obj::BoolOrObj, config_regex::ConfigRegex, Mode, TargetEnv};
use farmfe_core::HashMap;

mod common;
use crate::common::{
  assert_compiler_result_with_config, create_compiler_with_args, generate_runtime,
  AssertCompilerResultConfig,
};

fn normalize_path(path: &str) -> String {
  if cfg!(windows) {
    path.replace('\\', "/").into()
  } else {
    path.to_string()
  }
}

#[allow(dead_code)]
fn test(file_path_buf: PathBuf, crate_path_buf: PathBuf) {
  use common::{format_output_name, get_dir_config_files, try_merge_config_file};

  let cwd = file_path_buf.parent().unwrap();
  println!("testing test case: {cwd:?}");

  let entry_name = "index".to_string();
  let files = get_dir_config_files(cwd);
  let test_cases_that_need_real_runtime = vec![
    "cjs/",
    "hybrid/",
    "dynamic/lodash_export",
    "external/import_namespace",
    "library/external/",
    "dynamic/require",
    "reexport/use_external_reexport",
    "formats/umd",
  ];
  let is_cjs = test_cases_that_need_real_runtime
    .iter()
    .any(|i| normalize_path(&cwd.to_string_lossy()).contains(i));

  for (name, config_entry) in files {
    let compiler = create_compiler_with_args(
      cwd.to_path_buf(),
      crate_path_buf.clone(),
      |mut config, plugins| {
        config.mode = Mode::Development;
        config.input = HashMap::from_iter(vec![(
          entry_name.clone(),
          file_path_buf.to_string_lossy().to_string().clone(),
        )]);
        config.minify = Box::new(BoolOrObj::Bool(false));

        if is_cjs {
          config.tree_shaking = Box::new(BoolOrObj::Bool(true));
          config.comments = Box::new(CommentsConfig::Bool(true));
        } else {
          config.tree_shaking = Box::new(BoolOrObj::Bool(false));
        }

        config.external = vec![
          ConfigRegex::new("(^node:.*)"),
          ConfigRegex::new("^fs$"),
          ConfigRegex::new("/external/.+"),
        ];
        config.output.target_env = TargetEnv::Library;
        // config.resolve.auto_external_failed_resolve = true;

        if is_cjs {
          config.runtime = generate_runtime(crate_path_buf.clone(), true);
        }

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

#[test]
fn library_test() {
  use farmfe_testing_helpers::fixture;

  fixture!("tests/fixtures/library/**/index.ts", test);

  // fixture!("tests/fixtures/library/cjs/basic/**/index.ts", test);
  // fixture!(
  //   "tests/fixtures/library/cjs/require/external/**/index.ts",
  //   test
  // );
  // fixture!("tests/fixtures/library/hybrid/normal/**/index.ts", test);
  // fixture!(
  //   "tests/fixtures/library/external/deep-export-all/**/index.ts",
  //   test
  // );
  // fixture!(
  //   "tests/fixtures/library/external/conflicts/**/index.ts",
  //   test
  // );
  // fixture!(
  //   "tests/fixtures/library/reexport/reexport_hybrid_cjs/default/**/index.ts",
  //   test
  // );
  // fixture!(
  //   "tests/fixtures/library/reexport/use_external_reexport/**/index.ts",
  //   test
  // );
  // fixture!("tests/fixtures/library/cyclic/**/index.ts", test);

  // fixture!(
  //   "tests/fixtures/library/formats/umd/rollup_repl/**/index.ts",
  //   test
  // );
}

// farmfe_testing::testing! {
//   "tests/fixtures/library/external/conflicts/**/index.ts",
//   test
// }
