//! CSS utility class candidate scanner.
//!
//! Rust equivalent of the candidate-extraction logic used by the upstream
//! `tailwindcss` package.  Extracts potential utility class names from any
//! source text so that callers can feed them into the compiler.

use regex::Regex;
use std::sync::OnceLock;

/// Regex that matches a single candidate token inside source content.
///
/// Mirrors the pattern used in `@farmfe/plugin-tailwindcss` and in the
/// upstream JavaScript scanner.
static CANDIDATE_RE: OnceLock<Regex> = OnceLock::new();

fn candidate_re() -> &'static Regex {
  CANDIDATE_RE.get_or_init(|| {
    Regex::new(
      r#"(?:^|[\s'"`;{}\(])([!a-zA-Z0-9@\[\]:./_%\-][a-zA-Z0-9\-_:/.\[\]%#!]*[a-zA-Z0-9\]%])"#,
    )
    .expect("CANDIDATE_RE is a valid pattern")
  })
}

/// Extract potential Tailwind CSS utility class candidates from source text.
///
/// Returns a deduplicated list of candidate strings in order of first
/// appearance.  The caller is responsible for determining which candidates
/// correspond to real Tailwind utilities.
///
/// # Examples
///
/// ```
/// use farmfe_ecosystem_tailwindcss::scanner::extract_candidates;
///
/// let html = r#"<div class="flex items-center bg-blue-500 p-4">"#;
/// let candidates = extract_candidates(html);
/// assert!(candidates.contains(&"flex".to_string()));
/// assert!(candidates.contains(&"items-center".to_string()));
/// assert!(candidates.contains(&"bg-blue-500".to_string()));
/// ```
pub fn extract_candidates(content: &str) -> Vec<String> {
  let re = candidate_re();
  let mut seen = std::collections::HashSet::new();
  let mut result = Vec::new();

  for cap in re.captures_iter(content) {
    if let Some(m) = cap.get(1) {
      let candidate = m.as_str();
      // Filter out obvious non-candidates such as URL protocols and comments.
      if !candidate.starts_with("//")
        && !candidate.starts_with("/*")
        && !candidate.contains("://")
        && seen.insert(candidate.to_string())
      {
        result.push(candidate.to_string());
      }
    }
  }

  result
}
