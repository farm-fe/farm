use std::path::PathBuf;

use common::{test_builder, TestBuilderOptions};

mod common;

fn script_test(file: String, crate_path: String) {
  let file_path_buf = PathBuf::from(file.clone());
  let cwd = file_path_buf.parent().unwrap();

  println!("testing test case: {cwd:?}");

  test_builder(TestBuilderOptions::new(file, PathBuf::from(crate_path)));
}

// farmfe_testing::testing!("tests/fixtures/script/**/index.ts", script_test);
