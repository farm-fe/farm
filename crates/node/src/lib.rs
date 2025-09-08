#![deny(clippy::all)]
#![allow(clippy::redundant_allocation)]
#![allow(clippy::blocks_in_conditions)]
#[cfg(feature = "file_watcher")]
use std::path::PathBuf;
use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use farmfe_compiler::{trace_module_graph::TracedModuleGraph, Compiler};

pub mod plugin_adapters;
#[cfg(feature = "profile")]
pub mod profile_gui;

use farmfe_core::{
  cache::module_cache::MetadataOption,
  config::{Config, Mode},
  context::CompilationContext,
  module::ModuleId,
  plugin::ResolveKind,
  HashMap,
};

use farmfe_plugin_resolve::resolver::{ResolveOptions, Resolver};
use napi::{
  bindgen_prelude::{Buffer, FromNapiValue, JsObjectValue, Object, ObjectRef, Undefined},
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Env, JsValue, Status, Unknown,
};

use plugin_adapters::{js_plugin_adapter::JsPluginAdapter, rust_plugin_adapter::RustPluginAdapter};

#[macro_use]
extern crate napi_derive;

#[cfg(feature = "file_watcher")]
pub mod file_watcher;
pub mod plugin_toolkit;

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
  pub fn new(env: Env, config: Object) -> napi::Result<Self> {
    let js_plugins = unsafe {
      Vec::<Object>::from_napi_value(
        env.raw(),
        config
          .get_named_property::<Object>("jsPlugins")
          .expect("jsPlugins must exist")
          .raw(),
      )
      .expect("jsPlugins should be an array of js functions")
    };

    let rust_plugins = unsafe {
      Vec::<Vec<String>>::from_napi_value(
        env.raw(),
        config
          .get_named_property::<Object>("rustPlugins")
          .expect("rustPlugins must exists")
          .raw(),
      )
      .expect("rustPlugins should be an array of js strings")
    };

    let config: Config = env
      .from_js_value(
        config
          .get_named_property::<Object>("config")
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
  pub fn trace_dependencies(&self, e: Env) -> napi::Result<ObjectRef> {
    let (promise, result) =
      e.create_deferred::<Vec<String>, Box<dyn FnOnce(Env) -> napi::Result<Vec<String>>>>()?;

    let compiler = self.compiler.clone();
    self.compiler.context().thread_pool.spawn(move || {
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

    result.create_ref()
  }

  #[napi]
  pub fn trace_module_graph(&self, e: Env) -> napi::Result<ObjectRef> {
    let (promise, result) =
      e.create_deferred::<JsTracedModuleGraph, Box<dyn FnOnce(Env) -> napi::Result<JsTracedModuleGraph>>>()?;

    let compiler = self.compiler.clone();
    self.compiler.context().thread_pool.spawn(move || {
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

    result.create_ref()
  }

  /// async compile, return promise
  #[napi]
  pub fn compile(&self, e: Env) -> napi::Result<ObjectRef> {
    let (promise, result) =
      e.create_deferred::<Undefined, Box<dyn FnOnce(Env) -> napi::Result<Undefined>>>()?;

    let compiler = self.compiler.clone();
    self.compiler.context().thread_pool.spawn(move || {
      match compiler
        .compile()
        .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{e}")))
      {
        Ok(_) => {
          promise.resolve(Box::new(|_| Ok(())));
        }
        Err(err) => {
          promise.reject(err);
        }
      }
    });

    result.create_ref()
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
    paths: Vec<(String, String)>,
    thread_safe_callback: ThreadsafeFunction<()>,
    sync: bool,
    generate_update_resource: bool,
  ) -> napi::Result<ObjectRef> {
    let context = self.compiler.context().clone();

    let (promise, result) =
      e.create_deferred::<JsUpdateResult, Box<dyn FnOnce(Env) -> napi::Result<JsUpdateResult>>>()?;

    let compiler = self.compiler.clone();
    self.compiler.context().thread_pool.spawn(move || {
      match compiler
        .update(
          paths
            .into_iter()
            .map(|(path, ty)| (path, ty.into()))
            .collect(),
          move || {
            thread_safe_callback.call(Ok(()), ThreadsafeFunctionCallMode::Blocking);
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
                    k.id(context.config.mode),
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

    result.create_ref()
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

    let mut result = HashMap::default();

    for resource in resources.values() {
      // only write expose non-emitted resource
      if !resource.emitted {
        result.insert(resource.name.clone(), resource.bytes.clone().into());
      }
    }

    result
  }

  #[napi]
  pub fn resources_map(&self, e: Env) -> HashMap<String, Unknown> {
    let context = self.compiler.context();
    let resources = context.resources_map.lock();
    let mut resources_map = HashMap::default();

    for (name, resource) in resources.iter() {
      resources_map.insert(name.clone(), e.to_js_value(resource).unwrap());
    }

    resources_map
  }

  #[napi]
  pub fn write_resources_to_disk(&self) {
    self.compiler.write_resources_to_disk().unwrap();
  }

  #[napi]
  pub fn watch_modules(&self) -> Vec<String> {
    let context = self.compiler.context();

    let watch_graph = context.watch_graph.read();

    watch_graph
      .modules()
      .into_iter()
      .map(|id| id.resolved_path(&context.config.root))
      .collect()
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
    context.stats.to_string()
  }

  #[napi]
  pub fn invalidate_module(&self, module_id: String) {
    invalidate_module(self, module_id);
  }

  /// Write cache with name and data
  #[napi]
  pub fn write_cache(&self, name: String, data: String, options: Option<JsApiMetadata>) {
    write_cache(self, name, data, options.map(|v| v.into()));
  }

  /// Read cache with name, return `undefined` if not exists
  #[napi]
  pub fn read_cache(&self, name: String, options: Option<JsApiMetadata>) -> Option<String> {
    read_cache(self, name, options.map(|v| v.into()))
  }

  #[napi]
  pub fn read_cache_by_scope(&self, scope: String) -> Vec<String> {
    read_cache_by_scope(self, scope)
  }
}

#[napi(object)]
pub struct JsApiMetadata {
  /// Scope of the cache, used different name, same scope will hit the same cache
  pub scope: Option<Vec<String>>,
  /// reference ModuleId of the cache, when the module is invalidated, the cache will be invalidated too
  pub refer: Option<Vec<String>>,
}

impl Into<MetadataOption> for JsApiMetadata {
  fn into(self) -> MetadataOption {
    MetadataOption {
      scope: self.scope,
      refer: self.refer,
    }
  }
}

fn read_cache(
  js_compiler: &JsCompiler,
  name: String,
  options: Option<MetadataOption>,
) -> Option<String> {
  let context = js_compiler.compiler.context();
  context.read_cache(&name, options).map(|v| *v)
}

fn write_cache(
  js_compiler: &JsCompiler,
  name: String,
  data: String,
  options: Option<MetadataOption>,
) {
  let context = js_compiler.compiler.context();
  context.write_cache(&name, data, options);
}

fn read_cache_by_scope(js_compiler: &JsCompiler, scope: String) -> Vec<String> {
  let context = js_compiler.compiler.context();
  context.read_cache_by_scope::<String>(&scope)
}

fn invalidate_module(js_compiler: &JsCompiler, module_id: String) {
  let context = js_compiler.compiler.context();
  let module_id = ModuleId::new(&module_id, "", &context.config.root);

  context.invalidate_module(&module_id);
}

#[napi(js_name = "Resolver")]
pub struct JsResolver {
  resolver: Arc<Resolver>,
  context: Arc<CompilationContext>,
}

#[napi]
impl JsResolver {
  #[napi(constructor)]
  pub fn new(env: Env, config: Object) -> napi::Result<Self> {
    let config: Config = env
      .from_js_value(config)
      .expect("Create Resolver failed: Can not transform js config object to rust config");

    Ok(Self {
      resolver: Arc::new(Resolver::new()),
      context: Arc::new(
        CompilationContext::new(config, vec![])
          .expect("Create Resolver failed: Can not create compilation context"),
      ),
    })
  }

  #[napi]
  pub fn resolve(
    &self,
    source: String,
    base_dir: String,
    dynamic_extensions: Option<Vec<String>>,
  ) -> napi::Result<String> {
    let base_dir = PathBuf::from(base_dir);
    let options = ResolveOptions { dynamic_extensions };

    let result = self
      .resolver
      .resolve(
        &source,
        base_dir,
        &ResolveKind::Import,
        &options,
        &self.context,
      )
      .unwrap();

    Ok(result.resolved_path)
  }
}
