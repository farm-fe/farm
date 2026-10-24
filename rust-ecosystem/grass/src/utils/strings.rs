use super::{is_name, is_name_start};

pub(crate) fn is_ident(s: &str) -> bool {
    let mut chars = s.chars().peekable();
    match chars.next() {
        Some(c) if is_name_start(c) && !c.is_numeric() => {}
        Some(..) | None => return false,
    }
    while let Some(c) = chars.next() {
        if c == '\\' {
            for _ in 0..6 {
                let next = match chars.next() {
                    Some(t) => t,
                    None => return true,
                };
                if !next.is_ascii_hexdigit() {
                    break;
                }
            }
            match chars.peek() {
                Some(c) if c.is_whitespace() => {
                    chars.next();
                }
                _ => {}
            };
            continue;
        }
        if !is_name(c) {
            return false;
        }
    }
    true
}
