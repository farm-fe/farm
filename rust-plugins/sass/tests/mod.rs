use std::{collections::HashMap, sync::Arc};

use farmfe_compiler::Compiler;
use farmfe_core::config::{Config, OutputConfig, RuntimeConfig};
use farmfe_testing_helpers::fixture;

#[test]
fn test() {
  fixture!("tests/fixtures/index.scss", |file, cwd| {
    // TODO
    println!("【 file 】==> {:?}", file);
  });
}
