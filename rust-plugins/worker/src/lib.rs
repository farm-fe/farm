#![deny(clippy::all)]

mod codegen;
mod constants;
mod options;
mod resource_patch;

use std::sync::{Arc, Mutex};

use crate::{
  codegen::{
    compute_worker_entry_name, compute_worker_url, get_inline_worker_placeholder_code,
    get_worker_module_code,
  },
  constants::{WORKER_IMPORT_META_URL_RE, WORKER_OR_SHARED_WORKER_RE},
  options::Options,
  resource_patch::patch_inline_worker_resources,
};
use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::{ModuleId, ModuleType},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginAnalyzeDepsHookResultEntry, PluginHookContext,
    PluginLoadHookResult, PluginResolveHookParam, PluginTransformHookResult, ResolveKind,
  },
  serde_json::{self, Map, Value},
};
use farmfe_macro_plugin::farm_plugin;
use farmfe_utils::relative;
use regress::{Match, Regex as JsRegex};

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------

#[farm_plugin]
pub struct FarmfePluginWorker {
  options: Options,
  /// Workers discovered in `transform()` for `new Worker(new URL(…))` patterns.
  /// Importer module_id → Vec<(resolved_path, entry_name)>.
  transform_worker_deps: Mutex<std::collections::HashMap<String, Vec<(String, String)>>>,
}

impl FarmfePluginWorker {
  fn new(_config: &Config, options: String) -> Self {
    let options: Value = serde_json::from_str(&options).unwrap_or(Value::Object(Map::new()));
    let mut compiler_config = options
      .get("compilerConfig")
      .unwrap_or(&Value::Object(Map::new()))
      .clone();
    if let Value::Object(ref mut map) = compiler_config {
      if !map.contains_key("presetEnv") {
        map.insert("presetEnv".to_string(), Value::Bool(true));
      }
    }
    Self {
      options: Options {
        compiler_config: serde_json::from_value(compiler_config).ok(),
      },
      transform_worker_deps: Mutex::new(std::collections::HashMap::new()),
    }
  }
}

// ---------------------------------------------------------------------------
// Plugin hooks
// ---------------------------------------------------------------------------

impl Plugin for FarmfePluginWorker {
  fn name(&self) -> &str {
    "FarmfePluginWorker"
  }

  fn priority(&self) -> i32 {
    105
  }

  /// Handle `?worker`, `?sharedworker`, and `?worker&inline` virtual module imports.
  ///
  /// - `?worker&inline`: returns a placeholder module; `analyze_deps()` injects a
  ///   DynamicEntry dep so Farm's pipeline compiles the worker source.  `generate_end()`
  ///   base64-encodes the worker chunk and replaces the placeholder in the bundled
  ///   resources.
  /// - `?worker` / `?sharedworker`: returns a URL-based factory module.  `analyze_deps()`
  ///   injects a DynamicEntry dep, and `generate_end()` leaves the chunk as emitted.
  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    if JsRegex::new(WORKER_OR_SHARED_WORKER_RE)
      .unwrap()
      .find(&param.module_id)
      .is_none()
    {
      return Ok(None);
    }

    let is_inline = param.query.iter().any(|(k, _)| k == "inline");
    let is_url = param.query.iter().any(|(k, _)| k == "url");
    let compiler_config = self.options.compiler_config.as_ref().unwrap();

    if is_inline {
      // Don't compile now (build_worker would run on a Rayon thread — unsafe).
      // Instead return a placeholder module; `analyze_deps()` injects a DynamicEntry
      // dep so Farm's pipeline compiles the worker source, and `generate_end()` /
      // `update_finished()` base64-encodes the worker chunk and replaces the
      // placeholder in the bundled resources.
      let (entry_name, _) =
        worker_output_name(param.resolved_path, &param.module_id, compiler_config);
      let content = get_inline_worker_placeholder_code(&param.module_id, &entry_name);
      return Ok(Some(PluginLoadHookResult {
        content,
        module_type: ModuleType::Js,
        source_map: None,
      }));
    }

