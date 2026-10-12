#![deny(clippy::all)]

use std::collections::{HashMap, HashSet};
use std::path;
use std::sync::{Arc, Mutex};

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::CompilationError,
  module::{Module, ModuleId, ModuleType},
  plugin::{
    Plugin, PluginTransformHookParam, PluginTransformHookResult,
    hooks::freeze_module::PluginFreezeModuleHookParam,
  },
  serde_json,
};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::css::parse_css_stylesheet;
use farmfe_toolkit::lazy_static::lazy_static;
use farmfe_toolkit::regex::Regex;

use farmfe_ecosystem_tailwindcss::TailwindConfig;
use farmfe_ecosystem_tailwindcss_node::compile::{self, CompileOptions};
use tailwindcss_oxide::{ChangedContent, Scanner};

const PKG_NAME: &str = "@farmfe/plugin-tailwindcss";

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
#[serde(rename_all = "camelCase")]
struct TailwindCSSOptions {
  /// Extra paths / globs to scan for candidates.
  #[serde(default)]
  #[allow(dead_code)]
  content: Vec<String>,
  /// Optional Tailwind config payload passed through directly to the Rust core crate.
  #[serde(default)]
  config: Option<serde_json::Value>,
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
  /// Set of all candidates collected from scanned source files.
  candidates: Arc<Mutex<HashSet<String>>>,
  /// Set of module IDs that contributed candidates.
  candidate_sources: Arc<Mutex<HashSet<String>>>,
  /// Original (untransformed) CSS content for CSS root files that need
  /// deferred TailwindCSS compilation. The actual compile-and-inline pass
  /// runs in `freeze_module`, by which time the candidate set is complete.
  pending_css: Arc<Mutex<HashMap<String, String>>>,
}

impl FarmPluginTailwindCSS {
  pub fn new(_config: &Config, options: String) -> Self {
    let parsed: TailwindCSSOptions = serde_json::from_str(&options).unwrap_or_default();

    Self {
      _options: parsed,
      candidates: Arc::new(Mutex::new(HashSet::new())),
      candidate_sources: Arc::new(Mutex::new(HashSet::new())),
      pending_css: Arc::new(Mutex::new(HashMap::new())),
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
        std::fs::read_to_string(&resolved_path).unwrap_or_else(|_| module.content.to_string());
      self.register_candidate_source(&resolved_path, &content)?;
    }
    // Let other plugins decide whether this otherwise valid module cache expires.
    Ok(None)
  }

  fn transform(
    &self,
    param: &PluginTransformHookParam,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<PluginTransformHookResult>> {
    let resolved_path = param.resolved_path;

    // ── Step 1: Scan non-CSS source files for candidates ────────────
    if Self::is_candidate_source_file(resolved_path) {
      self.register_candidate_source(resolved_path, &param.content)?;

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
      return Ok(None);
    }

    self
      .pending_css
      .lock()
      .unwrap()
      .insert(resolved_path.to_string(), param.content.clone());

    // Also scan the CSS itself for candidates inside string literals / @apply
    // values — Tailwind's official scanner picks these up.
    let new_candidates = self.scan_candidates(&param.content, "css");
    if !new_candidates.is_empty() {
      let mut candidates = self.candidates.lock().unwrap();
      for c in new_candidates {
        candidates.insert(c);
      }
    }

    Ok(None)
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
      guard.remove(&resolved_path)
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
        eprintln!("[{PKG_NAME}] Failed to compile TailwindCSS for {resolved_path}: {e}");
        return Ok(None);
      }
    };

    if !compiler.features.has_any_output_feature() {
      return Ok(None);
    }

    let candidates: Vec<String> = self.candidates.lock().unwrap().iter().cloned().collect();

    let css = compiler.build(&candidates);

    // Re-parse the generated CSS so the module AST matches the new content.
    let module_id_str = param.module.id.to_string();
    let parsed = match parse_css_stylesheet(&module_id_str, Arc::new(css.clone())) {
      Ok(p) => p,
      Err(e) => {
        eprintln!("[{PKG_NAME}] Failed to re-parse generated CSS for {resolved_path}: {e}");
        return Ok(None);
      }
    };

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
      let _ = context.add_watch_files(
        param.module.id.clone(),
        vec![ModuleId::new(source, "", &context.config.root)],
      );
    }

    param.module.meta.as_css_mut().set_ast(parsed.ast);
    param.module.content = Arc::new(css);

    Ok(Some(()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Helper for unit tests that construct the plugin without going through
  /// the JSON-options entry point. Keeps tests insulated from new internal
  /// fields.
  fn make_plugin() -> FarmPluginTailwindCSS {
    FarmPluginTailwindCSS {
      _options: TailwindCSSOptions::default(),
      candidates: Arc::new(Mutex::new(HashSet::new())),
      candidate_sources: Arc::new(Mutex::new(HashSet::new())),
      pending_css: Arc::new(Mutex::new(HashMap::new())),
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
