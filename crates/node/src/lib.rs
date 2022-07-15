#![deny(clippy::all)]

use farmfe_compiler::Compiler;

pub mod config;
pub mod plugin_adapters;

use config::JsUserConfig;

#[macro_use]
extern crate napi_derive;

#[napi(object)]
pub struct JsUpdateResult {}

#[napi(js_name = "Compiler")]
pub struct JsCompiler {
  compiler: Compiler,
}

#[napi]
impl JsCompiler {
  #[napi(constructor)]
  pub fn new(config: JsUserConfig) -> Self {
    let plugins_adapters = vec![];

    for _ in 0..config.js_plugins.len() {}

    for _ in 0..config.wasm_plugins.len() {}

    Self {
      compiler: Compiler::new(config.into(), plugins_adapters),
    }
  }

  /// async compile, return promise
  ///
  /// TODO: usage example
  #[napi]
  pub async fn compile(&self) {
    self.compiler.compile();
  }

  /// sync compile
  #[napi]
  pub fn compile_sync(&self) {}

  /// async update, return promise
  ///
  /// TODO: usage example
  #[napi]
  pub async fn update(&self, paths: Vec<String>) -> napi::Result<JsUpdateResult> {
    Ok(JsUpdateResult {})
  }

  /// sync update
  #[napi]
  pub fn update_sync(&self, paths: Vec<String>) -> napi::Result<JsUpdateResult> {
    Ok(JsUpdateResult {})
  }
}
