use criterion::{black_box, criterion_group, criterion_main, Criterion};

use farmfe_compiler::Compiler;
use farmfe_core::{
  config::{preset_env::PresetEnvConfig, Config, RuntimeConfig},
  HashMap,
};

fn setup_compiler() -> Compiler {
  let cwd = std::env::current_dir().unwrap();
  let workspace_root = cwd.parent().unwrap().parent().unwrap();
  let example_root = workspace_root.join("examples").join("css-url");

  let rt_path = workspace_root
    .join("packages")
    .join("runtime")
    .join("src")
    .join("index.ts");
  assert!(rt_path.exists());

  let input = HashMap::from_iter(vec![(
    "index".to_string(),
    example_root
      .join("index.html")
      .to_string_lossy()
      .to_string(),
  )]);

  Compiler::new(
    Config {
      root: example_root.to_string_lossy().to_string(),
      runtime: Box::new(RuntimeConfig {
        path: rt_path.to_string_lossy().to_string(),
        plugins: vec![String::from("@farmfe/plugin-react")],
        swc_helpers_path: workspace_root
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
      preset_env: Box::new(PresetEnvConfig::Bool(false)),
      input,
      ..Default::default()
    },
    vec![],
  )
  .unwrap()
}

fn bench_compiler_compile(c: &mut Criterion) {
  let compiler = setup_compiler();
  c.bench_function("compiler_compile", |b| {
    b.iter(|| {
      // black_box(compiler.compile().unwrap());
      let compiler = setup_compiler();
      black_box(compiler.compile())
    })
  });
}

criterion_group!(benches, bench_compiler_compile);
criterion_main!(benches);
