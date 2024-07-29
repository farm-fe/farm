use criterion::{black_box, criterion_group, criterion_main, Criterion};
// use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Criterion};

use farmfe_compiler::Compiler;
use farmfe_core::{
  config::{Config, RuntimeConfig},
  relative_path::RelativePath,
};

fn setup_compiler() -> Compiler {
  let relative_root = RelativePath::new("./index.ts");
  let cwd = std::env::current_dir().unwrap();
  let react_examples_root = relative_root.to_logical_path(cwd.clone());

  Compiler::new(
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
  .unwrap()
}

fn bench_compiler_compile(c: &mut Criterion) {
  let mut compiler = setup_compiler();
  c.bench_function("compiler_compile", |b| {
      b.iter(|| {
          black_box(compiler.compile().unwrap());
      })
  });
}

criterion_group!(benches, bench_compiler_compile);
criterion_main!(benches);
