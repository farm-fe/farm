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

// ── Types ───────────────────────────────────────────────────────────────────

/// Bitflags that describe which Tailwind CSS features are used in the compiled
/// CSS.  Mirrors the `Features` enum from `tailwindcss`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Features(u32);

impl Features {
  pub const NONE: Self = Self(0);
  pub const AT_APPLY: Self = Self(1 << 0);
  pub const JS_PLUGIN_COMPAT: Self = Self(1 << 1);
  pub const THEME_FUNCTION: Self = Self(1 << 2);
  pub const UTILITIES: Self = Self(1 << 3);

  pub fn contains(self, other: Self) -> bool {
    self.0 & other.0 != 0
  }

  pub fn has_any_output_feature(self) -> bool {
    self.contains(Self::AT_APPLY)
      || self.contains(Self::JS_PLUGIN_COMPAT)
      || self.contains(Self::THEME_FUNCTION)
      || self.contains(Self::UTILITIES)
  }
}

/// Polyfill flags that control which CSS compatibility transforms are applied.
///
/// Mirrors the `Polyfills` enum from `tailwindcss`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Polyfills(u32);

impl Polyfills {
  pub const NONE: Self = Self(0);
  /// Enable `@media (hover: hover)` polyfill for older browsers.
  pub const AT_MEDIA_HOVER: Self = Self(1 << 0);

  pub fn contains(self, other: Self) -> bool {
    self.0 & other.0 != 0
  }
}

impl std::ops::BitOr for Features {
  type Output = Self;
  fn bitor(self, rhs: Self) -> Self {
    Self(self.0 | rhs.0)
  }
}

impl std::ops::BitOrAssign for Features {
  fn bitor_assign(&mut self, rhs: Self) {
    self.0 |= rhs.0;
  }
}

impl std::ops::BitAnd for Features {
  type Output = Self;
  fn bitand(self, rhs: Self) -> Self {
    Self(self.0 & rhs.0)
  }
}

impl std::ops::BitOr for Polyfills {
  type Output = Self;
  fn bitor(self, rhs: Self) -> Self {
    Self(self.0 | rhs.0)
  }
}

impl std::ops::BitOrAssign for Polyfills {
  fn bitor_assign(&mut self, rhs: Self) {
    self.0 |= rhs.0;
  }
}

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

/// The compiled state. Holds the processed CSS, detected features, dependency
/// graph, and methods for building the final output from a set of candidates.
pub struct Compiler {
  /// The original (or processed) CSS content.
  css: String,
  /// Detected features in the CSS.
  pub features: Features,
  /// Active polyfills.
  pub polyfills: Polyfills,
  /// Source detection root, if any.
  pub root: Option<SourceRoot>,
  /// Build dependencies discovered during compilation.
  dependencies: Vec<String>,
  /// Cached built CSS output (from last `build` call).
  last_build: Option<String>,
  /// Whether source maps are enabled.
  source_maps_enabled: bool,
}

impl Compiler {
  /// Build the final CSS output using the given set of candidate class names.
  ///
  /// In the upstream TS implementation this delegates to the core tailwindcss
  /// compiler. In this Rust version the CSS is returned with imports resolved
  /// and URLs rewritten. The `candidates` parameter is accepted for API
  /// compatibility and will be used when a full Rust CSS generator is
  /// available.
  pub fn build(&mut self, _candidates: &[String]) -> String {
    let css = self.css.clone();
    self.last_build = Some(css.clone());
    css
  }

  /// Build and return a source-map string, if source maps are enabled.
  pub fn build_source_map(&self) -> Option<String> {
    if !self.source_maps_enabled {
      return None;
    }
    // Minimal source map — a full implementation would produce proper
    // mappings by tracking transformations.
    Some(r#"{"version":3,"sources":[],"names":[],"mappings":""}"#.to_string())
  }

  /// Return the list of dependencies discovered during compilation.
  pub fn dependencies(&self) -> &[String] {
    &self.dependencies
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

  let compiler = Compiler {
    css: processed_css,
    features,
    polyfills: options.polyfills,
    root: None,
    dependencies: seen.keys().cloned().collect(),
    last_build: None,
    source_maps_enabled: options.from.is_some(),
  };

  ensure_source_detection_root_exists(&compiler.root)?;

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
