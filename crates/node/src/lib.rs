#![deny(clippy::all)]

use std::{collections::HashMap, sync::Arc};

use farmfe_compiler::Compiler;

pub mod plugin_adapters;

use farmfe_core::config::Config;
use napi::{bindgen_prelude::FromNapiValue, Env, JsObject, NapiRaw, Status};
use plugin_adapters::{js_plugin_adapter::JsPluginAdapter, rust_plugin_adapter::RustPluginAdapter};

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
  pub fn new(env: Env, config: JsObject) -> napi::Result<Self> {
    let js_plugins = unsafe {
      Vec::<JsObject>::from_napi_value(
        env.raw(),
        config
          .get_named_property::<JsObject>("jsPlugins")
          .expect("jsPlugins must exist")
          .raw(),
      )
      .expect("jsPlugins should be an array of js functions")
    };

    let rust_plugins = unsafe {
      Vec::<String>::from_napi_value(
        env.raw(),
        config
          .get_named_property::<JsObject>("rustPlugins")
          .expect("rustPlugins must exists")
          .raw(),
      )
      .expect("rustPlugins should be an array of js strings")
    };

    let config: Config = env
      .from_js_value(
        config
          .get_named_property::<JsObject>("config")
          .expect("config should exist"),
      )
      .expect("can not transform js config object to rust config");
    let mut plugins_adapters = vec![];

    for js_plugin_object in js_plugins {
      let js_plugin = Arc::new(
        JsPluginAdapter::new(&env, js_plugin_object)
          .unwrap_or_else(|e| panic!("load rust plugin error: {:?}", e)),
      ) as _;
      plugins_adapters.push(js_plugin);
    }

    for rust_plugin_path in rust_plugins {
      let rust_plugin = Arc::new(
        RustPluginAdapter::new(&rust_plugin_path, &config)
          .unwrap_or_else(|e| panic!("load rust plugin error: {:?}", e)),
      ) as _;
      plugins_adapters.push(rust_plugin);
    }

    Ok(Self {
      compiler: Compiler::new(config, plugins_adapters)
        .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{}", e)))?,
    })
  }

  /// async compile, return promise
  ///
  /// TODO: usage example
  #[napi]
  pub async fn compile(&self) -> napi::Result<()> {
    self
      .compiler
      .compile()
      .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{}", e)))
  }

  /// sync compile
  #[napi]
  pub fn compile_sync(&self) -> napi::Result<()> {
    self
      .compiler
      .compile()
      .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{}", e)))
  }

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

  #[napi]
  pub fn resources(&self) -> HashMap<String, Vec<u8>> {
    let context = self.compiler.context();
    let resources = context.resources_map.lock();

    let mut result = HashMap::new();

    for resource in resources.values() {
      result.insert(resource.name.clone(), resource.bytes.clone());
    }

    result
  }
}
