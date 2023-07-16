#![deny(clippy::all)]

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::Arc,
};

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
use notify::{
  event::{AccessKind, ModifyKind},
  EventKind, RecommendedWatcher, Watcher,
};
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

pub struct FsWatcher {
  watcher: notify::RecommendedWatcher,
  watched_paths: Vec<PathBuf>,
}

impl FsWatcher {
  pub fn new<F>(mut callback: F) -> notify::Result<Self>
  where
    F: FnMut(Vec<String>) + Send + Sync + 'static,
  {
    let watcher = RecommendedWatcher::new(
      move |result: std::result::Result<notify::Event, notify::Error>| {
        let event = result.unwrap();
        let get_paths = || {
          event
            .paths
            .iter()
            .map(|p| p.to_str().unwrap().to_string())
            .collect::<Vec<_>>()
        };
        // println!("{:?} {:?}", event.kind, event);
        if cfg!(target_os = "macos") {
          if matches!(event.kind, EventKind::Modify(ModifyKind::Data(_))) {
            callback(get_paths());
          }
        } else if cfg!(target_os = "linux") {
          // a close event is always followed by a modify event
          if matches!(event.kind, EventKind::Access(AccessKind::Close(_))) {
            callback(get_paths());
          }
        } else if event.kind.is_modify() {
          callback(get_paths());
        }
      },
      Default::default(),
    )?;

    Ok(Self {
      watcher,
      watched_paths: vec![],
    })
  }

  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub fn watch(&mut self, paths: Vec<&Path>) -> notify::Result<()> {
    if paths.is_empty() {
      return Ok(());
    }
    // find the longest common prefix
    let mut prefix_comps = vec![];
    let first_item = &paths[0];
    let rest = &paths[1..];

    for (index, comp) in first_item.components().enumerate() {
      if rest.iter().all(|item| {
        let mut item_comps = item.components();

        if index >= item.components().count() {
          return false;
        }

        item_comps.nth(index).unwrap() == comp
      }) {
        prefix_comps.push(comp);
      }
    }

    let watch_path = PathBuf::from_iter(prefix_comps.iter());

    if self
      .watched_paths
      .iter()
      .any(|item| watch_path.starts_with(item))
    {
      return Ok(());
    } else {
      self.watched_paths.push(watch_path.clone());
    }

    // println!("watch path {:?}", watch_path);

    self
      .watcher
      .watch(watch_path.as_path(), notify::RecursiveMode::Recursive)
  }

  #[cfg(target_os = "linux")]
  pub fn watch(&mut self, paths: Vec<&Path>) -> notify::Result<()> {
    for path in paths {
      if self.watched_paths.contains(&path.to_path_buf()) {
        continue;
      }

      self
        .watcher
        .watch(path, notify::RecursiveMode::NonRecursive)
        .ok();

      self.watched_paths.push(path.to_path_buf());
    }

    Ok(())
  }

  pub fn unwatch(&mut self, path: &str) -> notify::Result<()> {
    self.watcher.unwatch(Path::new(path))
  }
}

#[napi(js_name = "JsFileWatcher")]
pub struct FileWatcher {
  watcher: FsWatcher,
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
      })?;

    let watcher = FsWatcher::new(move |paths| {
      thread_safe_callback.call(paths, ThreadsafeFunctionCallMode::Blocking);
    })
    .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{}", e)))?;

    Ok(Self { watcher })
  }

  #[napi]
  pub fn watch(&mut self, paths: Vec<String>) -> napi::Result<()> {
    self
      .watcher
      .watch(paths.iter().map(Path::new).collect())
      .ok();

    Ok(())
  }

  #[napi]
  pub fn unwatch(&mut self, paths: Vec<String>) -> napi::Result<()> {
    for path in paths {
      self.watcher.unwatch(&path).ok();
    }

    Ok(())
  }
}
