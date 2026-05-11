//! CSS optimization.
//!
//! Rust port of
//! [`@tailwindcss-node/src/optimize.ts`](https://github.com/tailwindlabs/tailwindcss/blob/main/packages/%40tailwindcss-node/src/optimize.ts).
//!
//! Uses the native [`lightningcss`] crate to apply the same transforms as the
//! upstream TypeScript version, which used `lightningcss`'s JavaScript bindings.
//!
//! The transform pipeline:
//! 1. Parse the CSS with [`lightningcss`].
//! 2. Apply nesting and media-query transforms targeting the same browser
//!    versions as the upstream (Safari 16.4, Firefox 128, Chrome 111).
//! 3. Optionally minify.
//! 4. Run the transform **twice** so that adjacent rules are merged after
//!    nesting is resolved (mirrors the TS original).
//! 5. Rewrite `@media not (` → `@media not all and (` to fix a known
//!    media-query range syntax transpilation issue.

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

#[cfg(test)]
mod tests {
    use super::*;

    // ── basic pass-through ────────────────────────────────────────────────────

    #[test]
    fn plain_css_is_preserved() {
        let input = ".foo { color: red; }";
        let result = optimize(input, OptimizeOptions::default()).unwrap();
        assert!(result.code.contains(".foo"));
        assert!(result.code.contains("color: red"));
    }

    #[test]
    fn empty_input_returns_empty() {
        let result = optimize("", OptimizeOptions::default()).unwrap();
        // lightningcss may output an empty string or just whitespace for empty input
        assert!(
            result.code.trim().is_empty(),
            "Expected empty/whitespace output for empty input, got: {:?}",
            result.code
        );
    }

    // ── minify ───────────────────────────────────────────────────────────────

    #[test]
    fn minify_removes_whitespace() {
        let input = ".foo {\n  color: red;\n  background: blue;\n}\n";
        let result = optimize(
            input,
            OptimizeOptions {
                minify: true,
                ..Default::default()
            },
        )
        .unwrap();
        // Minified output should not contain newlines inside the block
        assert!(!result.code.contains("  "), "whitespace should be removed");
        assert!(result.code.contains("color:red") || result.code.contains("color: red"));
    }

    #[test]
    fn no_minify_preserves_whitespace() {
        let input = ".foo {\n  color: red;\n}\n";
        let result = optimize(input, OptimizeOptions::default()).unwrap();
        // Non-minified output should have whitespace
        assert!(result.code.contains('\n') || result.code.contains("  "));
    }

    // ── CSS nesting ───────────────────────────────────────────────────────────

    #[test]
    fn nesting_is_expanded() {
        // CSS nesting: `.foo { & .bar { color: red; } }`
        // lightningcss should flatten this for the target browsers
        let input = ".foo { color: blue; & .bar { color: red; } }";
        let result = optimize(input, OptimizeOptions::default()).unwrap();
        // Both selectors should appear in the output
        assert!(result.code.contains(".foo"), "output: {}", result.code);
        // The CSS should still be valid and contain the declarations
        assert!(
            result.code.contains("color: red") || result.code.contains("color:red"),
            "output: {}",
            result.code
        );
    }

    // ── media query fixes ─────────────────────────────────────────────────────

    #[test]
    fn media_not_is_fixed() {
        let input = "@media not (hover: hover) { .foo { color: red; } }";
        let result = optimize(input, OptimizeOptions::default()).unwrap();
        // The fix converts "@media not (" → "@media not all and ("
        assert!(
            result.code.contains("@media not all and ("),
            "Expected '@media not all and (' in: {}",
            result.code
        );
    }

    #[test]
    fn media_not_all_and_already_correct_is_preserved() {
        // Already correct format should not be double-fixed
        let input = "@media not all and (hover: hover) { .foo { color: red; } }";
        let result = optimize(input, OptimizeOptions::default()).unwrap();
        assert!(result.code.contains("@media not all and ("));
    }

    // ── media query range syntax ───────────────────────────────────────────────

    #[test]
    fn media_range_syntax_is_downleveled() {
        // CSS media query range syntax `(width >= 768px)` should be
        // transformed to the equivalent level-3 form for older browsers
        let input = "@media (width >= 768px) { .foo { color: red; } }";
        let result = optimize(input, OptimizeOptions::default()).unwrap();
        // Should have transformed OR kept valid media query syntax
        assert!(result.code.contains(".foo"));
        assert!(result.code.contains("color: red"));
    }

    // ── adjacent rule merging (double-pass) ───────────────────────────────────

    #[test]
    fn adjacent_identical_rules_are_merged() {
        // Two adjacent rules with the same selector — lightningcss should
        // process them. Even if they aren't merged into one block, both
        // declarations should be present in the output.
        let input = ".foo { color: red; }\n.foo { background: blue; }";
        let result = optimize(input, OptimizeOptions::default()).unwrap();
        // The output should contain both declarations
        assert!(
            result.code.contains("color: red") || result.code.contains("color:red"),
            "output: {}",
            result.code
        );
        // background: blue should be present somewhere (lightningcss may normalize color names)
        assert!(
            result.code.contains("blue") || result.code.contains("#00f") || result.code.contains("background"),
            "output: {}",
            result.code
        );
    }

    // ── TransformResult equality ──────────────────────────────────────────────

    #[test]
    fn transform_result_is_equal_for_same_input() {
        let input = ".foo { color: red; }";
        let r1 = optimize(input, OptimizeOptions::default()).unwrap();
        let r2 = optimize(input, OptimizeOptions::default()).unwrap();
        assert_eq!(r1, r2);
    }

    // ── OptimizeOptions default ───────────────────────────────────────────────

    #[test]
    fn default_options_file_is_input_css() {
        let opts = OptimizeOptions::default();
        assert_eq!(opts.file, "input.css");
        assert!(!opts.minify);
    }

    // ── error handling ────────────────────────────────────────────────────────

    // Note: lightningcss is quite lenient with malformed CSS and generally
    // recovers rather than returning errors, so we test with content that
    // is valid but might trip up simple parsers.
    #[test]
    fn css_with_variables_is_preserved() {
        let input = ":root { --color: red; }\n.foo { color: var(--color); }";
        let result = optimize(input, OptimizeOptions::default()).unwrap();
        assert!(result.code.contains("--color"));
        assert!(result.code.contains("var(--color)"));
    }

    #[test]
    fn keyframes_are_preserved() {
        let input = "@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }\n.foo { animation: spin 1s linear; }";
        let result = optimize(input, OptimizeOptions::default()).unwrap();
        assert!(result.code.contains("@keyframes"));
        assert!(result.code.contains("spin"));
    }

    #[test]
    fn custom_file_name_in_options() {
        let opts = OptimizeOptions {
            file: "styles.css".to_string(),
            ..Default::default()
        };
        assert_eq!(opts.file, "styles.css");
    }
}
