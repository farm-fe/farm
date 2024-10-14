use std::collections::HashMap;

use farmfe_testing_helpers::fixture;

use crate::common::{assert_compiler_result, create_compiler};

mod common;

#[test]
fn partial_bundling_test() {
  fixture!(
    "tests/fixtures/partial_bundling/**/index.ts",
    |file, crate_path| {
      let cwd = file.parent().unwrap();
      println!("testing tree shake: {cwd:?}");

      let entry_name = "index".to_string();
      let compiler = create_compiler(
        HashMap::from([(entry_name.clone(), "./index.ts".to_string())]),
        cwd.to_path_buf(),
        crate_path,
        false,
      );
      compiler.compile().unwrap();

      assert_compiler_result(&compiler, Some(&entry_name));
    }
  );
}
