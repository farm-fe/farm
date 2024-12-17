#![deny(clippy::all)]
#![allow(clippy::redundant_allocation)]
#![allow(clippy::blocks_in_conditions)]
#[cfg(feature = "file_watcher")]
use std::path::PathBuf;
use std::{collections::HashMap, path::Path, sync::Arc};

use farmfe_compiler::{trace_module_graph::TracedModuleGraph, Compiler};

pub mod plugin_adapters;
pub mod plugin_toolkit;
#[cfg(feature = "profile")]
pub mod profile_gui;

use farmfe_core::{
  config::{Config, Mode},
  module::ModuleId,
  plugin::UpdateType,
};

#[cfg(feature = "file_watcher")]
use farmfe_core::resource::Resource;
#[cfg(feature = "file_watcher")]
use napi::threadsafe_function::ThreadSafeCallContext;

use napi::{
  bindgen_prelude::{Buffer, FromNapiValue},
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Env, JsFunction, JsObject, JsUndefined, JsUnknown, NapiRaw, Status,
};

#[cfg(feature = "file_watcher")]
use notify::{
  event::{AccessKind, ModifyKind},
  EventKind, RecommendedWatcher, Watcher,
};
use plugin_adapters::{js_plugin_adapter::JsPluginAdapter, rust_plugin_adapter::RustPluginAdapter};

// pub use farmfe_toolkit_plugin;

#[macro_use]
extern crate napi_derive;

#[napi(object)]
pub struct WatchDiffResult {
  pub add: Vec<String>,
  pub remove: Vec<String>,
}

#[napi(object)]
pub struct JsTracedModule {
  pub id: String,
  pub content_hash: String,
  pub package_name: String,
  pub package_version: String,
}

#[napi(object)]
pub struct JsTracedModuleGraph {
  pub root: String,
  pub modules: Vec<JsTracedModule>,
  pub edges: HashMap<String, Vec<String>>,
  pub reverse_edges: HashMap<String, Vec<String>>,
}

impl From<TracedModuleGraph> for JsTracedModuleGraph {
  fn from(t: TracedModuleGraph) -> Self {
    Self {
      root: t.root,
      modules: t
        .modules
        .into_iter()
        .map(|m| JsTracedModule {
          id: m.id,
          content_hash: m.content_hash,
          package_name: m.package_name,
          package_version: m.package_version,
        })
        .collect(),
      edges: t.edges,
      reverse_edges: t.reverse_edges,
    }
  }
}

