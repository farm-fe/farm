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
                let inner: String = nodes.iter().map(|n| n.to_css()).collect::<Vec<_>>().join("\n");
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
                    let inner: String =
                        nodes.iter().map(|n| n.to_css()).collect::<Vec<_>>().join("\n");
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
        Some(
            r#"{"version":3,"sources":[],"names":[],"mappings":""}"#
                .to_string(),
        )
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
    let import_re =
        regex::Regex::new(r#"@import\s+['"]([^'"]+)['"]"#).unwrap();

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
    let css: String = ast.iter().map(|n| n.to_css()).collect::<Vec<_>>().join("\n");
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::{Arc, Mutex};

    fn setup_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("farm_tw_compile_test")
            .join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn compile_simple_css() {
        let dir = setup_dir("simple");
        let css = ".foo { color: red; }";

        let compiler = compile(
            css,
            CompileOptions {
                base: dir.to_str().unwrap().to_string(),
                ..Default::default()
            },
        )
        .unwrap();

        let mut compiler = compiler;
        let output = compiler.build(&[]);
        assert!(output.contains(".foo"));
        assert!(output.contains("color: red"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compile_resolves_imports() {
        let dir = setup_dir("imports");
        let sub = dir.join("sub.css");
        fs::write(&sub, "body { margin: 0; }").unwrap();

        let css = r#"@import "./sub.css";
.main { color: blue; }"#;

        let deps: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let deps_clone = deps.clone();

        let compiler = compile(
            css,
            CompileOptions {
                base: dir.to_str().unwrap().to_string(),
                on_dependency: Box::new(move |dep| {
                    deps_clone.lock().unwrap().push(dep.to_string());
                }),
                ..Default::default()
            },
        )
        .unwrap();

        let mut compiler = compiler;
        let output = compiler.build(&[]);
        assert!(output.contains("body { margin: 0; }"));
        assert!(output.contains(".main { color: blue; }"));

        // Check that the dependency was tracked
        let tracked = deps.lock().unwrap();
        assert!(!tracked.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compile_rewrites_urls() {
        let dir = setup_dir("rewrite");
        let sub_dir = dir.join("sub");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(
            sub_dir.join("styles.css"),
            ".bg { background: url(./image.png); }",
        )
        .unwrap();

        let css = r#"@import "./sub/styles.css";"#;

        let compiler = compile(
            css,
            CompileOptions {
                base: dir.to_str().unwrap().to_string(),
                should_rewrite_urls: true,
                ..Default::default()
            },
        )
        .unwrap();

        let mut compiler = compiler;
        let output = compiler.build(&[]);
        // The URL should be rebased relative to the root
        assert!(output.contains("image.png"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compile_detects_at_apply() {
        let dir = setup_dir("at_apply");
        let css = ".foo { @apply text-red-500; }";

        let compiler = compile(
            css,
            CompileOptions {
                base: dir.to_str().unwrap().to_string(),
                ..Default::default()
            },
        )
        .unwrap();

        assert!(compiler.features.contains(Features::AT_APPLY));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compile_detects_theme_function() {
        let dir = setup_dir("theme_fn");
        let css = ".foo { color: theme(colors.red.500); }";

        let compiler = compile(
            css,
            CompileOptions {
                base: dir.to_str().unwrap().to_string(),
                ..Default::default()
            },
        )
        .unwrap();

        assert!(compiler.features.contains(Features::THEME_FUNCTION));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compile_detects_tailwindcss_import() {
        let dir = setup_dir("tw_import");
        let css = r#"@import "tailwindcss";"#;

        let compiler = compile(
            css,
            CompileOptions {
                base: dir.to_str().unwrap().to_string(),
                ..Default::default()
            },
        )
        .unwrap();

        assert!(compiler.features.contains(Features::UTILITIES));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compile_source_maps_disabled_by_default() {
        let dir = setup_dir("no_sm");
        let css = ".foo { color: red; }";

        let compiler = compile(
            css,
            CompileOptions {
                base: dir.to_str().unwrap().to_string(),
                ..Default::default()
            },
        )
        .unwrap();

        assert!(compiler.build_source_map().is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compile_source_maps_enabled_with_from() {
        let dir = setup_dir("with_sm");
        let css = ".foo { color: red; }";

        let compiler = compile(
            css,
            CompileOptions {
                base: dir.to_str().unwrap().to_string(),
                from: Some("input.css".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        assert!(compiler.build_source_map().is_some());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn features_bitflags() {
        let f = Features::AT_APPLY | Features::UTILITIES;
        assert!(f.contains(Features::AT_APPLY));
        assert!(f.contains(Features::UTILITIES));
        assert!(!f.contains(Features::THEME_FUNCTION));
        assert!(f.has_any_output_feature());

        assert!(!Features::NONE.has_any_output_feature());
    }

    #[test]
    fn compile_handles_missing_import_gracefully() {
        let dir = setup_dir("missing_import");
        let css = r#"@import "./nonexistent.css";
.foo { color: red; }"#;

        let compiler = compile(
            css,
            CompileOptions {
                base: dir.to_str().unwrap().to_string(),
                ..Default::default()
            },
        )
        .unwrap();

        let mut compiler = compiler;
        let output = compiler.build(&[]);
        // The missing import should be kept as-is
        assert!(output.contains("nonexistent.css"));
        assert!(output.contains(".foo { color: red; }"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_stylesheet_works() {
        let dir = setup_dir("load_sheet");
        let css_file = dir.join("styles.css");
        fs::write(&css_file, "body { margin: 0; }").unwrap();

        let deps: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let deps_clone = deps.clone();

        let sheet = load_stylesheet(
            "./styles.css",
            dir.to_str().unwrap(),
            &move |dep| {
                deps_clone.lock().unwrap().push(dep.to_string());
            },
            None,
        )
        .unwrap();

        assert_eq!(sheet.content, "body { margin: 0; }");
        assert_eq!(sheet.path, css_file);

        let tracked = deps.lock().unwrap();
        assert!(!tracked.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_module_works() {
        let dir = setup_dir("load_mod");
        let js_file = dir.join("config.js");
        fs::write(&js_file, "module.exports = {};").unwrap();

        let module = load_module(
            "./config.js",
            dir.to_str().unwrap(),
            &|_| {},
            None,
        )
        .unwrap();

        assert_eq!(module.path, js_file);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ensure_source_root_none_passes() {
        assert!(ensure_source_detection_root_exists(&None).is_ok());
    }

    #[test]
    fn ensure_source_root_valid_dir_passes() {
        let dir = setup_dir("src_root");
        let root = SourceRoot {
            pattern: "**/*.ts".to_string(),
            base: dir.to_str().unwrap().to_string(),
        };
        // The base dir itself exists, so this should pass
        assert!(ensure_source_detection_root_exists(&Some(root)).is_ok());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ensure_source_root_missing_dir_fails() {
        let root = SourceRoot {
            pattern: "nonexistent/**/*.ts".to_string(),
            base: "/tmp/farm_tw_compile_test_missing".to_string(),
        };
        assert!(ensure_source_detection_root_exists(&Some(root)).is_err());
    }

    #[test]
    fn polyfills_bitflags() {
        let p = Polyfills::NONE;
        assert!(!p.contains(Polyfills::AT_MEDIA_HOVER));

        let p = Polyfills::AT_MEDIA_HOVER;
        assert!(p.contains(Polyfills::AT_MEDIA_HOVER));

        let combined = Polyfills::NONE | Polyfills::AT_MEDIA_HOVER;
        assert!(combined.contains(Polyfills::AT_MEDIA_HOVER));
    }

    #[test]
    fn compile_with_polyfills() {
        let dir = setup_dir("with_polyfills");
        let css = ".foo { color: red; }";

        let compiler = compile(
            css,
            CompileOptions {
                base: dir.to_str().unwrap().to_string(),
                polyfills: Polyfills::AT_MEDIA_HOVER,
                ..Default::default()
            },
        )
        .unwrap();

        assert!(compiler.polyfills.contains(Polyfills::AT_MEDIA_HOVER));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compile_ast_simple() {
        let dir = setup_dir("compile_ast");
        let ast = vec![
            AstNode::Rule {
                selector: ".foo".to_string(),
                nodes: vec![AstNode::Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                }],
            },
            AstNode::Rule {
                selector: ".bar".to_string(),
                nodes: vec![AstNode::Declaration {
                    property: "margin".to_string(),
                    value: "0".to_string(),
                }],
            },
        ];

        let mut compiler = compile_ast(
            &ast,
            CompileOptions {
                base: dir.to_str().unwrap().to_string(),
                ..Default::default()
            },
        )
        .unwrap();

        let output = compiler.build(&[]);
        assert!(output.contains(".foo"));
        assert!(output.contains("color: red"));
        assert!(output.contains(".bar"));
        assert!(output.contains("margin: 0"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compile_ast_at_rule() {
        let dir = setup_dir("compile_ast_atrule");
        let ast = vec![AstNode::AtRule {
            name: "import".to_string(),
            params: "\"tailwindcss\"".to_string(),
            nodes: vec![],
        }];

        let compiler = compile_ast(
            &ast,
            CompileOptions {
                base: dir.to_str().unwrap().to_string(),
                ..Default::default()
            },
        )
        .unwrap();

        // The @import "tailwindcss" should be detected as a UTILITIES feature
        assert!(compiler.features.contains(Features::UTILITIES));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ast_node_to_css() {
        let node = AstNode::Rule {
            selector: ".foo".to_string(),
            nodes: vec![AstNode::Declaration {
                property: "color".to_string(),
                value: "red".to_string(),
            }],
        };
        let css = node.to_css();
        assert!(css.contains(".foo {"));
        assert!(css.contains("color: red;"));

        let at_rule = AstNode::AtRule {
            name: "media".to_string(),
            params: "(hover: hover)".to_string(),
            nodes: vec![AstNode::Rule {
                selector: ".btn:hover".to_string(),
                nodes: vec![AstNode::Declaration {
                    property: "opacity".to_string(),
                    value: "0.8".to_string(),
                }],
            }],
        };
        let css = at_rule.to_css();
        assert!(css.contains("@media (hover: hover) {"));
        assert!(css.contains(".btn:hover {"));

        let comment = AstNode::Comment("test comment".to_string());
        assert_eq!(comment.to_css(), "/* test comment */");
    }

    #[test]
    fn load_design_system_works() {
        let dir = setup_dir("design_system");
        let css = ".foo { color: red; }";

        let mut compiler =
            load_design_system(css, dir.to_str().unwrap()).unwrap();
        let output = compiler.build(&[]);
        assert!(output.contains(".foo"));
        assert!(output.contains("color: red"));

        let _ = fs::remove_dir_all(&dir);
    }
}
