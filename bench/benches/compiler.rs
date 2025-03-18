use criterion::{black_box, criterion_group, criterion_main, Criterion};

use farmfe_benchmarks::get_runtime_config;
use farmfe_compiler::Compiler;
use farmfe_core::{
  config::{
    bool_or_obj::BoolOrObj, persistent_cache::PersistentCacheConfig, preset_env::PresetEnvConfig,
    Config,
  },
  HashMap,
};

fn compiler_compile(c: &mut Criterion) {
  c.bench_function("rust compiler compile vanilla", |b| {
    b.iter(|| {
      let cwd = std::env::current_dir().unwrap();
      let root = cwd.join("benches").join("fixtures").join("vanilla");

      let compiler = Compiler::new(
        Config {
          root: root.to_string_lossy().to_string(),
          input: HashMap::from_iter([(
            "index".to_string(),
            root.join("./index.html").to_string_lossy().to_string(),
          )]),
          progress: false,
          runtime: get_runtime_config(&cwd),
          preset_env: Box::new(PresetEnvConfig::Bool(false)),
          persistent_cache: Box::new(PersistentCacheConfig::Bool(false)),
          minify: Box::new(BoolOrObj::Bool(false)),
          ..Default::default()
        },
        vec![],
      )
      .unwrap();

      black_box(compiler.compile().unwrap());
    });
  });
}

criterion_group!(benches, compiler_compile);
criterion_main!(benches);
