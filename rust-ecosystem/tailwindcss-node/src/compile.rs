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
  css: &str,
  base: &str,
  root_base: &str,
  should_rewrite_urls: bool,
  on_dependency: &dyn Fn(&str),
  custom_css_resolver: Option<&CustomResolver>,
  seen: &mut HashMap<String, bool>,
) -> io::Result<(String, Features)> {
  let import_re = regex::Regex::new(r#"@import\s+['"]([^'"]+)['"]"#).unwrap();

  let mut result = String::new();
  let mut features = Features::NONE;
  let mut last_end = 0;

  for cap in import_re.captures_iter(css) {
    let full_match = cap.get(0).unwrap();
    let import_path = &cap[1];

    result.push_str(&css[last_end..full_match.start()]);
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

    // Handle special imports (tailwindcss directives)
    if import_path == "tailwindcss" || import_path.starts_with("tailwindcss/") {
      features |= Features::UTILITIES;
      result.push_str(&format!("@import \"{import_path}\";\n"));
      continue;
    }

    // Try to resolve and inline the import
    match load_stylesheet(import_path, base, on_dependency, custom_css_resolver) {
      Ok(sheet) => {
        let sheet_key = sheet.path.to_string_lossy().to_string();
        if seen.contains_key(&sheet_key) {
          continue; // Already inlined
        }
        seen.insert(sheet_key, true);

        let mut content = sheet.content.clone();

        // Rewrite URLs if needed
        if should_rewrite_urls {
          content = urls::rewrite_urls(&content, &sheet.base, root_base);
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
        result.push_str(&resolved_content);
      }
      Err(_) => {
        // If we can't resolve, keep the original import
        result.push_str(&format!("@import \"{import_path}\";\n"));
      }
    }
  }

  result.push_str(&css[last_end..]);

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
  let mut seen = HashMap::new();

  let (processed_css, features) = resolve_imports(
    css,
    &options.base,
    &options.base,
    options.should_rewrite_urls,
    &*options.on_dependency,
    options.custom_css_resolver.as_ref(),
    &mut seen,
  )?;

  let compiler = farmfe_ecosystem_tailwindcss::compiler::compile(
    &processed_css,
    CompilerOptions {
      features,
      polyfills: options.polyfills,
      dependencies: seen.keys().cloned().collect(),
      source_maps_enabled: options.from.is_some(),
      config: options.config.clone(),
    },
  )
  .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Compile error: {}", e)))?;

  ensure_source_detection_root_exists(&None)?;

  Ok(compiler)
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
