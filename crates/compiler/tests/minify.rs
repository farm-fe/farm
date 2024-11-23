use farmfe_core::HashMap;
use farmfe_testing_helpers::fixture;

mod common;

use common::{assert_compiler_result, create_compiler};

#[test]
fn minify_script_test() {
  fixture!(
    "tests/fixtures/minify/script/**/index.ts",
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
