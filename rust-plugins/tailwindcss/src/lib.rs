#![deny(clippy::all)]

use std::collections::HashSet;
use std::path;
use std::sync::{Arc, Mutex};

use farmfe_core::{
    config::Config,
    context::CompilationContext,
    module::{ModuleId, ModuleType},
    plugin::{Plugin, PluginTransformHookParam, PluginTransformHookResult},
    serde_json,
};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::lazy_static::lazy_static;
use farmfe_toolkit::regex::Regex;

use farmfe_ecosystem_tailwindcss::compile::{self, CompileOptions};

const PKG_NAME: &str = "@farmfe/plugin-tailwindcss";

lazy_static! {
    /// Regex matching CSS file extensions.
    static ref CSS_EXT_RE: Regex = Regex::new(r"\.css(\?|$)").unwrap();

    /// Regex matching file extensions that may contain TailwindCSS candidates.
    static ref CANDIDATE_EXT_RE: Regex = Regex::new(
        r"\.(js|jsx|ts|tsx|vue|svelte|html|mdx|md|astro|php|blade\.php|twig|erb|hbs|liquid|pug|slim|haml)(\?|$)"
    ).unwrap();

    /// Simple candidate scanner that extracts potential Tailwind CSS utility class
    /// names from source files.
    static ref CANDIDATE_RE: Regex = Regex::new(
        r#"(?:^|[\s'"`;{}\(])([!a-zA-Z0-9@\[\]:./_%\-][a-zA-Z0-9\-_:/.\[\]%#!]*[a-zA-Z0-9\]%])"#
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
}

#[farm_plugin]
pub struct FarmPluginTailwindCSS {
    _options: TailwindCSSOptions,
    /// Set of all candidates collected from scanned source files.
    candidates: Arc<Mutex<HashSet<String>>>,
    /// Set of module IDs that contributed candidates.
    candidate_sources: Arc<Mutex<HashSet<String>>>,
}

impl FarmPluginTailwindCSS {
    pub fn new(_config: &Config, options: String) -> Self {
        let parsed: TailwindCSSOptions =
            serde_json::from_str(&options).unwrap_or_default();

        Self {
            _options: parsed,
            candidates: Arc::new(Mutex::new(HashSet::new())),
            candidate_sources: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Scan `content` for TailwindCSS candidate class names.
    fn scan_candidates(&self, content: &str) -> Vec<String> {
        let mut result = Vec::new();
        for cap in CANDIDATE_RE.captures_iter(content) {
            if let Some(m) = cap.get(1) {
                let candidate = m.as_str().to_string();
                // Filter out obvious non-candidates
                if !candidate.starts_with("//")
                    && !candidate.starts_with("/*")
                    && !candidate.contains("://")
                {
                    result.push(candidate);
                }
            }
        }
        result
    }

    /// Check whether `id` looks like a CSS root file that should trigger
    /// TailwindCSS generation.
    fn is_css_root_file(id: &str) -> bool {
        CSS_EXT_RE.is_match(id) && !id.contains("node_modules")
    }

    /// Get the file extension from a module id (stripping query strings).
    #[cfg(test)]
    fn get_extension(id: &str) -> &str {
        let filename = id.split('?').next().unwrap_or(id);
        filename
            .rsplit('.')
            .next()
            .unwrap_or("")
    }
}

impl Plugin for FarmPluginTailwindCSS {
    fn name(&self) -> &str {
        "FarmPluginTailwindCSS"
    }

    fn priority(&self) -> i32 {
        101
    }

    fn config(
        &self,
        config: &mut Config,
    ) -> farmfe_core::error::Result<Option<()>> {
        // Ensure .css is in the resolve extensions
        if config.resolve.extensions.iter().all(|e| e != "css") {
            config.resolve.extensions.push("css".to_string());
        }
        Ok(Some(()))
    }

    fn transform(
        &self,
        param: &PluginTransformHookParam,
        context: &Arc<CompilationContext>,
    ) -> farmfe_core::error::Result<Option<PluginTransformHookResult>> {
        let resolved_path = param.resolved_path;

        // ── Step 1: Scan non-CSS source files for candidates ────────────
        if CANDIDATE_EXT_RE.is_match(resolved_path) {
            let new_candidates = self.scan_candidates(&param.content);

            if !new_candidates.is_empty() {
                let mut candidates = self.candidates.lock().unwrap();
                for c in new_candidates {
                    candidates.insert(c);
                }
                drop(candidates);

                self.candidate_sources
                    .lock()
                    .unwrap()
                    .insert(resolved_path.to_string());
            }

            // Pass through — we don't modify the source
            return Ok(None);
        }

        // ── Step 2: Process CSS files that contain TailwindCSS directives ─
        if param.module_type != ModuleType::Css
            || !Self::is_css_root_file(resolved_path)
        {
            return Ok(None);
        }

        // Check if this CSS file contains a tailwindcss directive
        let has_tailwind_directive = param.content.contains("@import \"tailwindcss\"")
            || param.content.contains("@import 'tailwindcss'")
            || param.content.contains("@tailwind")
            || param.content.contains("@apply ");

        if !has_tailwind_directive {
            return Ok(None);
        }

        let base = path::Path::new(resolved_path)
            .parent()
            .unwrap_or(path::Path::new("."))
            .to_string_lossy()
            .into_owned();

        let sourcemap_enabled =
            context.sourcemap_enabled(&param.module_id.to_string());

        // Compile the CSS using the ecosystem crate
        let compiler_result = compile::compile(
            &param.content,
            CompileOptions {
                base,
                from: if sourcemap_enabled {
                    Some(resolved_path.to_string())
                } else {
                    None
                },
                should_rewrite_urls: true,
                on_dependency: Box::new(move |_dep| {
                    // Dependencies are tracked through the module graph
                }),
                ..Default::default()
            },
        );

        match compiler_result {
            Ok(mut compiler) => {
                // Check if the CSS uses Tailwind features
                if !compiler.features.has_any_output_feature() {
                    return Ok(None);
                }

                // Gather candidates and build
                let candidates: Vec<String> = {
                    let guard = self.candidates.lock().unwrap();
                    guard.iter().cloned().collect()
                };

                let css = compiler.build(&candidates);
                let source_map = compiler.build_source_map();

                // Add watch files for candidate sources
                let sources: Vec<String> = {
                    let guard = self.candidate_sources.lock().unwrap();
                    guard.iter().cloned().collect()
                };
                for source in &sources {
                    let _ = context.add_watch_files(
                        ModuleId::new(resolved_path, "", &context.config.root),
                        vec![ModuleId::new(source, "", &context.config.root)],
                    );
                }

                Ok(Some(PluginTransformHookResult {
                    content: css,
                    source_map,
                    module_type: Some(ModuleType::Css),
                    ignore_previous_source_map: false,
                }))
            }
            Err(e) => {
                eprintln!("[{PKG_NAME}] Failed to compile TailwindCSS: {e}");
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_candidates_basic() {
        let plugin = FarmPluginTailwindCSS {
            _options: TailwindCSSOptions::default(),
            candidates: Arc::new(Mutex::new(HashSet::new())),
            candidate_sources: Arc::new(Mutex::new(HashSet::new())),
        };

        let content = r#"<div class="bg-red-500 text-white p-4 hover:bg-blue-500"></div>"#;
        let candidates = plugin.scan_candidates(content);
        assert!(!candidates.is_empty());
        // Check that at least some expected candidates are found
        assert!(candidates.iter().any(|c| c.contains("bg-red")));
    }

    #[test]
    fn scan_candidates_ignores_urls() {
        let plugin = FarmPluginTailwindCSS {
            _options: TailwindCSSOptions::default(),
            candidates: Arc::new(Mutex::new(HashSet::new())),
            candidate_sources: Arc::new(Mutex::new(HashSet::new())),
        };

        let content = r#"const url = "https://example.com";"#;
        let candidates = plugin.scan_candidates(content);
        // Should not contain URL-like candidates
        for c in &candidates {
            assert!(!c.contains("://"), "Found URL-like candidate: {c}");
        }
    }

    #[test]
    fn is_css_root_file_works() {
        assert!(FarmPluginTailwindCSS::is_css_root_file("/src/app.css"));
        assert!(FarmPluginTailwindCSS::is_css_root_file("/src/styles.css?v=1"));
        assert!(!FarmPluginTailwindCSS::is_css_root_file(
            "/node_modules/lib/style.css"
        ));
        assert!(!FarmPluginTailwindCSS::is_css_root_file("/src/app.js"));
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
}
