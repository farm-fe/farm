use farmfe_compiler::Compiler;
use farmfe_core::{
  config::{Config, RuntimeConfig},
  relative_path::RelativePath,
};

fn main() {
  let relative_root = RelativePath::new("examples/script_entry");
  let cwd = std::env::current_dir().unwrap();
  let react_examples_root = relative_root.to_logical_path(cwd.clone());

  let compiler = Compiler::new(
    Config {
      root: react_examples_root.to_string_lossy().to_string(),
      runtime: Box::new(RuntimeConfig {
        path: cwd
          .join("packages")
          .join("runtime")
          .join("src")
          .join("index.ts")
          .to_string_lossy()
          .to_string(),
        plugins: vec![],
        swc_helpers_path: cwd
          .join("packages")
          .join("core")
          .join("node_modules")
          .join("@swc")
          .join("helpers")
          .read_link()
          .unwrap()
          .to_string_lossy()
          .to_string(),
        ..Default::default()
      }),
      ..Default::default()
    },
    vec![],
  )
  .unwrap();

  compiler.compile().unwrap();
}
