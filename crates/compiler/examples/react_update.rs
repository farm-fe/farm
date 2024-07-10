use farmfe_compiler::Compiler;
use farmfe_core::{
  config::{bool_or_obj::BoolOrObj, Config, RuntimeConfig},
  plugin::UpdateType,
  relative_path::RelativePath,
};

fn main() {
  let relative_root = RelativePath::new("examples/css-modules");
  let cwd = std::env::current_dir().unwrap();
  let react_examples_root = relative_root.to_logical_path(cwd.clone());
  let linked_swc_helper_path = cwd
    .join("packages")
    .join("core")
    .join("node_modules")
    .join("@swc")
    .join("helpers");
  let relative_swc_helpers_path = linked_swc_helper_path.read_link().unwrap();
  let swc_helpers_path = RelativePath::new(relative_swc_helpers_path.to_str().unwrap())
    .to_logical_path(linked_swc_helper_path.parent().unwrap())
    .to_string_lossy()
    .to_string();

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
        swc_helpers_path,
        ..Default::default()
      }),
      tree_shaking: Box::new(BoolOrObj::Bool(false)),
      ..Default::default()
    },
    vec![],
  )
  .unwrap();

  compiler.compile().unwrap();

  compiler
    .update(
      vec![(
        cwd
          .join("examples")
          .join("css-modules")
          .join("src")
          .join("main.module.css")
          .to_string_lossy()
          .to_string(),
        UpdateType::Updated,
      )],
      || {},
      false,
      true,
    )
    .unwrap();
}
