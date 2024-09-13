use regex::Regex;

pub fn is_valid_pkg_name(project_name: &str) -> bool {
  let regex =
    Regex::new(r"^(?:(?:@(?:[a-z0-9-*][a-z0-9-*_]*)?/[a-z0-9-_])|[a-z][a-z0-9-]?)[a-z0-9-_]*$")
      .unwrap();
  regex.is_match(project_name)
}

pub fn to_valid_pkg_name(project_name: &str) -> String {
  let ret = project_name
    .trim()
    .to_lowercase()
    .replace([':', ';', ' ', '~'], "-")
    .replace(['.', '\\', '/'], "");

  let ret = ret
    .chars()
    .skip_while(|ch| ch.is_ascii_digit() || *ch == '-')
    .collect::<String>();

  if ret.is_empty() || !is_valid_pkg_name(&ret) {
    "farm-project".to_string()
  } else {
    ret
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_is_valid_pkg_name() {
    assert!(is_valid_pkg_name("a"));
    assert!(is_valid_pkg_name("a1"));
    assert!(is_valid_pkg_name("a_12"));
    assert!(is_valid_pkg_name("a-b"));
    assert!(is_valid_pkg_name("a_b"));
    assert!(!is_valid_pkg_name("a.b"));
    assert!(!is_valid_pkg_name("a/b"));
    assert!(is_valid_pkg_name("@scope/a"));
    assert!(is_valid_pkg_name("@scope/a-b"));
    assert!(is_valid_pkg_name("@scope/a_b"));
    assert!(!is_valid_pkg_name("@scope/a.b"));
    assert!(!is_valid_pkg_name("@scope/a/b"));
    assert!(!is_valid_pkg_name("@scope/a/b/c"));
    assert!(is_valid_pkg_name("a-b-c"));
    assert!(is_valid_pkg_name("a-b-c-d"));
    assert!(is_valid_pkg_name("a-b-c-d-e"));
    assert!(!is_valid_pkg_name("A"));
    assert!(!is_valid_pkg_name("Abc"));
    assert!(!is_valid_pkg_name("aBc"));
    assert!(!is_valid_pkg_name("abC"));
    assert!(!is_valid_pkg_name("a b"));
    assert!(!is_valid_pkg_name(""));
    assert!(!is_valid_pkg_name("@a"));
    assert!(!is_valid_pkg_name("@@a/b"));
    assert!(!is_valid_pkg_name("a😊"));
    assert!(!is_valid_pkg_name("!test"));
    assert!(!is_valid_pkg_name("test!#$"));
    assert!(!is_valid_pkg_name("(test)"));
    assert!(!is_valid_pkg_name("-a"));
  }
}
