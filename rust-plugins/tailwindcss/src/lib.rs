#![deny(clippy::all)]

use std::collections::{HashMap, HashSet};
use std::path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::CompilationError,
  module::{Module, ModuleId, ModuleType},
  plugin::{
    Plugin, PluginTransformHookParam, PluginTransformHookResult, PluginUpdateModulesHookParam,
    UpdateType, hooks::freeze_module::PluginFreezeModuleHookParam,
  },
  serde_json,
};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::css::parse_css_stylesheet;
use farmfe_toolkit::lazy_static::lazy_static;
use farmfe_toolkit::regex::Regex;
use farmfe_toolkit::{swc_css_prefixer, swc_css_visit::VisitMutWith};

use farmfe_ecosystem_tailwindcss::TailwindConfig;
use farmfe_ecosystem_tailwindcss_node::compile::{self, CompileOptions};
use tailwindcss_oxide::{ChangedContent, Scanner};

const PKG_NAME: &str = "@farmfe/plugin-tailwindcss";

#[derive(serde::Serialize, serde::Deserialize)]
struct TailwindPluginCache {
  version: u32,
  candidates: Vec<String>,
  candidate_sources: Vec<String>,
  css_roots: Vec<String>,
}

fn plugin_cache_error(error: impl std::fmt::Display) -> farmfe_core::error::CompilationError {
  farmfe_core::error::CompilationError::GenericError(format!(
    "[{PKG_NAME}] Persistent plugin state: {error}"
  ))
}

fn compilation_error(
  stage: &str,
  error: impl std::fmt::Display,
) -> farmfe_core::error::CompilationError {
  farmfe_core::error::CompilationError::GenericError(format!("[TAILWIND_COMPILE] {stage}: {error}"))
}

lazy_static! {
    /// Regex matching CSS file extensions.
    static ref CSS_EXT_RE: Regex = Regex::new(r"\.css(\?|$)").unwrap();

    /// Regex matching file extensions that may contain TailwindCSS candidates.
    static ref CANDIDATE_EXT_RE: Regex = Regex::new(
        r"\.(js|jsx|ts|tsx|vue|svelte|html|mdx|md|astro|php|blade\.php|twig|erb|hbs|liquid|pug|slim|haml)(\?|$)"
    ).unwrap();
}

/// Options parsed from the JSON string provided at plugin creation.
#[derive(serde::Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TailwindCSSOptions {
  /// Extra paths / globs to scan for candidates.
  #[serde(default)]
  #[allow(dead_code)]
  content: Vec<String>,
  /// Optional Tailwind config payload passed through directly to the Rust core crate.
  #[serde(default)]
  config: Option<serde_json::Value>,
  /// Emit flattened color aliases in the base layer, using Rust theme data.
  #[serde(default)]
  color_variables: bool,
}

/// Returns `true` if `content` contains any TailwindCSS root directive.
///
/// We look for `@tailwind`, `@apply ` (note the space — guards against
/// `@applyXyz` identifiers), and quoted `@import "tailwindcss"` /
/// `@import 'tailwindcss'`. Each check is a single-pass substring search;
/// the early-exit on `@tailwind` covers the most common case first.
fn has_tailwind_directive(content: &str) -> bool {
  if content.contains("@tailwind") {
    return true;
  }
  if content.contains("@apply ") {
    return true;
  }
  content.contains("@import \"tailwindcss\"") || content.contains("@import 'tailwindcss'")
}

#[farm_plugin]
pub struct FarmPluginTailwindCSS {
  _options: TailwindCSSOptions,
  options_error: Option<String>,
  /// Set of all candidates collected from scanned source files.
  candidates: Arc<Mutex<HashSet<String>>>,
  /// Set of module IDs that contributed candidates.
  candidate_sources: Arc<Mutex<HashSet<String>>>,
  /// Transform output provenance, never a claimed raw-file digest.
  candidate_producers: Mutex<HashMap<String, (u64, String)>>,
  /// Original (untransformed) CSS content for CSS root files that need
  /// deferred TailwindCSS compilation. The actual compile-and-inline pass
  /// runs in `freeze_module`, by which time the candidate set is complete.
  pending_css: Arc<Mutex<HashMap<String, String>>>,
  /// Active roots must also regenerate when previously unseen lazy code arrives.
  css_roots: Arc<Mutex<HashSet<String>>>,
  state_ready: AtomicBool,
}

