pub(crate) use chars::*;
pub(crate) use map_view::*;
pub(crate) use strings::*;

mod chars;
mod map_view;
mod strings;

#[allow(clippy::case_sensitive_file_extension_comparisons)]
pub(crate) fn is_plain_css_import(url: &str) -> bool {
    if url.len() < 5 {
        return false;
    }

    let lower = url.to_ascii_lowercase();

    lower.ends_with(".css")
        || lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("//")
}

pub(crate) fn opposite_bracket(b: char) -> char {
    debug_assert!(matches!(b, '(' | '{' | '[' | ')' | '}' | ']'));
    match b {
        '(' => ')',
        '{' => '}',
        '[' => ']',
        ')' => '(',
        '}' => '{',
        ']' => '[',
        _ => unreachable!(),
    }
}

pub(crate) fn is_special_function(s: &str) -> bool {
    s.starts_with("calc(")
        || s.starts_with("var(")
        || s.starts_with("env(")
        || s.starts_with("min(")
        || s.starts_with("max(")
        || s.starts_with("clamp(")
}

/// Trim ASCII whitespace from both sides of string.
///
/// If [excludeEscape] is `true`, this doesn't trim whitespace included in a CSS
/// escape.
pub(crate) fn trim_ascii(
    s: &str,
    // default=false
    exclude_escape: bool,
) -> &str {
    match s.chars().position(|c| !c.is_ascii_whitespace()) {
        Some(start) => &s[start..=last_non_whitespace(s, exclude_escape).unwrap()],
        None => "",
    }
}

fn last_non_whitespace(s: &str, exclude_escape: bool) -> Option<usize> {
    let mut idx = s.len() - 1;
    for c in s.chars().rev() {
        if !c.is_ascii_whitespace() {
            if exclude_escape && idx != 0 && idx != s.len() - 1 && c == '\\' {
                return Some(idx + 1);
            } else {
                return Some(idx);
            }
        }

        idx -= 1;
    }

    None
}

pub(crate) fn to_sentence<T: Into<String>>(mut elems: Vec<T>, conjunction: &'static str) -> String {
    debug_assert!(
        !elems.is_empty(),
        "expected sentence to contain at least one element"
    );
    if elems.len() == 1 {
        return elems.pop().unwrap().into();
    }

    let last = elems.pop().unwrap();

    format!(
        "{} {conjunction} {}",
        elems
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>()
            .join(", "),
        last.into(),
        conjunction = conjunction,
    )
}
