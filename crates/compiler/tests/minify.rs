use std::collections::HashMap;

use farmfe_testing_helpers::fixture;

mod common;

use common::{assert_compiler_result, create_compiler};

#[test]
fn minify_test() {
  fixture!(
    "tests/fixtures/minify/**/index.ts",
    |file, crate_path| {
      let cwd = file.parent().unwrap();
      println!("testing minify: {:?}", cwd);

      let entry_name = "index".to_string();
      let compiler = create_compiler(
        HashMap::from([(entry_name.clone(), "./index.ts".to_string())]),
        cwd.to_path_buf(),
        crate_path,
      );
      compiler.compile().unwrap();

      assert_compiler_result(&compiler, Some(&entry_name));
    }
  );
}