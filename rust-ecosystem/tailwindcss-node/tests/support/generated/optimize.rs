use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};
use lightningcss::targets::{Browsers, Features, Targets};

// ── types ────────────────────────────────────────────────────────────────────

/// Options for [`optimize`].
#[derive(Debug, Clone)]
pub struct OptimizeOptions {
    /// The filename being transformed — used in error messages and source maps.
    /// Defaults to `"input.css"`.
    pub file: String,
    /// Enable minified output (removes whitespace, merges declarations, etc.).
    pub minify: bool,
}

impl Default for OptimizeOptions {
    fn default() -> Self {
        Self {
            file: "input.css".to_string(),
            minify: false,
        }
    }
}

/// The result of [`optimize`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformResult {
    /// The optimized (and optionally minified) CSS string.
    pub code: String,
}

// ── browser targets ──────────────────────────────────────────────────────────

/// Browser targets matching the upstream optimize.ts defaults.
///
/// ```text
/// safari:  16.4  → (16 << 16) | (4 << 8)
/// firefox: 128   → (128 << 16)
/// chrome:  111   → (111 << 16)
/// ```
fn default_targets() -> Targets {
    Targets {
        browsers: Some(Browsers {
            safari: Some((16 << 16) | (4 << 8)),
            firefox: Some(128 << 16),
            chrome: Some(111 << 16),
            ..Default::default()
        }),
        // Mirror upstream: include Nesting + MediaQueries transforms
        include: Features::Nesting | Features::MediaQueries,
        // Mirror upstream: exclude LogicalProperties, DirSelector, LightDark
        exclude: Features::LogicalProperties | Features::DirSelector | Features::LightDark,
    }
}

// ── implementation ────────────────────────────────────────────────────────────

/// Run one pass of the lightningcss transform pipeline.
///
/// Returns the transformed CSS or an error message.
fn run_pass(input: &str, minify: bool) -> Result<String, String> {
    let targets = default_targets();

    let mut sheet = StyleSheet::parse(input, ParserOptions::default())
        .map_err(|e| format!("lightningcss parse error: {e}"))?;

    sheet
        .minify(MinifyOptions {
            targets,
            ..Default::default()
        })
        .map_err(|e| format!("lightningcss minify error: {e}"))?;

    let result = sheet
        .to_css(PrinterOptions {
            minify,
            targets,
            ..Default::default()
        })
        .map_err(|e| format!("lightningcss print error: {e}"))?;

    Ok(result.code)
}

/// Apply `@media not (` → `@media not all and (` workaround.
///
/// The upstream TypeScript version uses `MagicString.replaceAll` for this.
/// Since we do not need source-map preservation at this level (source-map
/// support requires the optional `sourcemap` feature), a plain string replace
/// is sufficient.
fn fix_media_not(css: &str) -> String {
    css.replace("@media not (", "@media not all and (")
}

// ── public API ────────────────────────────────────────────────────────────────

/// Optimize a CSS string using Lightning CSS.
///
/// Mirrors the `optimize()` function from `optimize.ts`.
///
/// ```
/// use farmfe_ecosystem_tailwindcss::optimize::{optimize, OptimizeOptions};
///
/// let input = ".foo { & .bar { color: red; } }";
/// let result = optimize(input, OptimizeOptions::default()).unwrap();
/// // The nested rule is expanded by lightningcss
/// assert!(result.code.contains(".foo"));
/// ```
pub fn optimize(input: &str, options: OptimizeOptions) -> Result<TransformResult, String> {
    // First pass — apply transforms (nesting → flat, media-query range → level-3)
    let after_first = run_pass(input, options.minify)?;

    // Second pass — merge adjacent rules produced by the first pass
    let mut code = run_pass(&after_first, options.minify)?;

    // Apply the @media-not fix
    code = fix_media_not(&code);

    Ok(TransformResult { code })
}
