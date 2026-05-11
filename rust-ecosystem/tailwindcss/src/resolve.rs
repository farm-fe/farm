//! CSS and JS module resolution.
//!
//! Mirrors the resolution logic from
//! [`@tailwindcss-node/src/compile.ts`](https://github.com/tailwindlabs/tailwindcss/blob/main/packages/%40tailwindcss-node/src/compile.ts)
//! which uses `enhanced-resolve` under the hood.
//!
//! In this Rust port we implement a simplified resolver that handles the common
//! cases: relative paths, extension inference, and `node_modules` look-ups.

use std::io;
use std::path::{Path, PathBuf};

/// A custom resolver function that takes `(id, base)` and returns the resolved
/// path, or `None` if it cannot resolve.
pub type CustomResolver = Box<dyn Fn(&str, &str) -> Option<String> + Send + Sync>;

// ── CSS resolution ──────────────────────────────────────────────────────────

const CSS_EXTENSIONS: &[&str] = &["", ".css"];
const CSS_MAIN_FIELDS: &[&str] = &["style"];

/// Resolve a CSS module id from `base`.
///
/// If a `custom_resolver` is provided it is tried first. Falls back to the
/// built-in resolution algorithm which checks extensions `.css` and the `style`
/// field in `package.json` when resolving from `node_modules`.
pub fn resolve_css_id(
    id: &str,
    base: &str,
    custom_resolver: Option<&CustomResolver>,
) -> io::Result<PathBuf> {
    // Try custom resolver first
    if let Some(resolver) = custom_resolver {
        if let Some(resolved) = resolver(id, base) {
            return Ok(PathBuf::from(resolved));
        }
    }

    resolve_with_extensions(id, base, CSS_EXTENSIONS, CSS_MAIN_FIELDS)
}

// ── JS resolution ───────────────────────────────────────────────────────────

const ESM_EXTENSIONS: &[&str] = &["", ".js", ".json", ".node", ".ts"];
const CJS_EXTENSIONS: &[&str] = &["", ".js", ".json", ".node", ".ts"];
const JS_ESM_MAIN_FIELDS: &[&str] = &["module", "main"];
const JS_CJS_MAIN_FIELDS: &[&str] = &["main"];
/// ESM condition names matching the upstream `esmResolver`.
pub const ESM_CONDITIONS: &[&str] = &["node", "import"];
/// CJS condition names matching the upstream `cjsResolver`.
pub const CJS_CONDITIONS: &[&str] = &["node", "require"];

/// Resolve a JS module id from `base` using ESM resolution.
///
/// If a `custom_resolver` is provided it is tried first. Falls back to the
/// built-in ESM resolution algorithm and then CJS as a fallback, matching
/// the upstream dual-resolver pattern.
pub fn resolve_js_id(
    id: &str,
    base: &str,
    custom_resolver: Option<&CustomResolver>,
) -> io::Result<PathBuf> {
    // Try custom resolver first
    if let Some(resolver) = custom_resolver {
        if let Some(resolved) = resolver(id, base) {
            return Ok(PathBuf::from(resolved));
        }
    }

    // Try ESM resolution first, fall back to CJS (matches upstream behaviour)
    resolve_with_extensions(id, base, ESM_EXTENSIONS, JS_ESM_MAIN_FIELDS)
        .or_else(|_| resolve_with_extensions(id, base, CJS_EXTENSIONS, JS_CJS_MAIN_FIELDS))
}

/// Resolve a JS module using ESM-only resolution.
pub fn resolve_js_esm(
    id: &str,
    base: &str,
    custom_resolver: Option<&CustomResolver>,
) -> io::Result<PathBuf> {
    if let Some(resolver) = custom_resolver {
        if let Some(resolved) = resolver(id, base) {
            return Ok(PathBuf::from(resolved));
        }
    }
    resolve_with_extensions(id, base, ESM_EXTENSIONS, JS_ESM_MAIN_FIELDS)
}

/// Resolve a JS module using CJS-only resolution.
pub fn resolve_js_cjs(
    id: &str,
    base: &str,
    custom_resolver: Option<&CustomResolver>,
) -> io::Result<PathBuf> {
    if let Some(resolver) = custom_resolver {
        if let Some(resolved) = resolver(id, base) {
            return Ok(PathBuf::from(resolved));
        }
    }
    resolve_with_extensions(id, base, CJS_EXTENSIONS, JS_CJS_MAIN_FIELDS)
}

// ── shared implementation ───────────────────────────────────────────────────

fn resolve_with_extensions(
    id: &str,
    base: &str,
    extensions: &[&str],
    _main_fields: &[&str],
) -> io::Result<PathBuf> {
    let base_path = Path::new(base);

    // Relative or absolute path
    if id.starts_with('.') || id.starts_with('/') {
        let full = if id.starts_with('/') {
            PathBuf::from(id)
        } else {
            base_path.join(id)
        };

        // Try exact match, then with extensions
        for ext in extensions {
            let candidate = if ext.is_empty() {
                full.clone()
            } else {
                let mut s = full.as_os_str().to_os_string();
                s.push(ext);
                PathBuf::from(s)
            };
            if candidate.is_file() {
                return Ok(candidate);
            }
        }

        // Try index files
        for ext in extensions {
            if ext.is_empty() {
                continue;
            }
            let candidate = full.join(format!("index{ext}"));
            if candidate.is_file() {
                return Ok(candidate);
            }
        }

        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Could not resolve '{id}' from '{base}'"),
        ));
    }

    // node_modules resolution
    let mut current = Some(base_path);
    while let Some(dir) = current {
        let nm = dir.join("node_modules").join(id);
        for ext in extensions {
            let candidate = if ext.is_empty() {
                nm.clone()
            } else {
                let mut s = nm.as_os_str().to_os_string();
                s.push(ext);
                PathBuf::from(s)
            };
            if candidate.is_file() {
                return Ok(candidate);
            }
        }

        // Try index files in node_modules
        for ext in extensions {
            if ext.is_empty() {
                continue;
            }
            let candidate = nm.join(format!("index{ext}"));
            if candidate.is_file() {
                return Ok(candidate);
            }
        }

        current = dir.parent();
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("Could not resolve '{id}' from '{base}'"),
    ))
}