impl FarmPluginTailwindCSS {
  fn controlled_reader(
    context: &Arc<CompilationContext>,
    reader: String,
  ) -> Box<compile::FileReader> {
    let context = context.clone();
    let generation = context.source_observations.generation();
    Box::new(move |path| {
      if context.source_observations.generation() != generation {
        return Err(std::io::Error::other(
          "stale Tailwind input reader generation",
        ));
      }
      context.source_observations.read_file(&reader, path)
    })
  }

  pub fn new(_config: &Config, options: String) -> Self {
    let parsed = serde_json::from_str::<TailwindCSSOptions>(&options)
      .map_err(|error| error.to_string())
      .and_then(|options| {
        if options
          .config
          .as_ref()
          .is_some_and(|config| !config.is_object())
        {
          Err("config must be an object".into())
        } else {
          Ok(options)
        }
      });
    let (parsed, options_error) = match parsed {
      Ok(options) => (options, None),
      Err(error) => (TailwindCSSOptions::default(), Some(error)),
    };

    Self {
      _options: parsed,
      options_error,
      candidates: Arc::new(Mutex::new(HashSet::new())),
      candidate_sources: Arc::new(Mutex::new(HashSet::new())),
      candidate_producers: Mutex::new(HashMap::new()),
      pending_css: Arc::new(Mutex::new(HashMap::new())),
      css_roots: Arc::new(Mutex::new(HashSet::new())),
      state_ready: AtomicBool::new(false),
    }
  }

  /// Scan `content` for TailwindCSS candidate class names using oxide scanner.
  fn scan_candidates(&self, content: &str, extension: &str) -> Vec<String> {
    let mut scanner = Scanner::new(vec![]);
    scanner.scan_content(vec![ChangedContent::Content(
      content.to_string(),
      extension.to_string(),
    )])
  }

  /// Check whether `id` looks like a CSS root file that should trigger
  /// TailwindCSS generation.
  fn is_css_root_file(id: &str) -> bool {
    CSS_EXT_RE.is_match(id) && !id.contains("node_modules")
  }

  /// Check whether `id` is a non-CSS source file that should be scanned for
  /// Tailwind class candidates.
  fn is_candidate_source_file(id: &str) -> bool {
    CANDIDATE_EXT_RE.is_match(id) && !id.contains("node_modules")
  }

  /// Get the file extension from a module id (stripping query strings).
  fn get_extension(id: &str) -> &str {
    let filename = id.split('?').next().unwrap_or(id);
    filename.rsplit('.').next().unwrap_or("")
  }

  fn register_candidate_source(&self, id: &str, content: &str) -> farmfe_core::error::Result<()> {
    let new_candidates = self.scan_candidates(content, Self::get_extension(id));
    self
      .candidates
      .lock()
      .map_err(|error| CompilationError::GenericError(error.to_string()))?
      .extend(new_candidates);
    // An initially empty source may acquire its first class during an update.
    self
      .candidate_sources
      .lock()
      .map_err(|error| CompilationError::GenericError(error.to_string()))?
      .insert(id.to_string());
    Ok(())
  }
}

impl Plugin for FarmPluginTailwindCSS {
  fn name(&self) -> &str {
    "FarmPluginTailwindCSS"
  }

  fn priority(&self) -> i32 {
    101
  }

  fn build_start(
    &self,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if let Some(error) = &self.options_error {
      return Err(farmfe_core::error::CompilationError::GenericError(format!(
        "[TAILWIND_OPTIONS] {error}"
      )));
    }
    Ok(None)
  }

  fn plugin_cache_loaded(
    &self,
    cache: &Vec<u8>,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let state: TailwindPluginCache = serde_json::from_slice(cache).map_err(plugin_cache_error)?;
    if state.version != 1 {
      return Err(plugin_cache_error(format!(
        "unsupported cache version {}; clear the build cache and retry",
        state.version
      )));
    }
    let mut candidates = self.candidates.lock().map_err(plugin_cache_error)?;
    let mut sources = self.candidate_sources.lock().map_err(plugin_cache_error)?;
    let mut roots = self.css_roots.lock().map_err(plugin_cache_error)?;
    *candidates = state.candidates.into_iter().collect();
    *sources = state.candidate_sources.into_iter().collect();
    *roots = state.css_roots.into_iter().collect();
    self.state_ready.store(true, Ordering::Release);
    Ok(Some(()))
  }

