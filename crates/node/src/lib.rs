#![feature(box_patterns)]
#![deny(clippy::all)]
#![allow(clippy::redundant_allocation)]
#![allow(clippy::blocks_in_conditions)]
use std::{
  fs,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

use farmfe_compiler::{trace_module_graph::TracedModuleGraph, Compiler};

pub mod module_runner_transform;
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
use farmfe_toolkit::sourcemap::{collapse_sourcemap_chain, CollapseSourcemapOptions, SourceMap};
use napi::{
  bindgen_prelude::{Buffer, FromNapiValue, JsObjectValue, Object, ObjectRef, Undefined},
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Env, JsValue, Status, Unknown,
};

use module_runner_transform::transform_script_module_to_runner_code;
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

#[napi(object)]
pub struct JsFetchModuleOptions {
  pub cached: Option<bool>,
  pub start_offset: Option<i32>,
}

#[napi(object)]
pub struct JsFetchModuleResult {
  pub cache: Option<bool>,
  pub externalize: Option<String>,
  pub bailout_reason: Option<String>,
  pub r#type: Option<String>,
  pub code: Option<String>,
  pub file: Option<String>,
  pub id: Option<String>,
  pub url: Option<String>,
  pub invalidate: Option<bool>,
  pub map: Option<String>,
}

#[napi(js_name = "Compiler")]
pub struct JsCompiler {
  compiler: Arc<Compiler>,
  module_runner_transform_cache: RunnerTransformCacheStore,
}

type RunnerTransformCacheStore = Mutex<HashMap<String, RunnerTransformCacheEntry>>;

#[derive(Clone)]
struct RunnerTransformCacheEntry {
  content_hash: String,
  code: String,
  map: Option<String>,
}

fn clean_url(value: &str) -> &str {
  let without_query = value
    .split_once('?')
    .map(|(before, _)| before)
    .unwrap_or(value);

  without_query
    .split_once('#')
    .map(|(before, _)| before)
    .unwrap_or(without_query)
}

fn path_to_file_url(path: &Path) -> String {
  let normalized = path.to_string_lossy().replace('\\', "/");

  if cfg!(windows) {
    if normalized.starts_with('/') {
      format!("file://{normalized}")
    } else {
      format!("file:///{normalized}")
    }
  } else {
    format!("file://{normalized}")
  }
}

fn normalize_importer_path(importer: Option<&str>, root: &str) -> Option<PathBuf> {
  let importer = clean_url(importer?);

  if importer.starts_with("file://") {
    let raw = importer.trim_start_matches("file://");
    #[cfg(windows)]
    {
      if raw.len() >= 3 && raw.starts_with('/') && raw.as_bytes()[2] == b':' {
        return Some(PathBuf::from(&raw[1..]));
      }
    }
    return Some(PathBuf::from(raw));
  }

  if importer.starts_with('/') {
    return Some(Path::new(root).join(importer.trim_start_matches('/')));
  }

  Some(PathBuf::from(importer))
}

fn try_resolve_file_path(id: &str, importer: Option<&str>, root: &str) -> Option<PathBuf> {
  let id = clean_url(id);

  if id.starts_with("file://") {
    return normalize_importer_path(Some(id), root);
  }

  if Path::new(id).is_absolute() {
    return Some(PathBuf::from(id));
  }

  if id.starts_with('/') {
    return Some(Path::new(root).join(id.trim_start_matches('/')));
  }

  if id.starts_with("./") || id.starts_with("../") {
    let importer_path = normalize_importer_path(importer, root)?;
    let base_dir = if importer_path.is_file() {
      importer_path.parent().unwrap_or(importer_path.as_path())
    } else {
      importer_path.as_path()
    };
    return Some(base_dir.join(id));
  }

  None
}

fn module_type_by_path(path: &Path) -> String {
  if path
    .extension()
    .is_some_and(|ext| ext.eq_ignore_ascii_case("cjs"))
  {
    return "commonjs".to_string();
  }

  "module".to_string()
}

fn external_fetch_result(
  externalize: String,
  r#type: String,
  bailout_reason: Option<String>,
) -> JsFetchModuleResult {
  JsFetchModuleResult {
    cache: None,
    externalize: Some(externalize),
    bailout_reason,
    r#type: Some(r#type),
    code: None,
    file: None,
    id: None,
    url: None,
    invalidate: None,
    map: None,
  }
}

fn extract_query_string(value: &str) -> String {
  let without_hash = value
    .split_once('#')
    .map(|(before, _)| before)
    .unwrap_or(value);
  without_hash
    .split_once('?')
    .map(|(_, query)| format!("?{query}"))
    .unwrap_or_default()
}

fn stringify_query_pairs(query: &[(String, String)]) -> String {
  if query.is_empty() {
    return String::new();
  }

  format!(
    "?{}",
    query
      .iter()
      .map(|(key, value)| {
        if value.is_empty() {
          key.clone()
        } else {
          format!("{key}={value}")
        }
      })
      .collect::<Vec<_>>()
      .join("&")
  )
}

fn module_id_from_resolved_path_and_query(
  resolved_path: &str,
  query_string: &str,
  root: &str,
) -> ModuleId {
  if resolved_path.contains('?') {
    return ModuleId::from_resolved_path_with_query(resolved_path, root);
  }

  if query_string.is_empty() {
    return ModuleId::new(resolved_path, "", root);
  }

  ModuleId::from_resolved_path_with_query(&format!("{resolved_path}{query_string}"), root)
}

fn is_runner_ready_inlined_code(code: &str) -> bool {
  code.contains("__farm_ssr_export_name__")
    || code.contains("__farm_ssr_import__")
    || code.contains("__farm_ssr_dynamic_import__")
}

fn collapse_module_source_map(
  module: &farmfe_core::module::Module,
  transformed_map: Option<String>,
) -> Option<String> {
  let mut chain = module
    .source_map_chain
    .iter()
    .filter_map(|item| SourceMap::from_slice(item.as_bytes()).ok())
    .collect::<Vec<_>>();

  if let Some(map) = transformed_map {
    if let Ok(parsed) = SourceMap::from_slice(map.as_bytes()) {
      chain.push(parsed);
    }
  }

  if chain.is_empty() {
    return None;
  }

  let collapsed = collapse_sourcemap_chain(chain, CollapseSourcemapOptions::default());
  let mut buffer = vec![];
  collapsed.to_writer(&mut buffer).ok()?;

  String::from_utf8(buffer).ok()
}

fn inlined_fetch_result(
  module_id: &ModuleId,
  code: String,
  file: Option<String>,
  url: String,
  map: Option<String>,
) -> JsFetchModuleResult {
  JsFetchModuleResult {
    cache: None,
    externalize: None,
    bailout_reason: None,
    r#type: None,
    code: Some(code),
    file,
    id: Some(module_id.to_string()),
    url: Some(url),
    invalidate: Some(false),
    map,
  }
}

enum InlinedFetchAttempt {
  Inlined(JsFetchModuleResult),
  Bailout(String),
  NotApplicable,
}

fn get_cached_runner_transform(
  cache: &RunnerTransformCacheStore,
  module_key: &str,
  content_hash: &str,
) -> Option<(String, Option<String>)> {
  let cache = cache.lock().ok()?;
  let entry = cache.get(module_key)?;

  if entry.content_hash != content_hash {
    return None;
  }

  Some((entry.code.clone(), entry.map.clone()))
}

fn set_cached_runner_transform(
  cache: &RunnerTransformCacheStore,
  module_key: String,
  content_hash: String,
  code: String,
  map: Option<String>,
) {
  if let Ok(mut cache) = cache.lock() {
    cache.insert(
      module_key,
      RunnerTransformCacheEntry {
        content_hash,
        code,
        map,
      },
    );
  }
}

fn invalidate_cached_runner_transform(cache: &RunnerTransformCacheStore, module_key: &str) {
  if let Ok(mut cache) = cache.lock() {
    cache.remove(module_key);
  }
}

fn try_inlined_fetch_result(
  context: &Arc<CompilationContext>,
  module_id: &ModuleId,
  transform_cache: Option<&RunnerTransformCacheStore>,
) -> InlinedFetchAttempt {
  let module_graph = context.module_graph.read();
  let Some(module) = module_graph.module(module_id) else {
    return InlinedFetchAttempt::NotApplicable;
  };

  if module.external {
    return InlinedFetchAttempt::NotApplicable;
  }

  if !module.module_type.is_script() {
    return InlinedFetchAttempt::Bailout("not-script".to_string());
  }

  if module.content.is_empty() {
    return InlinedFetchAttempt::Bailout("empty-content".to_string());
  }

  let code = module.content.to_string();

  // Keep current runner behavior stable: only inline code that is already transformed
  // to farm runner runtime calls.
  if !is_runner_ready_inlined_code(&code) {
    let module_key = module_id.to_string();
    if let Some(cache) = transform_cache {
      if let Some((cached_code, cached_map)) =
        get_cached_runner_transform(cache, &module_key, &module.content_hash)
      {
        let map = collapse_module_source_map(module, cached_map);
        let file = module_id.resolved_path(&context.config.root);
        let url = module_id.resolved_path_with_query(&context.config.root);

        return InlinedFetchAttempt::Inlined(inlined_fetch_result(
          module_id,
          cached_code,
          Some(file),
          url,
          map,
        ));
      }
    }

    match transform_script_module_to_runner_code(module, module_id, context) {
      Ok((transformed_code, transformed_map)) => {
        if let Some(cache) = transform_cache {
          set_cached_runner_transform(
            cache,
            module_key,
            module.content_hash.clone(),
            transformed_code.clone(),
            transformed_map.clone(),
          );
        }

        let map = collapse_module_source_map(module, transformed_map);
        let file = module_id.resolved_path(&context.config.root);
        let url = module_id.resolved_path_with_query(&context.config.root);

        return InlinedFetchAttempt::Inlined(inlined_fetch_result(
          module_id,
          transformed_code,
          Some(file),
          url,
          map,
        ));
      }
      Err(reason) => {
        if let Some(cache) = transform_cache {
          invalidate_cached_runner_transform(cache, &module_key);
        }
        return InlinedFetchAttempt::Bailout(reason.as_str().to_string());
      }
    }
  }

  let map = collapse_module_source_map(module, None);
  let file = module_id.resolved_path(&context.config.root);
  let url = module_id.resolved_path_with_query(&context.config.root);

  InlinedFetchAttempt::Inlined(inlined_fetch_result(module_id, code, Some(file), url, map))
}

#[cfg(test)]
mod tests {
  use std::{sync::Arc, sync::Mutex};

  use farmfe_core::module::{Module, ModuleId};
  use farmfe_core::HashMap;
  use farmfe_toolkit::sourcemap::SourceMap;

  use super::{
    collapse_module_source_map, extract_query_string, get_cached_runner_transform,
    is_runner_ready_inlined_code, set_cached_runner_transform, stringify_query_pairs,
  };

  #[test]
  fn should_extract_query_without_hash() {
    assert_eq!(extract_query_string("/src/entry.ts?t=1#hash"), "?t=1");
    assert_eq!(extract_query_string("/src/entry.ts"), "");
  }

  #[test]
  fn should_stringify_query_pairs() {
    assert_eq!(
      stringify_query_pairs(&vec![
        ("t".to_string(), "1".to_string()),
        ("raw".to_string(), "".to_string())
      ]),
      "?t=1&raw"
    );
  }

  #[test]
  fn should_detect_runner_ready_code() {
    assert!(is_runner_ready_inlined_code(
      "__farm_ssr_export_name__(\"value\", () => 1);"
    ));
    assert!(!is_runner_ready_inlined_code("export const value = 1;"));
  }

  #[test]
  fn should_hit_runner_transform_cache_when_hash_matches() {
    let cache = Mutex::new(HashMap::default());
    set_cached_runner_transform(
      &cache,
      "/src/entry.ts".to_string(),
      "hash-a".to_string(),
      "code-a".to_string(),
      Some("map-a".to_string()),
    );

    let hit = get_cached_runner_transform(&cache, "/src/entry.ts", "hash-a");
    assert_eq!(hit, Some(("code-a".to_string(), Some("map-a".to_string()))));

    let miss = get_cached_runner_transform(&cache, "/src/entry.ts", "hash-b");
    assert!(miss.is_none());
  }

  #[test]
  fn should_replace_runner_transform_cache_when_hash_changes() {
    let cache = Mutex::new(HashMap::default());
    set_cached_runner_transform(
      &cache,
      "/src/entry.ts".to_string(),
      "hash-a".to_string(),
      "code-a".to_string(),
      Some("map-a".to_string()),
    );
    set_cached_runner_transform(
      &cache,
      "/src/entry.ts".to_string(),
      "hash-b".to_string(),
      "code-b".to_string(),
      Some("map-b".to_string()),
    );

    assert!(get_cached_runner_transform(&cache, "/src/entry.ts", "hash-a").is_none());
    assert_eq!(
      get_cached_runner_transform(&cache, "/src/entry.ts", "hash-b"),
      Some(("code-b".to_string(), Some("map-b".to_string())))
    );
  }

  #[test]
  fn should_collapse_transformed_map_after_existing_chain() {
    let mut module = Module::new(ModuleId::from("entry.ts"));
    module.source_map_chain = vec![Arc::new(
      r#"{
  "version": 3,
  "file": "b.js",
  "sources": ["a.ts"],
  "names": [],
  "mappings": "AAAA"
}"#
        .to_string(),
    )];

    let transformed_map = Some(
      r#"{
  "version": 3,
  "file": "c.js",
  "sources": ["b.js"],
  "names": [],
  "mappings": "AAAA"
}"#
        .to_string(),
    );

    let collapsed = collapse_module_source_map(&module, transformed_map).expect("collapsed map");
    let parsed = SourceMap::from_slice(collapsed.as_bytes()).expect("valid source map");
    let sources = parsed
      .tokens()
      .filter_map(|token| token.get_source().map(|s| s.to_string()))
      .collect::<std::collections::HashSet<_>>();

    assert!(sources.contains("a.ts"), "{sources:?}");
    assert!(!sources.contains("b.js"), "{sources:?}");
  }
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
      module_runner_transform_cache: Mutex::new(HashMap::default()),
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
  pub fn fetch_module(
    &self,
    id: String,
    importer: Option<String>,
    _options: Option<JsFetchModuleOptions>,
  ) -> Option<JsFetchModuleResult> {
    let raw_id = id;
    let clean_id = clean_url(&raw_id).to_string();
    let has_query = raw_id.contains('?');

    if clean_id.starts_with("data:") {
      return Some(external_fetch_result(clean_id, "builtin".to_string(), None));
    }

    if clean_id.starts_with("node:") {
      return Some(external_fetch_result(clean_id, "builtin".to_string(), None));
    }

    if clean_id.starts_with("http://") || clean_id.starts_with("https://") {
      return Some(external_fetch_result(clean_id, "network".to_string(), None));
    }

    let context = self.compiler.context().clone();

    let mut direct_file_fallback: Option<PathBuf> = None;

    if let Some(path) = try_resolve_file_path(&raw_id, importer.as_deref(), &context.config.root) {
      let resolved = fs::canonicalize(&path).unwrap_or(path);
      if resolved.is_file() {
        let query = extract_query_string(&raw_id);
        let module_id = module_id_from_resolved_path_and_query(
          resolved.to_string_lossy().as_ref(),
          query.as_str(),
          &context.config.root,
        );

        match try_inlined_fetch_result(
          &context,
          &module_id,
          Some(&self.module_runner_transform_cache),
        ) {
          InlinedFetchAttempt::Inlined(inlined) => return Some(inlined),
          InlinedFetchAttempt::Bailout(reason) => {
            return Some(external_fetch_result(
              path_to_file_url(&resolved),
              module_type_by_path(&resolved),
              Some(reason),
            ));
          }
          InlinedFetchAttempt::NotApplicable => {}
        }

        if !has_query {
          return Some(external_fetch_result(
            path_to_file_url(&resolved),
            module_type_by_path(&resolved),
            None,
          ));
        }

        direct_file_fallback = Some(resolved);
      }
    }

    let resolver = Resolver::new();
    let resolve_base_dir = normalize_importer_path(importer.as_deref(), &context.config.root)
      .map(|importer_path| {
        if importer_path.is_file() {
          importer_path
            .parent()
            .unwrap_or(importer_path.as_path())
            .to_path_buf()
        } else {
          importer_path
        }
      })
      .unwrap_or_else(|| PathBuf::from(&context.config.root));

    let resolve_source = raw_id
      .split_once('#')
      .map(|(before, _)| before.to_string())
      .unwrap_or_else(|| raw_id.clone());

    if let Some(resolved) = resolver.resolve(
      &resolve_source,
      resolve_base_dir,
      &ResolveKind::Import,
      &ResolveOptions::default(),
      &context,
    ) {
      let resolved_path = clean_url(&resolved.resolved_path).to_string();

      if resolved.external {
        let resolved_type = if resolved_path.starts_with("node:") {
          "builtin".to_string()
        } else if resolved_path.starts_with("http://") || resolved_path.starts_with("https://") {
          "network".to_string()
        } else if resolved_path.ends_with(".cjs") {
          "commonjs".to_string()
        } else {
          "module".to_string()
        };

        return Some(external_fetch_result(resolved_path, resolved_type, None));
      }

      let query = stringify_query_pairs(&resolved.query);
      let module_id = module_id_from_resolved_path_and_query(
        &resolved.resolved_path,
        query.as_str(),
        &context.config.root,
      );

      match try_inlined_fetch_result(
        &context,
        &module_id,
        Some(&self.module_runner_transform_cache),
      ) {
        InlinedFetchAttempt::Inlined(inlined) => return Some(inlined),
        InlinedFetchAttempt::Bailout(reason) => {
          let resolved_file =
            fs::canonicalize(&resolved_path).unwrap_or(PathBuf::from(&resolved_path));
          if resolved_file.is_file() {
            return Some(external_fetch_result(
              path_to_file_url(&resolved_file),
              module_type_by_path(&resolved_file),
              Some(reason),
            ));
          }
        }
        InlinedFetchAttempt::NotApplicable => {}
      }

      let resolved_file = fs::canonicalize(&resolved_path).unwrap_or(PathBuf::from(&resolved_path));
      if resolved_file.is_file() {
        return Some(external_fetch_result(
          path_to_file_url(&resolved_file),
          module_type_by_path(&resolved_file),
          None,
        ));
      }
    }

    if let Some(fallback_file) = direct_file_fallback {
      return Some(external_fetch_result(
        path_to_file_url(&fallback_file),
        module_type_by_path(&fallback_file),
        None,
      ));
    }

    None
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
  pub fn write_metadata(&self, name: String, data: String, options: Option<JsApiMetadata>) {
    write_cache(self, name, data, options.map(|v| v.into()));
  }

  /// Read cache with name, return `undefined` if not exists
  #[napi]
  pub fn read_metadata(&self, name: String, options: Option<JsApiMetadata>) -> Option<String> {
    read_cache(self, name, options.map(|v| v.into()))
  }

  /// Read cache by scope, return Array of cache data, if not exists, return empty array
  #[napi]
  pub fn read_metadata_by_scope(&self, scope: String) -> Vec<String> {
    read_metadata_by_scope(self, scope)
  }
}

