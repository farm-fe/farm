//! Compilation orchestration.
//!
//! Rust port of
//! [`@tailwindcss-node/src/compile.ts`](https://github.com/tailwindlabs/tailwindcss/blob/main/packages/%40tailwindcss-node/src/compile.ts).
//!
//! This module wires together resolution, stylesheet loading, URL rewriting,
//! and dependency tracking into a single `compile` entry-point that mirrors
//! the TypeScript original.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::css_origins::MappedCss;
use crate::resolve::{self, CustomResolver};
use crate::urls;
pub use farmfe_ecosystem_tailwindcss::compiler::{
  Compiler, CompilerOptions, Features, Polyfills, TailwindConfig,
};

// ── Types ───────────────────────────────────────────────────────────────────

/// Options passed to [`compile`] and [`compile_ast`].
pub struct CompileOptions {
  /// Base directory for resolving relative paths.
  pub base: String,
  /// Optional source file path (enables source-maps when present).
  pub from: Option<String>,
  /// Whether to rewrite relative `url()` references.
  pub should_rewrite_urls: bool,
  /// Polyfill flags for CSS compatibility transforms.
  pub polyfills: Polyfills,
  /// Optional externally provided Tailwind config object.
  ///
  /// This is intentionally pass-through only. JS/TS config file loading is out
  /// of scope for this Rust implementation.
  pub config: Option<TailwindConfig>,
  /// Callback invoked whenever a dependency is discovered.
  pub on_dependency: Box<dyn Fn(&str) + Send>,
  /// Optional custom CSS resolver.
  pub custom_css_resolver: Option<CustomResolver>,
  /// Optional custom JS resolver.
  pub custom_js_resolver: Option<CustomResolver>,
}

impl Default for CompileOptions {
  fn default() -> Self {
    Self {
      base: ".".to_string(),
      from: None,
      should_rewrite_urls: false,
      polyfills: Polyfills::NONE,
      config: None,
      on_dependency: Box::new(|_| {}),
      custom_css_resolver: None,
      custom_js_resolver: None,
    }
  }
}

/// Loaded stylesheet information.
#[derive(Debug, Clone)]
pub struct LoadedStylesheet {
  pub path: PathBuf,
  pub base: String,
  pub content: String,
}

/// Loaded module information.
#[derive(Debug, Clone)]
pub struct LoadedModule {
  pub path: PathBuf,
  pub base: String,
}

/// Source detection root information.
#[derive(Debug, Clone)]
pub struct SourceRoot {
  pub pattern: String,
  pub base: String,
}

/// Simplified AST node representation.
///
/// Mirrors the `AstNode` type from `tailwindcss/src/ast`. In the upstream
/// TypeScript implementation, a full CSS AST is passed to `compileAst`.
/// This Rust version uses a simplified representation.
#[derive(Debug, Clone)]
pub enum AstNode {
  /// A CSS rule, e.g. `.foo { color: red; }`.
  Rule {
    selector: String,
    nodes: Vec<AstNode>,
  },
  /// A CSS at-rule, e.g. `@import "tailwindcss";`.
  AtRule {
    name: String,
    params: String,
    nodes: Vec<AstNode>,
  },
  /// A CSS declaration, e.g. `color: red`.
  Declaration { property: String, value: String },
  /// Raw CSS text (comment or preserved text).
  Comment(String),
}

impl AstNode {
  /// Convert the AST back into a CSS string.
  pub fn to_css(&self) -> String {
    match self {
      AstNode::Rule { selector, nodes } => {
        let mut inner = String::new();
        for (i, n) in nodes.iter().enumerate() {
          if i > 0 {
            inner.push('\n');
          }
          inner.push_str(&n.to_css());
        }
        format!("{selector} {{\n{inner}\n}}")
      }
      AstNode::AtRule {
        name,
        params,
        nodes,
      } => {
        if nodes.is_empty() {
          format!("@{name} {params};")
        } else {
          let mut inner = String::new();
          for (i, n) in nodes.iter().enumerate() {
            if i > 0 {
              inner.push('\n');
            }
            inner.push_str(&n.to_css());
          }
          format!("@{name} {params} {{\n{inner}\n}}")
        }
      }
      AstNode::Declaration { property, value } => {
        format!("  {property}: {value};")
      }
      AstNode::Comment(text) => format!("/* {text} */"),
    }
  }
}

