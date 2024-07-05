use std::collections::HashMap;

use farmfe_core::regex::Regex;

const REGEX_PREFIX: &str = "$__farm_regex:";

/// Determine whether the path conforms to the configuration of alias.
pub fn is_start_with_alias(alias_map: &HashMap<String, String>, path: &str) -> bool {
  let mut aliases: Vec<&str> = alias_map.keys().map(|k| k.as_str()).collect();
  aliases.sort_by(|a, b| b.len().cmp(&a.len()));

  aliases.iter().any(|&alias| {
    if let Some(stripped_alias) = alias.strip_prefix(REGEX_PREFIX) {
      let regex = Regex::new(stripped_alias).unwrap();
      return regex.is_match(path);
    } else if alias.ends_with("$") && path == alias.trim_end_matches('$') {
      return true;
    } else {
      return path.starts_with(alias);
    }
  })
}

#[cfg(test)]

mod test {
  use std::path::PathBuf;

  use super::*;

  #[test]
  fn test() {
    let cwd = PathBuf::from("/root/src");

    let alias = HashMap::from([
      ("/@".to_string(), cwd.to_string_lossy().to_string()),
      ("@".to_string(), cwd.to_string_lossy().to_string()),
      ("react$".to_string(), cwd.to_string_lossy().to_string()),
      (
        "$__farm_regex:^/(utils)$".to_string(),
        cwd.join("$1").to_string_lossy().to_string(),
      ),
    ]);

    assert_eq!(is_start_with_alias(&alias, "/@/img/logo.png"), true);
    assert_eq!(is_start_with_alias(&alias, "@/img/logo.png"), true);

    assert_eq!(is_start_with_alias(&alias, "./img/logo.png"), false);
    assert_eq!(is_start_with_alias(&alias, "../img/logo.png"), false);
    assert_eq!(is_start_with_alias(&alias, "/img/logo.png"), false);

    assert_eq!(is_start_with_alias(&alias, "react/useEffect.ts"), false);
    assert_eq!(is_start_with_alias(&alias, "react"), true);

    assert_eq!(is_start_with_alias(&alias, "/utils"), true);
    assert_eq!(is_start_with_alias(&alias, "utils"), false);
    assert_eq!(is_start_with_alias(&alias, "/utils/index.ts"), false);
  }
}
