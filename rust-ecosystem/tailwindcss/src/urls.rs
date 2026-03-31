//! CSS `url()` and `image-set()` rewriting.
//!
//! Rust port of
//! [`@tailwindcss-node/src/urls.ts`](https://github.com/tailwindlabs/tailwindcss/blob/main/packages/%40tailwindcss-node/src/urls.ts).
//!
//! When Tailwind CSS resolves `@import`-ed stylesheets the relative URLs inside
//! those sheets need to be rebased so that they still point at the correct
//! assets from the perspective of the *root* stylesheet.

use regex::Regex;
use std::sync::LazyLock;

use crate::normalize_path::normalize_path;

// ── regex patterns ──────────────────────────────────────────────────────────

static CSS_URL_RE: LazyLock<Regex> = LazyLock::new(|| {
    // url( ... ) — but NOT preceded by @import
    Regex::new(r#"(?:^|[^\w\-\x{0080}-\x{ffff}])url\((\s*('[^']+'|"[^"]+")\s*|[^'")\s]+)\)"#)
        .unwrap()
});

static CSS_IMAGE_SET_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"image-set\(((?:[^)]*(?:\([^)]*\))?)*)\)"#).unwrap()
});

static CSS_URL_INNER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"url\((\s*('[^']+'|"[^"]+")\s*|[^'")\s]+)\)"#).unwrap()
});

static DATA_URL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)^\s*data:"#).unwrap());

static EXTERNAL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^([a-z]+:)?//"#).unwrap());