// ── Stylesheet loading ──────────────────────────────────────────────────────

/// Load a CSS stylesheet, resolving the id from `base`.
pub fn load_stylesheet(
  id: &str,
  base: &str,
  on_dependency: &dyn Fn(&str),
  custom_css_resolver: Option<&CustomResolver>,
) -> io::Result<LoadedStylesheet> {
  let resolved = resolve::resolve_css_id(id, base, custom_css_resolver)?;

  on_dependency(resolved.to_str().unwrap_or_default());

  let content = std::fs::read_to_string(&resolved)?;
  let sheet_base = resolved
    .parent()
    .unwrap_or(Path::new("."))
    .to_string_lossy()
    .into_owned();

  Ok(LoadedStylesheet {
    path: resolved,
    base: sheet_base,
    content,
  })
}

/// Load a JS module, resolving the id from `base`.
pub fn load_module(
  id: &str,
  base: &str,
  on_dependency: &dyn Fn(&str),
  custom_js_resolver: Option<&CustomResolver>,
) -> io::Result<LoadedModule> {
  let resolved = resolve::resolve_js_id(id, base, custom_js_resolver)?;

  // For relative imports, trace all transitive dependencies.
  if id.starts_with('.') {
    let deps = crate::get_module_dependencies::get_module_dependencies(&resolved)?;
    for dep in &deps {
      on_dependency(dep.to_str().unwrap_or_default());
    }
  }

  let module_base = resolved
    .parent()
    .unwrap_or(Path::new("."))
    .to_string_lossy()
    .into_owned();

  Ok(LoadedModule {
    path: resolved,
    base: module_base,
  })
}

// ── Source root validation ──────────────────────────────────────────────────

/// Verify that the `source(…)` root directory exists.
///
/// Mirrors `ensureSourceDetectionRootExists` from the TS implementation.
fn ensure_source_detection_root_exists(root: &Option<SourceRoot>) -> io::Result<()> {
  let root = match root {
    Some(r) if r.pattern != "none" => r,
    _ => return Ok(()),
  };

  let glob_symbols = ['*', '{'];
  let base_segments: Vec<&str> = root
    .pattern
    .split('/')
    .take_while(|seg| !seg.chars().any(|c| glob_symbols.contains(&c)))
    .collect();

  let base_path = Path::new(&root.base).join(base_segments.join("/"));

  if !base_path.is_dir() {
    return Err(io::Error::new(
      io::ErrorKind::NotFound,
      format!(
        "The `source({})` does not exist or is not a directory.",
        root.pattern
      ),
    ));
  }

  Ok(())
}

// ── resolve @import ─────────────────────────────────────────────────────────

