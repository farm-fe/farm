//! Cross-platform path normalization.
//!
//! Rust port of
//! [`@tailwindcss-node/src/normalize-path.ts`](https://github.com/tailwindlabs/tailwindcss/blob/main/packages/%40tailwindcss-node/src/normalize-path.ts)
//! which is itself an inlined version of the `normalize-path` npm package.

/// Normalize a file-system path to use forward slashes.
///
/// Windows UNC paths (`\\\\server\\share`) are preserved with a leading `//`.
/// Trailing separators are stripped unless the path is the root `/`.
pub fn normalize_path(original_path: &str) -> String {
    let normalized = normalize_path_base(original_path, true);

    // Make sure Windows network share paths are normalized properly.
    // They have to begin with two slashes or they won't resolve correctly.
    if original_path.starts_with("\\\\")
        && normalized.starts_with('/')
        && !normalized.starts_with("//")
    {
        return format!("/{normalized}");
    }

    normalized
}

fn normalize_path_base(path: &str, strip_trailing: bool) -> String {
    if path == "\\" || path == "/" {
        return "/".to_string();
    }

    if path.len() <= 1 {
        return path.to_string();
    }

    // Handle win32 namespaces (\\?\ or \\.\)
    let mut prefix = String::new();
    let mut remaining = path;

    if path.len() > 4 {
        let bytes = path.as_bytes();
        if bytes[3] == b'\\' {
            let ch = bytes[2];
            if (ch == b'?' || ch == b'.') && &path[..2] == "\\\\" {
                remaining = &path[2..];
                prefix = "//".to_string();
            }
        }
    }

    // Check if the remaining string starts with a separator (produces a
    // leading empty segment in the TS `split(/[/\\]+/)` call).
    let starts_with_sep = remaining
        .as_bytes()
        .first()
        .is_some_and(|&b| b == b'/' || b == b'\\');

    // Split on one-or-more separators (like the TS `/[/\\]+/` regex)
    let segs: Vec<&str> = remaining
        .split(['/', '\\'])
        .filter(|s| !s.is_empty())
        .collect();

    let joined = segs.join("/");

    let body = if starts_with_sep {
        format!("/{joined}")
    } else {
        joined
    };

    if strip_trailing {
        format!("{prefix}{body}")
    } else {
        // Preserve trailing slash if original had one
        let last_byte = remaining.as_bytes().last().copied().unwrap_or(0);
        if last_byte == b'/' || last_byte == b'\\' {
            format!("{prefix}{body}/")
        } else {
            format!("{prefix}{body}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_slashes_are_kept() {
        assert_eq!(normalize_path("foo/bar/baz"), "foo/bar/baz");
    }

    #[test]
    fn backslashes_are_replaced() {
        assert_eq!(normalize_path("foo\\bar\\baz"), "foo/bar/baz");
    }

    #[test]
    fn mixed_slashes() {
        assert_eq!(normalize_path("foo/bar\\baz"), "foo/bar/baz");
    }

    #[test]
    fn trailing_slash_stripped() {
        assert_eq!(normalize_path("foo/bar/"), "foo/bar");
    }

    #[test]
    fn root_slash_preserved() {
        assert_eq!(normalize_path("/"), "/");
        assert_eq!(normalize_path("\\"), "/");
    }

    #[test]
    fn single_char_path() {
        assert_eq!(normalize_path("a"), "a");
    }

    #[test]
    fn empty_string() {
        assert_eq!(normalize_path(""), "");
    }

    #[test]
    fn win32_unc_path() {
        assert_eq!(normalize_path("\\\\server\\share"), "//server/share");
    }

    #[test]
    fn win32_namespace_prefix() {
        assert_eq!(
            normalize_path("\\\\?\\C:\\foo\\bar"),
            "//?/C:/foo/bar"
        );
        assert_eq!(
            normalize_path("\\\\.\\C:\\foo\\bar"),
            "//./C:/foo/bar"
        );
    }

    #[test]
    fn multiple_consecutive_separators() {
        assert_eq!(normalize_path("foo//bar///baz"), "foo/bar/baz");
    }
}