static FUNCTION_CALL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^[A-Z_][\.\w-]*\("#).unwrap());

// ── helpers ─────────────────────────────────────────────────────────────────

fn is_data_url(url: &str) -> bool {
    DATA_URL_RE.is_match(url)
}

fn is_external_url(url: &str) -> bool {
    EXTERNAL_RE.is_match(url)
}

fn should_skip(raw_url: &str) -> bool {
    if raw_url.is_empty() {
        return true;
    }
    let first = raw_url.as_bytes()[0];
    is_external_url(raw_url)
        || is_data_url(raw_url)
        || !first.is_ascii_alphanumeric() && first != b'.'
        || FUNCTION_CALL_RE.is_match(raw_url)
}

/// Rebase a single relative URL from `base` to `root`.
fn rebase_url(url: &str, base: &str, root: &str) -> String {
    if url.starts_with('/') {
        return url.to_string();
    }

    // Join base + url using forward slashes (posix-style)
    let base_norm = normalize_path(base);
    let absolute_url = join_posix(&base_norm, url);

    let root_norm = normalize_path(root);
    let mut relative = make_relative(&root_norm, &absolute_url);

    // path.relative removes the leading "./" — add it back if needed
    if !relative.starts_with('.') {
        relative = format!("./{relative}");
    }

    relative
}

/// Simple POSIX-style path joining.
fn join_posix(base: &str, relative: &str) -> String {
    let mut parts: Vec<&str> = base.split('/').collect();
    for seg in relative.split('/') {
        match seg {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

/// Compute the relative path from `from` to `to` (POSIX-style).
fn make_relative(from: &str, to: &str) -> String {
    let from_parts: Vec<&str> = from.split('/').filter(|s| !s.is_empty()).collect();
    let to_parts: Vec<&str> = to.split('/').filter(|s| !s.is_empty()).collect();

    // Find common prefix length
    let common = from_parts
        .iter()
        .zip(to_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let ups = from_parts.len() - common;
    let mut result: Vec<&str> = vec![".."; ups];
    for part in &to_parts[common..] {
        result.push(part);
    }

    if result.is_empty() {
        ".".to_string()
    } else {
        result.join("/")
    }
}

/// Replace a single `url(...)` token, preserving surrounding quotes.
fn do_url_replace(raw_url: &str, matched: &str, base: &str, root: &str) -> String {
    let (wrap, inner) = strip_quotes(raw_url);

    if should_skip(inner) {
        return matched.to_string();
    }

    let new_url = rebase_url(inner, base, root);

    let wrap_char = if (wrap.is_empty() && new_url != url_encode_simple(&new_url))
        || (wrap == "'" && new_url.contains('\''))
    {
        "\""
    } else {
        wrap
    };

    let escaped = if wrap_char == "\"" {
        new_url.replace('"', "\\\"")
    } else {
        new_url
    };

    format!("url({wrap_char}{escaped}{wrap_char})")
}

fn strip_quotes(s: &str) -> (&str, &str) {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        (&s[..1], &s[1..s.len() - 1])
    } else {
        ("", s)
    }
}

fn url_encode_simple(s: &str) -> String {
    s.replace(' ', "%20")
}

// ── public API ──────────────────────────────────────────────────────────────

/// Rewrite relative `url()` and `image-set()` references in `css` so that they
/// resolve correctly when the stylesheet is served from `root` but the CSS was
/// originally authored relative to `base`.
pub fn rewrite_urls(css: &str, base: &str, root: &str) -> String {
    if !css.contains("url(") && !css.contains("image-set(") {
        return css.to_string();
    }

    let mut result = css.to_string();

    // Process image-set(...) first – it may contain url() tokens inside.
    result = CSS_IMAGE_SET_RE
        .replace_all(&result, |caps: &regex::Captures| {
            let inner = &caps[1];
            rewrite_image_set_inner(inner, base, root)
        })
        .into_owned();

    // Process standalone url(...) tokens.
    result = rewrite_css_urls(&result, base, root);

    result
}

/// Rewrite url() tokens inside an image-set() argument list.
fn rewrite_image_set_inner(inner: &str, base: &str, root: &str) -> String {
    let replaced = CSS_URL_INNER_RE
        .replace_all(inner, |caps: &regex::Captures| {
            let raw = &caps[1];
            do_url_replace(raw, &caps[0], base, root)
        })
        .into_owned();

    // Also handle bare image references (not wrapped in url())
    // Split by comma, process each candidate
    let parts: Vec<&str> = replaced.split(',').collect();
    let processed: Vec<String> = parts
        .iter()
        .map(|part| {
            let trimmed = part.trim();
            // If it already contains url(...), leave it as-is
            if trimmed.starts_with("url(") || trimmed.contains("url(") {
                return part.to_string();
            }
            // Check if it looks like a gradient or function call
            if FUNCTION_CALL_RE.is_match(trimmed)
                || trimmed.starts_with("linear-gradient")
                || trimmed.starts_with("radial-gradient")
            {
                return part.to_string();
            }
            // Try to rewrite bare image references
            let mut tokens = trimmed.splitn(2, char::is_whitespace);
            if let Some(url_part) = tokens.next() {
                let (wrap, url_inner) = strip_quotes(url_part);
                if !url_inner.is_empty() && !should_skip(url_inner) {
                    let new_url = rebase_url(url_inner, base, root);
                    let descriptor = tokens.next().unwrap_or("");
                    let desc_str = if descriptor.is_empty() {
                        String::new()
                    } else {
                        format!(" {descriptor}")
                    };
                    if wrap.is_empty() {
                        return format!(" url({new_url}){desc_str}");
                    } else {
                        return format!(" url({wrap}{new_url}{wrap}){desc_str}");
                    }
                }
            }
            part.to_string()
        })
        .collect();

    format!("image-set({})", processed.join(","))
}

/// Rewrite standalone `url(...)` tokens in a CSS string.
fn rewrite_css_urls(css: &str, base: &str, root: &str) -> String {
    CSS_URL_RE
        .replace_all(css, |caps: &regex::Captures| {
            let full_match = &caps[0];
            let raw_url = &caps[1];

            // Preserve the prefix before url(
            let url_start = full_match.find("url(").unwrap_or(0);
            let prefix = &full_match[..url_start];

            let replaced = do_url_replace(raw_url, &full_match[url_start..], base, root);
            format!("{prefix}{replaced}")
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_urls_returns_input_unchanged() {
        let css = ".foo { color: red; }";
        assert_eq!(rewrite_urls(css, "/root/a", "/root"), css);
    }

    #[test]
    fn relative_url_is_rebased() {
        let css = ".foo { background: url(./image.jpg); }";
        let result = rewrite_urls(css, "/root/foo/bar", "/root");
        assert!(result.contains("foo/bar/image.jpg"));
    }

    #[test]
    fn parent_relative_url_is_rebased() {
        let css = ".foo { background: url(../image.jpg); }";
        let result = rewrite_urls(css, "/root/foo/bar", "/root");
        assert!(result.contains("foo/image.jpg"));
    }

    #[test]
    fn absolute_url_is_not_changed() {
        let css = ".foo { background: url(/image.jpg); }";
        let result = rewrite_urls(css, "/root/foo/bar", "/root");
        assert!(result.contains("url(/image.jpg)"));
    }

    #[test]
    fn external_url_is_not_changed() {
        let css = ".foo { background: url(http://example.com/image.jpg); }";
        let result = rewrite_urls(css, "/root/foo/bar", "/root");
        assert!(result.contains("http://example.com/image.jpg"));
    }

    #[test]
    fn data_uri_is_not_changed() {
        let css = ".foo { background: url('data:image/png;base64,abc=='); }";
        let result = rewrite_urls(css, "/root/foo/bar", "/root");
        assert!(result.contains("data:image/png;base64,abc=="));
    }

    #[test]
    fn function_call_in_url_is_not_changed() {
        let css = ".foo { background: url(var(--foo)); }";
        let result = rewrite_urls(css, "/root/foo/bar", "/root");
        assert!(result.contains("var(--foo)"));
    }

    #[test]
    fn fragment_is_not_changed() {
        let css = ".foo { background: url(#dont-touch-this); }";
        let result = rewrite_urls(css, "/root/foo/bar", "/root");
        assert!(result.contains("#dont-touch-this"));
    }

    #[test]
    fn join_posix_basic() {
        assert_eq!(join_posix("/root/foo", "bar.jpg"), "/root/foo/bar.jpg");
        assert_eq!(
            join_posix("/root/foo", "./bar.jpg"),
            "/root/foo/bar.jpg"
        );
        assert_eq!(join_posix("/root/foo", "../bar.jpg"), "/root/bar.jpg");
    }

    #[test]
    fn make_relative_basic() {
        assert_eq!(
            make_relative("/root", "/root/foo/bar/image.jpg"),
            "foo/bar/image.jpg"
        );
    }

    #[test]
    fn rebase_url_basic() {
        let result = rebase_url("./image.jpg", "/root/foo/bar", "/root");
        assert_eq!(result, "./foo/bar/image.jpg");
    }

    #[test]
    fn rebase_url_parent() {
        let result = rebase_url("../image.jpg", "/root/foo/bar", "/root");
        assert_eq!(result, "./foo/image.jpg");
    }

    #[test]
    fn rebase_url_plain_filename() {
        let result = rebase_url("image.jpg", "/root/foo/bar", "/root");
        assert_eq!(result, "./foo/bar/image.jpg");
    }
}
