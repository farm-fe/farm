use std::collections::HashMap;

use common::create_compiler;
use farmfe_testing_helpers::fixture;

use crate::common::assert_compiler_result;

mod common;

#[test]
fn tree_shake_test() {
  fixture!(
    "tests/fixtures/tree_shake/**/index.ts",
    |file, crate_path| {
      let cwd = file.parent().unwrap();
      println!("testing tree shake: {:?}", cwd);

      let compiler = create_compiler(
        HashMap::from([("index".to_string(), "./index.ts".to_string())]),
        cwd.to_path_buf(),
        crate_path,
      );
      compiler.compile().unwrap();

      assert_compiler_result(&compiler);
    }
  );
}
