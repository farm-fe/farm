//! JS/TS module dependency tracing.
//!
//! Rust port of
//! [`@tailwindcss-node/src/get-module-dependencies.ts`](https://github.com/tailwindlabs/tailwindcss/blob/main/packages/%40tailwindcss-node/src/get-module-dependencies.ts).
//!
//! Recursively follows `import` / `require` / `export … from` statements in
//! JavaScript and TypeScript files to produce the full set of transitive
//! dependencies.

use regex::Regex;
use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

// ── patterns ────────────────────────────────────────────────────────────────

static DEPENDENCY_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
  vec![
    Regex::new(r#"import[\s\S]*?['"](.{3,}?)['"]"#).unwrap(),
    Regex::new(r#"import[\s\S]*from[\s\S]*?['"](.{3,}?)['"]"#).unwrap(),
    Regex::new(r#"export[\s\S]*from[\s\S]*?['"](.{3,}?)['"]"#).unwrap(),
    Regex::new(r#"require\(['"`](.+)['"`]\)"#).unwrap(),
  ]
});

const JS_EXTENSIONS: &[&str] = &[".js", ".cjs", ".mjs"];
const JS_RESOLUTION_ORDER: &[&str] = &[
  "", ".js", ".cjs", ".mjs", ".ts", ".cts", ".mts", ".jsx", ".tsx",
];
const TS_RESOLUTION_ORDER: &[&str] = &[
  "", ".ts", ".cts", ".mts", ".tsx", ".js", ".cjs", ".mjs", ".jsx",
];

// ── resolution helpers ──────────────────────────────────────────────────────

fn resolve_with_extension(file: &Path, extensions: &[&str]) -> Option<PathBuf> {
  // Try to find `./a.ts`, `./a.cts`, … from `./a`
  for ext in extensions {
    // When ext is empty, use the original path.
    // Otherwise, append the extension (don't replace the existing one).
    let full = if ext.is_empty() {
      file.to_path_buf()
    } else {
      let mut s = file.as_os_str().to_os_string();
      s.push(ext);
      PathBuf::from(s)
    };
    if full.is_file() {
      return Some(full);
    }
  }

  // Try to find `./a/index.js` from `./a`
  for ext in extensions {
    if ext.is_empty() {
      continue;
    }
    let mut idx_name = String::from("index");
    idx_name.push_str(ext);
    let full = file.join(idx_name);
    if full.is_file() {
      return Some(full);
    }
  }

  None
}

fn trace_dependencies(
  seen: &mut HashSet<PathBuf>,
  filename: &str,
  base: &Path,
  ext: &str,
) -> io::Result<()> {
  let extensions = if JS_EXTENSIONS.contains(&ext) {
    JS_RESOLUTION_ORDER
  } else {
    TS_RESOLUTION_ORDER
  };

  let absolute_file = match resolve_with_extension(&base.join(filename), extensions) {
    Some(p) => p,
    None => return Ok(()), // File doesn't exist
  };

  // Prevent infinite loops from circular dependencies
  if seen.contains(&absolute_file) {
    return Ok(());
  }

  seen.insert(absolute_file.clone());

  let new_base = absolute_file
    .parent()
    .unwrap_or(Path::new("."))
    .to_path_buf();
  let new_ext = absolute_file
    .extension()
    .map(|e| format!(".{}", e.to_string_lossy()))
    .unwrap_or_default();

  let contents = std::fs::read_to_string(&absolute_file)?;

  for pattern in DEPENDENCY_PATTERNS.iter() {
    for cap in pattern.captures_iter(&contents) {
      if let Some(m) = cap.get(1) {
        let dep = m.as_str();
        // Only follow relative imports
        if dep.starts_with('.') {
          trace_dependencies(seen, dep, &new_base, &new_ext)?;
        }
      }
    }
  }

  Ok(())
}

// ── public API ──────────────────────────────────────────────────────────────

/// Trace all transitive dependencies of a JS/TS module.
///
/// Returns an unordered set of absolute file paths. The order is **not**
/// guaranteed to match source order or be stable across runs.
pub fn get_module_dependencies(absolute_file_path: &Path) -> io::Result<Vec<PathBuf>> {
  let mut seen = HashSet::new();

  let base = absolute_file_path
    .parent()
    .unwrap_or(Path::new("."))
    .to_path_buf();
  let ext = absolute_file_path
    .extension()
    .map(|e| format!(".{}", e.to_string_lossy()))
    .unwrap_or_default();

  trace_dependencies(
    &mut seen,
    absolute_file_path.to_str().unwrap_or_default(),
    &base,
    &ext,
  )?;

  Ok(seen.into_iter().collect())
}
