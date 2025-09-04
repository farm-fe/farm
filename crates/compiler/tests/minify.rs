use std::path::PathBuf;

use farmfe_core::{config::bool_or_obj::BoolOrObj, HashMap};
use farmfe_testing_helpers::fixture;

mod common;

use common::{
  assert_compiler_result, assert_compiler_result_with_config, create_compiler,
  create_compiler_with_args, AssertCompilerResultConfig,
};

#[test]
fn minify_script_test() {
  fixture!(
    "tests/fixtures/minify/script/**/index.ts",
    |file_path_buf: PathBuf, crate_path_buf: PathBuf| {
      use common::{format_output_name, get_dir_config_files, try_merge_config_file};

      let cwd = file_path_buf.parent().unwrap();
      println!("testing test case: {cwd:?}");

      let entry_name = "index".to_string();

      let files = get_dir_config_files(cwd);

      for (name, config_entry) in files {
        let compiler = create_compiler_with_args(
          cwd.to_path_buf(),
          crate_path_buf.clone(),
          |mut config, plugins| {
            config.input = HashMap::from_iter(vec![(
              entry_name.clone(),
              file_path_buf.to_string_lossy().to_string().clone(),
            )]);
            config.minify = Box::new(BoolOrObj::Bool(true));
            config.resolve.auto_external_failed_resolve = true;
            config.concatenate_modules = true;

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
  );
}

#[test]
fn minify_css_test() {
  fixture!(
    "tests/fixtures/minify/css/**/index.ts",
    |file, crate_path| {
      let cwd = file.parent().unwrap();
      println!("testing minify: {cwd:?}");

      let entry_name = "index".to_string();
      let compiler = create_compiler(
        HashMap::from_iter([(entry_name.clone(), "./index.ts".to_string())]),
        cwd.to_path_buf(),
        crate_path,
        true,
      );
      compiler.compile().unwrap();

      assert_compiler_result(&compiler, Some(&entry_name));
    }
  );
}

#[test]
fn minify_html_test() {
  fixture!(
    "tests/fixtures/minify/html/**/index.html",
    |file, crate_path| {
      let cwd = file.parent().unwrap();
      println!("testing minify: {cwd:?}");

      let entry_name = "index".to_string();
      let compiler = create_compiler(
        HashMap::from_iter([(entry_name.clone(), "./index.html".to_string())]),
        cwd.to_path_buf(),
        crate_path,
        true,
      );
      compiler.compile().unwrap();

      assert_compiler_result(&compiler, Some(&entry_name));
    }
  );
}