/// Simple regex-based `@import` resolver for CSS files.
///
/// This is a simplified version of what the core tailwindcss compiler does.
/// It recursively resolves `@import` directives and inlines the imported CSS.
fn resolve_imports(
  input: &MappedCss,
  base: &str,
  root_base: &str,
  should_rewrite_urls: bool,
  on_dependency: &dyn Fn(&str),
  custom_css_resolver: Option<&CustomResolver>,
  seen: &mut HashMap<String, bool>,
) -> io::Result<(MappedCss, Features)> {
  let css = &input.text;
  let import_re = regex::Regex::new(r#"@import\s+['"]([^'"]+)['"]"#).unwrap();

  let mut result = input.empty();
  let mut features = Features::NONE;
  let mut last_end = 0;

  for cap in import_re.captures_iter(css) {
    let full_match = cap.get(0).unwrap();
    let import_path = &cap[1];

    result.append_slice(input, last_end..full_match.start());
    last_end = full_match.end();

    // Also skip the trailing semicolon + newline if present
    let rest = &css[last_end..];
    if rest.starts_with(';') {
      last_end += 1;
      let rest2 = &css[last_end..];
      if rest2.starts_with('\n') {
        last_end += 1;
      } else if rest2.starts_with("\r\n") {
        last_end += 2;
      }
    }

    // Tailwind's package stylesheet includes theme and preflight layers, not
    // just utilities. Resolve it through the same loader as other CSS imports.
    if import_path == "tailwindcss" || import_path.starts_with("tailwindcss/") {
      features |= Features::UTILITIES;
    }

    // Try to resolve and inline the import
    match load_stylesheet(import_path, base, on_dependency, custom_css_resolver) {
      Ok(sheet) => {
        let sheet_key = sheet.path.to_string_lossy().to_string();
        if seen.contains_key(&sheet_key) {
          continue; // Already inlined
        }
        seen.insert(sheet_key.clone(), true);

        let mut content = MappedCss::new(
          &sheet.content,
          input.is_mapped().then_some(sheet_key.as_str()),
        );

        // Rewrite URLs if needed
        if should_rewrite_urls {
          let rewritten = urls::rewrite_urls(&content.text, &sheet.base, root_base);
          content = content.rewritten(rewritten);
        }

        // Recursively resolve imports in the inlined CSS
        let (resolved_content, inner_features) = resolve_imports(
          &content,
          &sheet.base,
          root_base,
          should_rewrite_urls,
          on_dependency,
          custom_css_resolver,
          seen,
        )?;
        features |= inner_features;
        result.append(&resolved_content);
      }
      Err(_) => {
        // If we can't resolve, keep the original import
        result.append(&MappedCss::new(
          &format!("@import \"{import_path}\";\n"),
          None,
        ));
      }
    }
  }

  result.append_slice(input, last_end..css.len());

  // Detect features in the CSS
  if css.contains("@apply ") {
    features |= Features::AT_APPLY;
  }
  if css.contains("theme(") {
    features |= Features::THEME_FUNCTION;
  }

  Ok((result, features))
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Compile a CSS string with the given options.
///
/// This mirrors the `compile()` function from the TS `@tailwindcss-node`
/// package. It resolves `@import` directives, rewrites URLs, tracks
/// dependencies, and returns a [`Compiler`] that can produce the final output.
pub fn compile(css: &str, options: CompileOptions) -> io::Result<Compiler> {
  let prepared = prepare_stylesheet(css, &options)?;
  let mut compiler = farmfe_ecosystem_tailwindcss::compiler::compile(
    &prepared.content,
    CompilerOptions {
      features: prepared.features,
      polyfills: options.polyfills,
      dependencies: prepared.dependencies,
      source_maps_enabled: options.from.is_some(),
      config: options.config.clone(),
    },
  )
  .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Compile error: {}", e)))?;

  if let Some(map) = prepared.source_map {
    compiler.set_input_source_map(map);
  }
  ensure_source_detection_root_exists(&None)?;
  Ok(compiler)
}

/// Import/URL preparation must precede Farm dependency analysis. Candidate
/// expansion can then wait until every script module has been scanned.
pub struct PreparedStylesheet {
  pub content: String,
  pub source_map: Option<sourcemap::SourceMap>,
  pub dependencies: Vec<String>,
  pub features: Features,
}

pub fn prepare_stylesheet(css: &str, options: &CompileOptions) -> io::Result<PreparedStylesheet> {
  let mut seen = HashMap::new();
  let input = MappedCss::new(css, options.from.as_deref());

  let (processed_css, features) = resolve_imports(
    &input,
    &options.base,
    &options.base,
    options.should_rewrite_urls,
    &*options.on_dependency,
    options.custom_css_resolver.as_ref(),
    &mut seen,
  )?;

  let source_map = processed_css.source_map();
  Ok(PreparedStylesheet {
    content: processed_css.text,
    source_map,
    dependencies: seen.keys().cloned().collect(),
    features,
  })
}
/// Compile a pre-parsed AST with the given options.
///
/// Mirrors the `compileAst()` function from the TS `@tailwindcss-node`
/// package. Instead of accepting raw CSS text, it takes a pre-parsed
/// [`AstNode`] tree, serialises it to CSS, and then runs the standard
/// compilation pipeline.
pub fn compile_ast(ast: &[AstNode], options: CompileOptions) -> io::Result<Compiler> {
  let mut css = String::new();
  for (i, node) in ast.iter().enumerate() {
    if i > 0 {
      css.push('\n');
    }
    css.push_str(&node.to_css());
  }
  compile(&css, options)
}

/// Load the design system from a CSS string.
///
/// Mirrors the `__unstable__loadDesignSystem()` function from the TS
/// `@tailwindcss-node` package. This is an unstable API used internally to
/// load the design system without a full compilation pipeline. Currently
/// delegates to [`compile`] with minimal options.
pub fn load_design_system(css: &str, base: &str) -> io::Result<Compiler> {
  compile(
    css,
    CompileOptions {
      base: base.to_string(),
      ..Default::default()
    },
  )
}