  fn write_plugin_cache(
    &self,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<Vec<u8>>> {
    let mut state = {
      let candidates = self.candidates.lock().map_err(plugin_cache_error)?;
      let sources = self.candidate_sources.lock().map_err(plugin_cache_error)?;
      let roots = self.css_roots.lock().map_err(plugin_cache_error)?;
      TailwindPluginCache {
        version: 1,
        candidates: candidates.iter().cloned().collect(),
        candidate_sources: sources.iter().cloned().collect(),
        css_roots: roots.iter().cloned().collect(),
      }
    };
    state.candidates.sort_unstable();
    state.candidate_sources.sort_unstable();
    state.css_roots.sort_unstable();
    let bytes = serde_json::to_vec(&state).map_err(plugin_cache_error)?;
    self.state_ready.store(true, Ordering::Release);
    Ok(Some(bytes))
  }

  fn config(&self, config: &mut Config) -> farmfe_core::error::Result<Option<()>> {
    // Ensure .css is in the resolve extensions
    if config.resolve.extensions.iter().all(|e| e != "css") {
      config.resolve.extensions.push("css".to_string());
    }
    Ok(Some(()))
  }

  fn handle_persistent_cached_module(
    &self,
    module: &Module,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<bool>> {
    let resolved_path = module.id.resolved_path(&context.config.root);
    if Self::is_candidate_source_file(&resolved_path) {
      // Cached Vue/Svelte modules can contain compiled script rather than their
      // original templates, so prefer the physical source when available.
      let content =
        context.source_observations
          .read_file(&module.id.to_string(), path::Path::new(&resolved_path))
          .ok().and_then(|bytes| String::from_utf8(bytes).ok())
          .unwrap_or_else(|| module.content.to_string());
      self.register_candidate_source(&resolved_path, &content)?;
    }
    // A compiled module cannot restore missing scanner/shared-root state.
    if !self.state_ready.load(Ordering::Acquire)
      && (Self::is_candidate_source_file(&resolved_path) || Self::is_css_root_file(&resolved_path))
    {
      return Ok(Some(true));
    }
    // Let other plugins decide whether this otherwise valid module cache expires.
    Ok(None)
  }

  fn transform(
    &self,
    param: &PluginTransformHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<PluginTransformHookResult>> {
    let resolved_path = param.resolved_path;

    // ── Step 1: Scan non-CSS source files for candidates ────────────
    if Self::is_candidate_source_file(resolved_path) {
      self.register_candidate_source(resolved_path, &param.content)?;
      self
        .candidate_producers
        .lock()
        .map_err(plugin_cache_error)?
        .insert(
          resolved_path.to_string(),
          (context.source_observations.generation(), param.module_id.clone()),
        );

      // Pass through — we don't modify the source.
      return Ok(None);
    }

    // ── Step 2: Defer CSS-root compilation until `freeze_module` ─────
    //
    // Compilation has to happen *after* every candidate-source file has
    // been transformed, otherwise the candidate set is incomplete and
    // `@tailwind utilities` would expand to nothing. We record the
    // module's original content here so the deferred pass can re-compile
    // it once the module graph is built.
    if param.module_type != ModuleType::Css || !Self::is_css_root_file(resolved_path) {
      return Ok(None);
    }

    if !has_tailwind_directive(&param.content) {
      self.css_roots.lock().unwrap().remove(resolved_path);
      return Ok(None);
    }

    let base = path::Path::new(resolved_path)
      .parent()
      .unwrap_or(path::Path::new("."))
      .to_string_lossy()
      .into_owned();
    let prepared = compile::prepare_stylesheet(
      &param.content,
      &CompileOptions {
        read_file: Some(Self::controlled_reader(context, param.module_id.clone())),
        base,
        from: context
          .sourcemap_enabled(&param.module_id)
          .then(|| resolved_path.to_string()),
        should_rewrite_urls: true,
        ..Default::default()
      },
    )
    .map_err(|error| compilation_error(resolved_path, error))?;
    if !prepared.dependencies.is_empty() {
      context.add_watch_files(
        ModuleId::new(resolved_path, "", &context.config.root),
        prepared
          .dependencies
          .iter()
          .map(|dependency| ModuleId::new(dependency, "", &context.config.root))
          .collect(),
      )?;
    }
    let source_map = if let Some(map) = prepared.source_map {
      let mut bytes = Vec::new();
      map
        .to_writer(&mut bytes)
        .map_err(|error| compilation_error(resolved_path, error))?;
      Some(String::from_utf8(bytes).map_err(|error| compilation_error(resolved_path, error))?)
    } else {
      None
    };

    self
      .css_roots
      .lock()
      .unwrap()
      .insert(resolved_path.to_string());

    self
      .pending_css
      .lock()
      .unwrap()
      .insert(param.module_id.clone(), prepared.content.clone());

    // Also scan the CSS itself for candidates inside string literals / @apply
    // values — Tailwind's official scanner picks these up.
    let new_candidates = self.scan_candidates(&param.content, "css");
    if !new_candidates.is_empty() {
      let mut candidates = self.candidates.lock().unwrap();
      for c in new_candidates {
        candidates.insert(c);
      }
    }

    Ok(Some(PluginTransformHookResult {
      content: prepared.content,
      source_map,
      module_type: None,
      ignore_previous_source_map: false,
    }))
  }

  fn update_modules(
    &self,
    params: &mut PluginUpdateModulesHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let mut roots = {
      let mut roots = self.css_roots.lock().unwrap();
      roots.retain(|root| {
        let root = ModuleId::new(root, "", &context.config.root);
        !params.paths.iter().any(|(path, kind)| {
          matches!(kind, UpdateType::Removed)
            && ModuleId::from_resolved_path_with_query(path, &context.config.root) == root
        })
      });
      roots.iter().cloned().collect::<Vec<_>>()
    };
    roots.sort();
    let candidate_update = params.paths.iter().any(|(path, _)| {
      let path = path.split('?').next().unwrap_or(path);
      path.ends_with(".farm_dynamic_import_virtual_module") || Self::is_candidate_source_file(path)
    });
    let graph = context.module_graph.read();
    let mut changed = false;
    for root in roots {
      let root_id = ModuleId::new(&root, "", &context.config.root);
      let already_updated = params.paths.iter().any(|(path, _)| {
        ModuleId::from_resolved_path_with_query(path, &context.config.root) == root_id
      });
      if (candidate_update || already_updated) && graph.has_module(&root_id) {
        // An unchanged CSS timestamp/hash cannot account for newly discovered
        // candidates. Ensure transform supplies fresh pending CSS before freeze.
        context
          .cache_manager
          .module_cache
          .invalidate_cache(&root_id);
        if !already_updated {
          params.paths.push((root, UpdateType::Updated));
        }
        changed = true;
      }
    }
    Ok(changed.then_some(()))
  }

  fn freeze_module(
    &self,
    param: &mut PluginFreezeModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if param.module.module_type != ModuleType::Css {
      return Ok(None);
    }

    let resolved_path = param.module.id.resolved_path(&context.config.root);

    let original = {
      let mut guard = self.pending_css.lock().unwrap();
      // Remove so re-compilation never runs twice for the same module.
      guard.remove(&param.module.id.to_string())
    };

    let Some(original) = original else {
      return Ok(None);
    };

    let base = path::Path::new(&resolved_path)
      .parent()
      .unwrap_or(path::Path::new("."))
      .to_string_lossy()
      .into_owned();

    let sourcemap_enabled = context.sourcemap_enabled(&param.module.id.to_string());

    let compiler_result = compile::compile(
      &original,
      CompileOptions {
        read_file: Some(Self::controlled_reader(
          context,
          param.module.id.to_string(),
        )),
        base,
        from: if sourcemap_enabled {
          Some(resolved_path.clone())
        } else {
          None
        },
        should_rewrite_urls: true,
        config: self
          ._options
          .config
          .as_ref()
          .cloned()
          .map(TailwindConfig::new),
        on_dependency: Box::new(move |_dep| {
          // Dependencies are tracked through the module graph.
        }),
        ..Default::default()
      },
    );

    let mut compiler = match compiler_result {
      Ok(c) => c,
      Err(e) => {
        return Err(compilation_error(&resolved_path, e));
      }
    };

    if !compiler.features.has_any_output_feature() {
      return Ok(None);
    }

    let candidates: Vec<String> = self.candidates.lock().unwrap().iter().cloned().collect();

    // Only current-generation transform results carry controlled provenance.
    // Restored candidates without it retain the legacy discovery path.
    for (generation, producer) in self
      .candidate_producers
      .lock()
      .map_err(plugin_cache_error)?
      .values()
    {
      if *generation == context.source_observations.generation() {
        context
          .source_observations
          .consume_derived(*generation, &param.module.id.to_string(), producer)
          .map_err(|error| compilation_error(&resolved_path, error))?;
      }
    }

    compiler.set_color_variables(self._options.color_variables);
    let css = compiler.build(&candidates);

    // Re-parse the generated CSS so the module AST matches the new content.
    let module_id_str = param.module.id.to_string();
    let mut parsed = match parse_css_stylesheet(&module_id_str, Arc::new(css.clone())) {
      Ok(p) => p,
      Err(e) => {
        return Err(compilation_error(&resolved_path, e));
      }
    };

    // The core's process_module pass ran before this deferred replacement.
    // Apply the same Rust prefixer to the final AST, preserving authored spans.
    if let Some(options) = &context.config.css.prefixer {
      parsed.ast.visit_mut_with(&mut swc_css_prefixer::prefixer(
        swc_css_prefixer::options::Options {
          env: options.targets.clone(),
        },
      ));
    }

    // Register watch edges so changes to candidate sources invalidate the
    // CSS root module on subsequent rebuilds.
    let sources: Vec<String> = self
      .candidate_sources
      .lock()
      .unwrap()
      .iter()
      .cloned()
      .collect();
    for source in &sources {
      context.add_watch_files(
        param.module.id.clone(),
        vec![ModuleId::new(source, "", &context.config.root)],
      )?;
    }

    if let Some(map) = compiler.build_source_map() {
      param.module.source_map_chain.push(Arc::new(map));
    }
    context
      .meta
      .set_module_source_map(&param.module.id, parsed.source_map);
    param.module.meta.as_css_mut().set_ast(parsed.ast);
    param.module.content = Arc::new(css);

    Ok(Some(()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cached_candidates_survive_restart_and_a_new_source_update() {
    use farmfe_core::module::{Module, ModuleMetaData, meta_data::css::CssModuleMetaData};
    let mut config = Config::default();
    config.root = std::env::temp_dir().to_string_lossy().into_owned();
    config.persistent_cache =
      Box::new(farmfe_core::config::persistent_cache::PersistentCacheConfig::Bool(false));
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    let root_id = ModuleId::new("farm-tailwind-restart.css", "", &context.config.root);
    let root = root_id.resolved_path(&context.config.root);
    let mut module = Module::new(root_id.clone());
    module.module_type = ModuleType::Css;
    let raw = "@tailwind utilities;";
    module.content = Arc::new(raw.into());
    module.meta = Box::new(ModuleMetaData::Css(Box::new(CssModuleMetaData {
      ast: parse_css_stylesheet(&root, Arc::new(raw.into()))
        .unwrap()
        .ast,
      comments: Default::default(),
      custom: Default::default(),
    })));
    context.module_graph.write().add_module(module.clone());
    let cached_source = Module::new(ModuleId::new("before.tsx", "", &context.config.root));
    let transform =
      |plugin: &FarmPluginTailwindCSS, id: &str, content: &str, module_type: ModuleType| {
        plugin
          .transform(
            &PluginTransformHookParam {
              resolved_path: id,
              content: content.into(),
              module_type,
              module_id: ModuleId::new(id, "", &context.config.root).to_string(),
              query: vec![],
              meta: Default::default(),
              source_map_chain: vec![],
            },
            &context,
          )
          .unwrap();
      };
    let before = make_plugin();
    assert_eq!(
      before
        .handle_persistent_cached_module(&cached_source, &context)
        .unwrap(),
      Some(true),
      "Rebuild cached sources when plugin state is absent"
    );
    transform(
      &before,
      "before.tsx",
      "<div className=\"grid\" />",
      ModuleType::Tsx,
    );
    transform(&before, &root, raw, ModuleType::Css);
    before
      .freeze_module(
        &mut PluginFreezeModuleHookParam {
          module: &mut module,
          resolved_deps: vec![],
        },
        &context,
      )
      .unwrap();
    assert!(module.content.contains(".grid"));
    assert_eq!(
      context.source_observations.producers(&root_id.to_string()),
      vec![ModuleId::new("before.tsx", "", &context.config.root).to_string()]
    );
    let cache = before
      .write_plugin_cache(&context)
      .unwrap()
      .expect("Persist Tailwind candidates and active roots for a new compiler process");
    assert_eq!(
      before
        .handle_persistent_cached_module(&cached_source, &context)
        .unwrap(),
      None
    );
    let dependency = Module::new(ModuleId::new(
      "node_modules/library/view.tsx",
      "",
      &context.config.root,
    ));
    assert_eq!(
      make_plugin()
        .handle_persistent_cached_module(&dependency, &context)
        .unwrap(),
      None
    );
    let after = make_plugin();
    after.plugin_cache_loaded(&cache, &context).unwrap();
    assert_eq!(
      after
        .handle_persistent_cached_module(&cached_source, &context)
        .unwrap(),
      None,
      "Allow cache reuse after state restoration without suppressing other plugins"
    );
    assert_eq!(after.write_plugin_cache(&context).unwrap().unwrap(), cache);
    assert_eq!(
      *after.candidate_sources.lock().unwrap(),
      *before.candidate_sources.lock().unwrap()
    );
    let mut update = PluginUpdateModulesHookParam {
      paths: vec![("after.tsx".into(), UpdateType::Updated)],
    };
    after.update_modules(&mut update, &context).unwrap();
    assert!(
      update.paths.iter().any(|(path, _)| path == &root),
      "Restore the active CSS root before the first warm source edit"
    );
    transform(
      &after,
      "after.tsx",
      "<div className=\"flex\" />",
      ModuleType::Tsx,
    );
    transform(&after, &root, raw, ModuleType::Css);
    after
      .freeze_module(
        &mut PluginFreezeModuleHookParam {
          module: &mut module,
          resolved_deps: vec![],
        },
        &context,
      )
      .unwrap();
    assert!(
      module.content.contains(".grid"),
      "Keep utilities from cached source modules"
    );
    assert!(
      module.content.contains(".flex"),
      "Generate utilities from the newly transformed module"
    );
  }

  #[test]
  fn malformed_or_unknown_tailwind_cache_is_an_error() {
    let context = Arc::new(CompilationContext::new(Config::default(), vec![]).unwrap());
    let plugin = make_plugin();
    assert!(
      plugin
        .plugin_cache_loaded(&b"not json".to_vec(), &context)
        .is_err()
    );
    let cache =
      br#"{"version":999,"candidates":[],"candidate_sources":[],"css_roots":[]}"#.to_vec();
    assert!(plugin.plugin_cache_loaded(&cache, &context).is_err());
  }

  #[test]
  fn lazy_sources_update_an_existing_tailwind_root_after_its_first_freeze() {
    use farmfe_core::module::{Module, ModuleMetaData, meta_data::css::CssModuleMetaData};
    let mut config = Config::default();
    config.root = std::env::temp_dir().to_string_lossy().into_owned();
    let root = path::Path::new(&config.root)
      .join("farm-tailwind-lazy.css")
      .to_string_lossy()
      .into_owned();
    let id = ModuleId::new(&root, "", &config.root);
    let root = id.resolved_path(&config.root);
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    let plugin = make_plugin();
    let content = "@tailwind utilities;".to_string();
    let parsed = parse_css_stylesheet(&root, Arc::new(content.clone())).unwrap();
    let mut module = Module::new(id.clone());
    module.module_type = ModuleType::Css;
    module.content = Arc::new(content.clone());
    module.meta = Box::new(ModuleMetaData::Css(Box::new(CssModuleMetaData {
      ast: parsed.ast,
      comments: Default::default(),
      custom: Default::default(),
    })));
    context.module_graph.write().add_module(module.clone());
    plugin
      .transform(
        &PluginTransformHookParam {
          resolved_path: &root,
          content,
          module_type: ModuleType::Css,
          module_id: id.to_string(),
          query: vec![],
          meta: Default::default(),
          source_map_chain: vec![],
        },
        &context,
      )
      .unwrap();
    plugin
      .freeze_module(
        &mut PluginFreezeModuleHookParam {
          module: &mut module,
          resolved_deps: vec![],
        },
        &context,
      )
      .unwrap();
    assert!(plugin.pending_css.lock().unwrap().is_empty());
    context.cache_manager.module_cache.set_cache(
      id.clone(),
      farmfe_core::cache::module_cache::CachedModule {
        module: module.clone(),
        dependencies: vec![],
        watch_dependencies: vec![],
        is_expired: false,
      },
    );
    assert!(context.cache_manager.module_cache.has_cache(&id));
    // Initial compilation persists the result before any lazy request arrives.
    context.cache_manager.module_cache.write_cache();
    let page = path::Path::new(&context.config.root)
      .join("new.tsx.farm_dynamic_import_virtual_module")
      .to_string_lossy()
      .into_owned();
    let mut update = PluginUpdateModulesHookParam {
      paths: vec![(page.clone(), UpdateType::Updated)],
    };
    plugin.update_modules(&mut update, &context).unwrap();
    assert!(
      !context.cache_manager.module_cache.has_cache(&id),
      "A newly discovered candidate must bypass the unchanged root's timestamp and hash cache"
    );
    assert_eq!(
      update
        .paths
        .iter()
        .map(|(path, _)| path)
        .collect::<Vec<_>>(),
      vec![&page, &root]
    );
    plugin.update_modules(&mut update, &context).unwrap();
    assert_eq!(
      update.paths.len(),
      2,
      "Do not append an already updated root twice"
    );
    let mut removal = PluginUpdateModulesHookParam {
      paths: vec![
        (root.clone(), UpdateType::Removed),
        (page, UpdateType::Updated),
      ],
    };
    plugin.update_modules(&mut removal, &context).unwrap();
    assert_eq!(
      removal.paths.len(),
      2,
      "Do not resurrect a removed stylesheet"
    );
    assert!(plugin.css_roots.lock().unwrap().is_empty());
  }

  /// Helper for unit tests that construct the plugin without going through
  /// the JSON-options entry point. Keeps tests insulated from new internal
  /// fields.
  fn make_plugin() -> FarmPluginTailwindCSS {
    FarmPluginTailwindCSS {
      _options: TailwindCSSOptions::default(),
      options_error: None,
      candidates: Arc::new(Mutex::new(HashSet::new())),
      candidate_sources: Arc::new(Mutex::new(HashSet::new())),
      candidate_producers: Mutex::new(HashMap::new()),
      pending_css: Arc::new(Mutex::new(HashMap::new())),
      css_roots: Arc::new(Mutex::new(HashSet::new())),
      state_ready: AtomicBool::new(false),
    }
  }

  #[test]
  fn scan_candidates_basic() {
    let plugin = make_plugin();

    let content = r#"<div class="bg-red-500 text-white p-4 hover:bg-blue-500"></div>"#;
    let candidates = plugin.scan_candidates(content, "html");
    assert!(!candidates.is_empty());
    // Check that at least some expected candidates are found
    assert!(candidates.iter().any(|c| c.contains("bg-red")));
  }

  #[test]
  fn scan_candidates_ignores_urls() {
    let plugin = make_plugin();

    let content = r#"const url = "https://example.com";"#;
    let candidates = plugin.scan_candidates(content, "js");
    // Should not contain URL-like candidates
    for c in &candidates {
      assert!(!c.contains("://"), "Found URL-like candidate: {c}");
    }
  }

  #[test]
  fn is_css_root_file_works() {
    assert!(FarmPluginTailwindCSS::is_css_root_file("/src/app.css"));
    assert!(FarmPluginTailwindCSS::is_css_root_file(
      "/src/styles.css?v=1"
    ));
    assert!(!FarmPluginTailwindCSS::is_css_root_file(
      "/node_modules/lib/style.css"
    ));
    assert!(!FarmPluginTailwindCSS::is_css_root_file("/src/app.js"));
  }

  #[test]
  fn is_candidate_source_file_works() {
    assert!(FarmPluginTailwindCSS::is_candidate_source_file(
      "/src/app.tsx"
    ));
    assert!(FarmPluginTailwindCSS::is_candidate_source_file(
      "/src/app.vue?lang.ts"
    ));
    assert!(!FarmPluginTailwindCSS::is_candidate_source_file(
      "/node_modules/pkg/index.js"
    ));
    assert!(!FarmPluginTailwindCSS::is_candidate_source_file(
      "/src/styles.css"
    ));
  }

  #[test]
  fn get_extension_works() {
    assert_eq!(FarmPluginTailwindCSS::get_extension("/src/app.css"), "css");
    assert_eq!(
      FarmPluginTailwindCSS::get_extension("/src/app.css?v=1"),
      "css"
    );
    assert_eq!(FarmPluginTailwindCSS::get_extension("/src/app.tsx"), "tsx");
  }

  #[test]
  fn plugin_name_and_priority() {
    let config = Config::default();
    let plugin = FarmPluginTailwindCSS::new(&config, "{}".to_string());
    assert_eq!(plugin.name(), "FarmPluginTailwindCSS");
    assert_eq!(plugin.priority(), 101);
  }

  #[test]
  fn has_tailwind_directive_detects_all_forms() {
    assert!(has_tailwind_directive("@tailwind utilities;"));
    assert!(has_tailwind_directive(
      "@tailwind base;\n@tailwind components;"
    ));
    assert!(has_tailwind_directive(".btn { @apply px-4; }"));
    assert!(has_tailwind_directive("@import \"tailwindcss\";"));
    assert!(has_tailwind_directive("@import 'tailwindcss';"));
    assert!(!has_tailwind_directive(".btn { color: red; }"));
    assert!(!has_tailwind_directive("@import \"./theme.css\";"));
    // Guard against false positives on identifiers that share a prefix.
    assert!(!has_tailwind_directive(".btn { @applyXyz: 1; }"));
  }
}

#[cfg(test)]
mod persistent_cache_tests {
  use super::*;
  use farmfe_core::module::Module;

  #[test]
  fn cached_candidate_source_restores_utility_scanning() {
    let context = Arc::new(CompilationContext::default());
    let plugin = FarmPluginTailwindCSS::new(&context.config, "{}".to_string());
    let mut module = Module::new("src/App.tsx".into());
    module.module_type = ModuleType::Tsx;
    module.content =
      Arc::new("export const app = <div className=\"bg-red-500 p-4\" />;".to_string());
    plugin
      .handle_persistent_cached_module(&module, &context)
      .unwrap();
    assert!(plugin.candidates.lock().unwrap().contains("bg-red-500"));
    assert!(
      plugin
        .candidate_sources
        .lock()
        .unwrap()
        .contains(&module.id.resolved_path(&context.config.root))
    );

    let mut css = Module::new("index.css".into());
    css.module_type = ModuleType::Css;
    css.content = Arc::new("@theme { --spacing: 0.25rem; } @tailwind utilities;".to_string());
    let resolved_path = css.id.resolved_path(&context.config.root);
    plugin
      .transform(
        &PluginTransformHookParam {
          module_id: css.id.to_string(),
          content: css.content.to_string(),
          module_type: ModuleType::Css,
          resolved_path: &resolved_path,
          query: vec![],
          meta: Default::default(),
          source_map_chain: vec![],
        },
        &context,
      )
      .unwrap();
    let parsed = parse_css_stylesheet(&css.id.to_string(), css.content.clone()).unwrap();
    *css.meta = farmfe_core::module::ModuleMetaData::Css(Box::new(
      farmfe_core::module::meta_data::css::CssModuleMetaData {
        ast: parsed.ast,
        comments: Default::default(),
        custom: Default::default(),
      },
    ));
    plugin
      .freeze_module(
        &mut PluginFreezeModuleHookParam {
          module: &mut css,
          resolved_deps: vec![],
        },
        &context,
      )
      .unwrap();
    assert!(
      css.content.contains(".p-4"),
      "cached source utilities must be generated in the stylesheet"
    );
  }

  #[test]
  fn source_without_candidates_still_invalidates_styles_when_classes_are_added() {
    let context = Arc::new(CompilationContext::default());
    let plugin = FarmPluginTailwindCSS::new(&context.config, "{}".to_string());
    let path = format!("{}/src/empty.tsx", context.config.root);
    plugin
      .transform(
        &PluginTransformHookParam {
          module_id: "src/empty.tsx".to_string(),
          content: String::new(),
          module_type: ModuleType::Tsx,
          resolved_path: &path,
          query: vec![],
          meta: Default::default(),
          source_map_chain: vec![],
        },
        &context,
      )
      .unwrap();
    assert!(plugin.candidate_sources.lock().unwrap().contains(&path));
  }
}