#[napi(object)]
pub struct JsApiMetadata {
  /// Scope of the cache, used different name, same scope will hit the same cache
  pub scope: Option<Vec<String>>,
  /// reference ModuleId of the cache, when the module is invalidated, the cache will be invalidated too
  pub refer: Option<Vec<String>>,
}

impl From<JsApiMetadata> for MetadataOption {
  fn from(value: JsApiMetadata) -> Self {
    MetadataOption {
      scope: value.scope,
      refer: value.refer,
    }
  }
}

fn read_cache(
  js_compiler: &JsCompiler,
  name: String,
  options: Option<MetadataOption>,
) -> Option<String> {
  let context = js_compiler.compiler.context();
  context.read_metadata(&name, options).map(|v| *v)
}

fn write_cache(
  js_compiler: &JsCompiler,
  name: String,
  data: String,
  options: Option<MetadataOption>,
) {
  let context = js_compiler.compiler.context();
  context.write_metadata(&name, data, options);
}

fn read_metadata_by_scope(js_compiler: &JsCompiler, scope: String) -> Vec<String> {
  let context = js_compiler.compiler.context();
  context.read_metadata_by_scope::<String>(&scope)
}

fn invalidate_module(js_compiler: &JsCompiler, module_id: String) {
  let raw_module_id = module_id.clone();
  let context = js_compiler.compiler.context();
  let module_id = ModuleId::new(&module_id, "", &context.config.root);

  context.invalidate_module(&module_id);
  invalidate_cached_runner_transform(
    &js_compiler.module_runner_transform_cache,
    &module_id.to_string(),
  );
  invalidate_cached_runner_transform(
    &js_compiler.module_runner_transform_cache,
    clean_url(&raw_module_id),
  );
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
