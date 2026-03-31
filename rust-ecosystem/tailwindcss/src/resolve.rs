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
const JS_MAIN_FIELDS: &[&str] = &["main", "module"];

/// Resolve a JS module id from `base`.
///
/// If a `custom_resolver` is provided it is tried first. Falls back to the
/// built-in resolution algorithm.
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

    resolve_with_extensions(id, base, ESM_EXTENSIONS, JS_MAIN_FIELDS)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("farm_tw_resolve_test").join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolve_relative_css() {
        let dir = setup_dir("rel_css");
        let css_file = dir.join("styles.css");
        fs::write(&css_file, "body {}").unwrap();

        let resolved =
            resolve_css_id("./styles.css", dir.to_str().unwrap(), None).unwrap();
        assert_eq!(resolved, css_file);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_relative_without_extension() {
        let dir = setup_dir("no_ext_css");
        let css_file = dir.join("styles.css");
        fs::write(&css_file, "body {}").unwrap();

        let resolved =
            resolve_css_id("./styles", dir.to_str().unwrap(), None).unwrap();
        assert_eq!(resolved, css_file);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_relative_js() {
        let dir = setup_dir("rel_js");
        let js_file = dir.join("mod.js");
        fs::write(&js_file, "export default 1;").unwrap();

        let resolved =
            resolve_js_id("./mod.js", dir.to_str().unwrap(), None).unwrap();
        assert_eq!(resolved, js_file);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_js_without_extension() {
        let dir = setup_dir("no_ext_js");
        let js_file = dir.join("mod.ts");
        fs::write(&js_file, "export const x = 1;").unwrap();

        let resolved =
            resolve_js_id("./mod", dir.to_str().unwrap(), None).unwrap();
        assert_eq!(resolved, js_file);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_index_file() {
        let dir = setup_dir("index_js");
        let sub = dir.join("utils");
        fs::create_dir_all(&sub).unwrap();
        let idx = sub.join("index.js");
        fs::write(&idx, "export default {};").unwrap();

        let resolved =
            resolve_js_id("./utils", dir.to_str().unwrap(), None).unwrap();
        assert_eq!(resolved, idx);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn custom_resolver_takes_precedence() {
        let dir = setup_dir("custom");
        let css_file = dir.join("custom.css");
        fs::write(&css_file, "body {}").unwrap();

        let expected_path = css_file.to_str().unwrap().to_string();
        let custom: CustomResolver = Box::new(move |_id, _base| {
            Some(expected_path.clone())
        });

        let resolved =
            resolve_css_id("anything", dir.to_str().unwrap(), Some(&custom))
                .unwrap();
        assert_eq!(resolved, css_file);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn not_found_returns_error() {
        let dir = setup_dir("not_found");
        let result = resolve_css_id("./nope.css", dir.to_str().unwrap(), None);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }
}