#[napi(object)]
pub struct JsUpdateResult {
  pub added: Vec<String>,
  pub changed: Vec<String>,
  pub removed: Vec<String>,
  pub immutable_modules: String,
  pub mutable_modules: String,
  pub boundaries: HashMap<String, Vec<Vec<String>>>,
  pub dynamic_resources_map: Option<HashMap<String, Vec<Vec<String>>>>,
  pub extra_watch_result: WatchDiffResult,
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
          .unwrap_or_else(|e| panic!("load js plugin error: {e:?}")),
      ) as _;
      plugins_adapters.push(js_plugin);
    }

    for rust_plugin in rust_plugins {
      let rust_plugin_path = rust_plugin[0].clone();
      let rust_plugin_options = rust_plugin[1].clone();

      let rust_plugin = Arc::new(
        RustPluginAdapter::new(&rust_plugin_path, &config, rust_plugin_options)
          .unwrap_or_else(|e| panic!("load rust plugin error: {e:?}")),
      ) as _;
      plugins_adapters.push(rust_plugin);
    }

    Ok(Self {
      compiler: Arc::new(
        Compiler::new(config, plugins_adapters)
          .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{e}")))?,
      ),
    })
  }

  #[napi]
  pub fn trace_dependencies(&self, e: Env) -> napi::Result<JsObject> {
    let (promise, result) =
      e.create_deferred::<Vec<String>, Box<dyn FnOnce(Env) -> napi::Result<Vec<String>>>>()?;

    let compiler = self.compiler.clone();
    self.compiler.thread_pool.spawn(move || {
      match compiler
        .trace_dependencies()
        .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{e}")))
      {
        Ok(deps) => {
          promise.resolve(Box::new(|_| Ok(deps)));
        }
        Err(err) => {
          promise.reject(err);
        }
      }
    });

    Ok(result)
  }

  #[napi]
  pub fn trace_module_graph(&self, e: Env) -> napi::Result<JsObject> {
    let (promise, result) =
      e.create_deferred::<JsTracedModuleGraph, Box<dyn FnOnce(Env) -> napi::Result<JsTracedModuleGraph>>>()?;

    let compiler = self.compiler.clone();
    self.compiler.thread_pool.spawn(move || {
      match compiler
        .trace_module_graph()
        .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{e}")))
      {
        Ok(graph) => {
          promise.resolve(Box::new(|_| Ok(graph.into())));
        }
        Err(err) => {
          promise.reject(err);
        }
      }
    });

    Ok(result)
  }

  /// async compile, return promise
  #[napi]
  pub fn compile(&self, e: Env) -> napi::Result<JsObject> {
    let (promise, result) =
      e.create_deferred::<JsUndefined, Box<dyn FnOnce(Env) -> napi::Result<JsUndefined>>>()?;

    let compiler = self.compiler.clone();
    self.compiler.thread_pool.spawn(move || {
      match compiler
        .compile()
        .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{e}")))
      {
        Ok(_) => {
          promise.resolve(Box::new(|e| e.get_undefined()));
        }
        Err(err) => {
          promise.reject(err);
        }
      }
    });

    Ok(result)
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
      .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{e}")))?;

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
    generate_update_resource: bool,
  ) -> napi::Result<JsObject> {
    let context = self.compiler.context().clone();
    let thread_safe_callback: ThreadsafeFunction<(), ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| ctx.env.get_undefined().map(|v| vec![v]))?;

    let (promise, result) =
      e.create_deferred::<JsUpdateResult, Box<dyn FnOnce(Env) -> napi::Result<JsUpdateResult>>>()?;

    let compiler = self.compiler.clone();
    self.compiler.thread_pool.spawn(move || {
      match compiler
        .update(
          paths
            .into_iter()
            .map(|p| (p, UpdateType::Updated))
            .collect(),
          move || {
            thread_safe_callback.call((), ThreadsafeFunctionCallMode::Blocking);
          },
          sync,
          generate_update_resource,
        )
        .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{e}")))
      {
        Ok(res) => {
          let js_update_result = JsUpdateResult {
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
            immutable_modules: res.immutable_resources,
            mutable_modules: res.mutable_resources,
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

            extra_watch_result: WatchDiffResult {
              add: res
                .extra_watch_result
                .add
                .into_iter()
                .map(|path| ModuleId::new(&path, "", &context.config.root).id(Mode::Development))
                .collect(),
              remove: res
                .extra_watch_result
                .remove
                .into_iter()
                .map(|path| ModuleId::new(&path, "", &context.config.root).id(Mode::Development))
                .collect(),
            },
          };

          promise.resolve(Box::new(move |_| Ok(js_update_result)));
        }
        Err(err) => {
          promise.reject(err);
        }
      }
    });

    Ok(result)
  }

  #[napi]
  pub fn add_watch_files(&self, root: String, paths: Vec<String>) {
    let context = self.compiler.context().clone();
    let root = ModuleId::new(&root, "", &context.config.root);

    context
      .add_watch_files(
        root,
        paths
          .iter()
          .map(|i| ModuleId::new(i, "", &context.config.root))
          .collect(),
      )
      .expect("failed add extra files to watch list");
  }

  #[napi]
  pub fn has_module(&self, resolved_path: String) -> bool {
    let context = self.compiler.context();
    let module_graph = context.module_graph.read();
    let watch_graph = context.watch_graph.read();
    let module_id = ModuleId::new(&resolved_path, "", &context.config.root);
    let module_ids_by_file = module_graph.module_ids_by_file(&module_id);

    module_graph.has_module(&module_id)
      || watch_graph.has_module(&module_id)
      || !module_ids_by_file.is_empty()
  }

  #[napi]
  pub fn get_parent_files(&self, resolved_path: String) -> Vec<String> {
    let context = self.compiler.context();
    let module_graph = context.module_graph.read();
    let path = Path::new(&resolved_path);
    let module_id = if path.is_absolute() {
      ModuleId::from_resolved_path_with_query(&resolved_path, &context.config.root)
    } else {
      resolved_path.into()
    };
    let current_exec_order = module_graph
      .module(&module_id)
      .map(|m| m.execution_order)
      .unwrap_or(usize::MAX);
    let parents = module_graph.dependents_ids(&module_id);

    parents
      .into_iter()
      .filter(|id| {
        let module = module_graph.module(id).unwrap();
        module.execution_order > current_exec_order
      })
      .map(|p| p.resolved_path_with_query(&context.config.root))
      .collect()
  }

  #[napi]
  pub fn resources(&self) -> HashMap<String, Buffer> {
    let context = self.compiler.context();
    let resources = context.resources_map.lock();

    let mut result = HashMap::new();

    for resource in resources.values() {
      // only write expose non-emitted resource
      if !resource.emitted {
        result.insert(resource.name.clone(), resource.bytes.clone().into());
      }
    }

    result
  }

  #[napi]
  pub fn resources_map(&self, e: Env) -> HashMap<String, JsUnknown> {
    let context = self.compiler.context();
    let resources = context.resources_map.lock();
    let mut resources_map = HashMap::new();

    for (name, resource) in resources.iter() {
      resources_map.insert(name.clone(), e.to_js_value(resource).unwrap());
    }

    resources_map
  }

  #[napi]
  pub fn watch_modules(&self) -> Vec<String> {
    let context = self.compiler.context();

    let watch_graph = context.watch_graph.read();

    return watch_graph
      .modules()
      .into_iter()
      .map(|id| id.resolved_path(&context.config.root))
      .collect();
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

  #[napi]
  pub fn stats(&self) -> String {
    let context = self.compiler.context();
    context.record_manager.to_string()
  }

  #[napi]
  pub fn invalidate_module(&self, module_id: String) {
    invalidate_module(self, module_id);
  }
}

fn invalidate_module(js_compiler: &JsCompiler, module_id: String) {
  let context = js_compiler.compiler.context();
  let module_id = ModuleId::new(&module_id, "", &context.config.root);

  context.invalidate_module(&module_id);
}

#[cfg(feature = "file_watcher")]
pub struct FsWatcher {
  watcher: notify::RecommendedWatcher,
  watched_paths: Vec<PathBuf>,
}
#[cfg(feature = "file_watcher")]
impl FsWatcher {
  pub fn new<F>(mut callback: F) -> notify::Result<Self>
  where
    F: FnMut(Vec<String>) + Send + Sync + 'static,
  {
    // TODO support other kind of events
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

#[cfg(feature = "file_watcher")]
#[napi(js_name = "JsFileWatcher")]
pub struct FileWatcher {
  watcher: FsWatcher,
}

#[cfg(feature = "file_watcher")]
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