    // Non-inline: return URL wrapper; Farm's DynamicEntry pipeline (via analyze_deps)
    // compiles the worker source and emits it as-is.
    let (entry_name, _) =
      worker_output_name(param.resolved_path, &param.module_id, compiler_config);
    let worker_url = compute_worker_url(&entry_name, compiler_config);
    let content = get_worker_module_code(&param.module_id, &worker_url, is_url);
    Ok(Some(PluginLoadHookResult {
      content,
      module_type: ModuleType::Js,
      source_map: None,
    }))
  }

  /// Rewrite `new Worker(new URL("./path", import.meta.url))` expressions to use the
  /// deterministic output URL.  Records (importer → worker source) in
  /// `transform_worker_deps` so `analyze_deps()` can inject the DynamicEntry dep.
  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<PluginTransformHookResult>> {
    let matches: Vec<Match> = JsRegex::new(WORKER_IMPORT_META_URL_RE)
      .unwrap()
      .find_iter(&param.content)
      .collect();
    if matches.is_empty() {
      return Ok(None);
    }

    let compiler_config = self.options.compiler_config.as_ref().unwrap();
    let mut output = String::new();
    let mut last_end = 0;
    let mut found_workers = false;

    for m in &matches {
      let args = m.captures[0].as_ref().unwrap();
      let url_cap = m.captures[1].as_ref().unwrap();
      let arg_code = &param.content[args.start..args.end];
      let url_code = &param.content[url_cap.start..url_cap.end];

      if arg_code.contains('`') && arg_code.contains("${") {
        println!("new URL(url, import.meta.url) is not supported in dynamic template strings.");
        continue;
      }

      let url_str = &url_code[1..url_code.len() - 1]; // strip surrounding quotes
      let full_path = if url_str.starts_with('/') {
        context.config.root.clone() + url_str
      } else {
        context
          .plugin_driver
          .resolve(
            &PluginResolveHookParam {
              source: url_str.to_string(),
              importer: Some(param.module_id.clone().into()),
              kind: ResolveKind::Import,
            },
            context,
            &PluginHookContext::default(),
          )
          .unwrap()
          .unwrap()
          .resolved_path
      };

      // Use the same virtual module_id formula as load() for a consistent output name.
      let rel_path = relative(&context.config.root, &full_path);
      let virtual_id = format!("{rel_path}?worker");
      let (entry_name, _) = worker_output_name(&full_path, &virtual_id, compiler_config);
      let worker_url = compute_worker_url(&entry_name, compiler_config);

      self
        .transform_worker_deps
        .lock()
        .unwrap()
        .entry(param.module_id.to_string())
        .or_default()
        .push((full_path.clone(), entry_name.clone()));
      output.push_str(&param.content[last_end..args.start]);
      // Replace the `new URL(...)` arg with the deterministic URL string.
      // The worker script is emitted as a classic script, so it does not need
      // `{ type: 'module' }`.
      output.push_str(&arg_code.replace(url_code, &format!(r#""{worker_url}""#)));
      last_end = args.end;
      found_workers = true;

      // Watch the worker source so HMR triggers on changes to it.
      let _ = context.add_watch_files(
        ModuleId::new(param.resolved_path, "", &context.config.root),
        vec![ModuleId::new(&full_path, "", &context.config.root)],
      );
    }

    if !found_workers {
      return Ok(None);
    }
    output.push_str(&param.content[last_end..]);
    Ok(Some(PluginTransformHookResult {
      content: output,
      module_type: Some(param.module_type.clone()),
      ..Default::default()
    }))
  }

  /// Inject `DynamicEntry` deps for worker virtual modules and for importers that
  /// contain `new Worker(new URL(...))` patterns discovered by `transform()`.
  ///
  /// - `?worker&inline` modules → DynamicEntry; `generate_end()` base64-encodes
  ///   the worker chunk and patches the placeholder in the host resource.
  /// - `?worker` / `?sharedworker` modules → DynamicEntry.
  /// - Importer modules with transform-discovered workers → DynamicEntry for each.
  fn analyze_deps(
    &self,
    param: &mut PluginAnalyzeDepsHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let module_id_str = param.module.id.to_string();
    let compiler_config = self.options.compiler_config.as_ref().unwrap();

    // Case 1: ?worker, ?sharedworker, or ?worker&inline virtual modules.
    // Inject a DynamicEntry dep so Farm's main pipeline compiles the worker source
    // as a separate chunk.
    if JsRegex::new(WORKER_OR_SHARED_WORKER_RE)
      .unwrap()
      .find(&module_id_str)
      .is_some()
    {
      let resolved_path = param.module.id.resolved_path(&context.config.root);
      let (entry_name, _) = worker_output_name(&resolved_path, &module_id_str, compiler_config);

      param.deps.push(PluginAnalyzeDepsHookResultEntry {
        source: resolved_path,
        kind: ResolveKind::DynamicEntry {
          name: entry_name.clone(),
          output_filename: None,
        },
      });
      return Ok(Some(()));
    }

    // Case 2: modules whose transform() discovered new Worker(new URL(...)) patterns.
    // Drain the collected worker entries and inject DynamicEntry deps for each.
    let mut deps_map = self.transform_worker_deps.lock().unwrap();
    if let Some(entries) = deps_map.remove(&module_id_str) {
      drop(deps_map);
      if entries.is_empty() {
        return Ok(None);
      }
      for (resolved_path, entry_name) in entries {
        param.deps.push(PluginAnalyzeDepsHookResultEntry {
          source: resolved_path,
          kind: ResolveKind::DynamicEntry {
            name: entry_name.clone(),
            output_filename: None,
          },
        });
      }
      return Ok(Some(()));
    }

    Ok(None)
  }

  /// Patch worker resources after Farm's generate phase.
  ///
  /// Inline workers: base64-encoded worker chunk replaces the placeholder string
  /// embedded by `load()` in the containing JS resource.
  fn generate_end(
    &self,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    patch_inline_worker_resources(context);
    Ok(None)
  }

  /// After an HMR update: re-patch inline worker resources.
  fn update_finished(
    &self,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    patch_inline_worker_resources(context);
    Ok(None)
  }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

/// Return `(entry_name, full_output_filename)` for a worker source file.
///
/// `entry_name` is the bare name used by `compute_worker_url()`;
/// `full_output_filename` is the final filename on disk / served by the dev server.
fn worker_output_name(
  resolved_path: &str,
  module_id: &str,
  compiler_config: &farmfe_core::config::Config,
) -> (String, String) {
  let entry_name = compute_worker_entry_name(resolved_path, module_id, compiler_config);
  let filename = format!("{entry_name}.js");
  (entry_name, filename)
}
