//! Environment variable utilities.
//!
//! Rust port of
//! [`@tailwindcss-node/src/env.ts`](https://github.com/tailwindlabs/tailwindcss/blob/main/packages/%40tailwindcss-node/src/env.ts).
//!
//! Resolves the `DEBUG` environment variable into a boolean flag following the
//! same conventions as the upstream TypeScript version and the popular `debug`
//! npm package.
//!
//! ```
//! use farmfe_ecosystem_tailwindcss::env::resolve_debug;
//! // When DEBUG is not set
//! assert!(!resolve_debug(None));
//! // When DEBUG is "tailwindcss"
//! assert!(resolve_debug(Some("tailwindcss")));
//! ```

/// Resolve a `DEBUG`-style environment variable string into a boolean.
///
/// Mirrors the `resolveDebug()` function from `env.ts`.
///
/// # Rules (in order)
/// - `None` / not set → `false`
/// - `"true"` or `"1"` → `true`
/// - `"false"` or `"0"` → `false`
/// - `"*"` → `true`
/// - Comma-separated list:
///   - Contains `"-tailwindcss"` → `false`
///   - Contains `"tailwindcss"` (or `"tailwindcss:*"`) → `true`
/// - Anything else → `false`
pub fn resolve_debug(debug: Option<&str>) -> bool {
    match debug {
        None => false,
        Some("true" | "1") => true,
        Some("false" | "0") => false,
        Some("*") => true,
        Some(s) => {
            // Parse comma-separated debugger list, strip sub-namespaces
            let debuggers: Vec<&str> = s
                .split(',')
                .map(|d| d.split(':').next().unwrap_or(d).trim())
                .collect();

            if debuggers.contains(&"-tailwindcss") {
                return false;
            }
            if debuggers.contains(&"tailwindcss") {
                return true;
            }
            false
        }
    }
}

/// Returns the current value of the `DEBUG` flag by reading `std::env`.
///
/// Evaluates `resolve_debug` against the `DEBUG` environment variable at call
/// time, so the result may change across calls if the environment changes.
pub fn debug() -> bool {
    let val = std::env::var("DEBUG").ok();
    resolve_debug(val.as_deref())
}
