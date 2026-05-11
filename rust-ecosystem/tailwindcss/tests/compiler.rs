use farmfe_ecosystem_tailwindcss::{compile, CompilerOptions, Features, Polyfills, TailwindConfig};
use farmfe_testing_helpers::assert_snapshot;
use serde_json::json;

#[test]
fn core_compiler_accepts_external_config() {
  let mut compiler = compile(
    ".foo { color: red; }",
    CompilerOptions {
      features: Features::AT_APPLY | Features::THEME_FUNCTION,
      polyfills: Polyfills::AT_MEDIA_HOVER,
      dependencies: vec!["/tmp/input.css".to_string()],
      source_maps_enabled: true,
      config: Some(TailwindConfig::new(json!({
        "theme": {
          "extend": {
            "colors": {
              "brand": "#123456"
            }
          }
        },
        "content": ["src/**/*.tsx"]
      }))),
    },
  );

  let output = format!(
    "css: {}\nmap: {}\ndeps: {}\nhas_config: {}\nfeatures: {}\npolyfills: {}",
    compiler.build(&[]),
    compiler.build_source_map().unwrap_or_default(),
    compiler.dependencies().join(","),
    compiler.config().is_some(),
    compiler.features.contains(Features::AT_APPLY),
    compiler.polyfills.contains(Polyfills::AT_MEDIA_HOVER),
  );

  assert_snapshot!(output);
}
