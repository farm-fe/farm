#![deny(clippy::all)]

use std::{collections::HashMap, path::Path, sync::Arc};

use farmfe_compiler::Compiler;

pub mod plugin_adapters;
pub mod plugin_toolkit;
#[cfg(feature = "profile")]
pub mod profile_gui;

use farmfe_core::{
  config::{Config, Mode},
  module::ModuleId,
  plugin::UpdateType,
};

use napi::{
  bindgen_prelude::{Buffer, FromNapiValue},
  threadsafe_function::{
    ErrorStrategy, ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
  },
  Env, JsFunction, JsObject, NapiRaw, Status,
};
use notify::{RecommendedWatcher, Watcher};
use plugin_adapters::{js_plugin_adapter::JsPluginAdapter, rust_plugin_adapter::RustPluginAdapter};

// pub use farmfe_toolkit_plugin;

#[macro_use]
extern crate napi_derive;

#[napi(object)]
pub struct JsUpdateResult {
  pub added: Vec<String>,
  pub changed: Vec<String>,
  pub removed: Vec<String>,
  pub modules: String,
  pub boundaries: HashMap<String, Vec<Vec<String>>>,
  pub dynamic_resources_map: Option<HashMap<String, Vec<Vec<String>>>>,
}

#[napi(js_name = "Compiler")]
pub struct JsCompiler {
  compiler: Arc<Compiler>,
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
      Vec::<Vec<String>>::from_napi_value(
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

    for rust_plugin in rust_plugins {
      let rust_plugin_path = rust_plugin[0].clone();
      let rust_plugin_options = rust_plugin[1].clone();

      let rust_plugin = Arc::new(
        RustPluginAdapter::new(&rust_plugin_path, &config, rust_plugin_options)
          .unwrap_or_else(|e| panic!("load rust plugin error: {:?}", e)),
      ) as _;
      plugins_adapters.push(rust_plugin);
    }

    Ok(Self {
      compiler: Arc::new(
        Compiler::new(config, plugins_adapters)
          .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{}", e)))?,
      ),
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
      .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{}", e)))?;

    Ok(())
  }

  /// sync compile
  #[napi]
  pub fn compile_sync(&self) -> napi::Result<()> {
    #[cfg(feature = "profile")]
    {
      farmfe_core::puffin::set_scopes_on(true); // Remember to call this, or puffin will be disabled!

      let native_options = Default::default();
      let compiler = self.compiler.clone();
      let _ = eframe::run_native(
        "puffin egui eframe",
        native_options,
        Box::new(move |_cc| Box::new(profile_gui::ProfileApp::new(compiler))),
      );
    }

    #[cfg(not(feature = "profile"))]
    self
      .compiler
      .compile()
      .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{}", e)))?;

    Ok(())
  }

  /// TODO: usage example
  #[napi]
  pub fn update(
    &self,
    e: Env,
    paths: Vec<String>,
    callback: JsFunction,
    sync: bool,
  ) -> napi::Result<JsObject> {
    let context = self.compiler.context().clone();
    let compiler = self.compiler.clone();
    let thread_safe_callback: ThreadsafeFunction<(), ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| ctx.env.get_undefined().map(|v| vec![v]))?;

    e.execute_tokio_future(
      async move {
        compiler
          .update(
            paths
              .into_iter()
              .map(|p| (p, UpdateType::Updated))
              .collect(),
            move || {
              thread_safe_callback.call((), ThreadsafeFunctionCallMode::Blocking);
            },
            sync,
          )
          .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{}", e)))
      },
      move |&mut _, res| {
        Ok(JsUpdateResult {
          added: res
            .added_module_ids
            .into_iter()
            .map(|id| id.id(Mode::Development))
            .collect(),
          changed: res
            .updated_module_ids
            .into_iter()
            .map(|id| id.id(Mode::Development))
            .collect(),
          removed: res
            .removed_module_ids
            .into_iter()
            .map(|id| id.id(Mode::Development))
            .collect(),
          modules: res.resources,
          boundaries: res.boundaries,
          dynamic_resources_map: res.dynamic_resources_map.map(|dynamic_resources_map| {
            dynamic_resources_map
              .into_iter()
              .map(|(k, v)| {
                (
                  k.id(context.config.mode.clone()),
                  v.into_iter()
                    .map(|(path, ty)| vec![path, ty.to_html_tag()])
                    .collect(),
                )
              })
              .collect()
          }),
        })
      },
    )
  }

  #[napi]
  pub fn has_module(&self, resolved_path: String) -> bool {
    let context = self.compiler.context();
    let module_graph = context.module_graph.read();
    let module_id = ModuleId::new(&resolved_path, "", &context.config.root);

    module_graph.has_module(&module_id)
  }

  #[napi]
  pub fn resources(&self) -> HashMap<String, Buffer> {
    let context = self.compiler.context();
    let resources = context.resources_map.lock();

    if let Ok(node_env) = std::env::var("NODE_ENV") {
      if node_env == "test" {
        println!("resources names: {:?}", resources.keys());
      }
    }

    let mut result = HashMap::new();

    for resource in resources.values() {
      // only write expose non-emitted resource
      if !resource.emitted {
        result.insert(resource.name.clone(), resource.bytes.clone().into());
      }
    }

    if let Ok(node_env) = std::env::var("NODE_ENV") {
      if node_env == "test" {
        println!("resources to js side: {:?}", result.keys());
      }
    }

    result
  }

  #[napi]
  pub fn relative_module_paths(&self) -> Vec<String> {
    let context = self.compiler.context();
    let module_graph = context.module_graph.read();

    module_graph
      .modules()
      .into_iter()
      .map(|m| m.id.relative_path().to_string())
      .collect()
  }

  #[napi]
  pub fn resource(&self, name: String) -> Option<Buffer> {
    let context = self.compiler.context();
    let resources = context.resources_map.lock();

    resources.get(&name).map(|r| r.bytes.clone().into())
  }
}

#[napi(js_name = "JsFileWatcher")]
pub struct FileWatcher {
  watcher: notify::RecommendedWatcher,
}

#[napi]
impl FileWatcher {
  #[napi(constructor)]
  pub fn new(_: Env, callback: JsFunction) -> napi::Result<Self> {
    let thread_safe_callback: ThreadsafeFunction<Vec<String>, ErrorStrategy::Fatal> = callback
      .create_threadsafe_function(0, |ctx: ThreadSafeCallContext<Vec<String>>| {
        let mut array = ctx.env.create_array_with_length(ctx.value.len())?;

        for (i, v) in ctx.value.iter().enumerate() {
          array.set_element(i as u32, ctx.env.create_string(v)?)?;
        }

        Ok(vec![array])
        // ctx
        //   .value
        //   .into_iter()
        //   .map(|v| ctx.env.create_string(v))
        //   .map(|v| vec![v])
      })?;

    let watcher = RecommendedWatcher::new(
      move |result: std::result::Result<notify::Event, notify::Error>| {
        let event = result.unwrap();
        println!("event: {:?} {:?}", event.kind, event.paths);
        if event.kind.is_modify() {
          let paths = event
            .paths
            .iter()
            .map(|p| p.to_str().unwrap().to_string())
            .collect();
          thread_safe_callback.call(paths, ThreadsafeFunctionCallMode::Blocking);
        }
      },
      Default::default(),
    )
    .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{}", e)))?;

    Ok(Self { watcher: watcher })
  }

  #[napi]
  pub fn watch(&mut self, path: String) -> napi::Result<()> {
    self
      .watcher
      .watch(Path::new(&path), notify::RecursiveMode::Recursive)
      .map_err(|e| {
        napi::Error::new(
          Status::GenericFailure,
          format!("watch path {} failed: {}", path, e),
        )
      })
  }

  #[napi]
  pub fn unwatch(&mut self, path: String) -> napi::Result<()> {
    self.watcher.unwatch(Path::new(&path)).map_err(|e| {
      napi::Error::new(
        Status::GenericFailure,
        format!("unwatch path {} failed: {}", path, e),
      )
    })
  }
}
