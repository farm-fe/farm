use farmfe_core::{
  config::{AliasItem, StringOrRegex},
  regex::Regex,
};

const REGEX_PREFIX: &str = "$__farm_regex:";

/// Determine whether the path conforms to the configuration of alias.
// pub fn is_start_with_alias(alias_map: &Vec<AliasItem>, path: &str) -> bool {
// let mut aliases: Vec<&str> = alias_map.keys().map(|k| k.as_str()).collect();
// aliases.sort_by(|a, b| b.len().cmp(&a.len()));

// aliases.iter().any(|&alias| {
//   if let Some(stripped_alias) = alias.strip_prefix(REGEX_PREFIX) {
//     let regex = Regex::new(stripped_alias).unwrap();
//     return regex.is_match(path);
//   } else if alias.ends_with("$") && path == alias.trim_end_matches('$') {
//     return true;
//   } else {
//     return path.starts_with(alias);
//   }
// })
// }

pub fn is_start_with_alias(alias_vec: &Vec<AliasItem>, path: &str) -> bool {
  alias_vec.iter().any(|alias_item| {
    match alias_item {
      AliasItem {
        find,
        replacement: _,
      } => match find {
        StringOrRegex::String(alias) => {
          if let Some(stripped_alias) = alias.strip_prefix(REGEX_PREFIX) {
            if let Ok(regex) = Regex::new(stripped_alias) {
              regex.is_match(path)
            } else {
              false
            }
          } else if alias.ends_with("$") && path == alias.trim_end_matches('$') {
            true
          } else {
            path.starts_with(alias)
          }
        }
        StringOrRegex::Regex(regex) => regex.is_match(path),
      }, // AliasItem::Simple(_) => false,
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

    let alias: Vec<AliasItem> = vec![
      AliasItem {
        find: StringOrRegex::String("/@".to_string()),
        replacement: cwd.to_string_lossy().to_string(),
      },
      AliasItem {
        find: StringOrRegex::String("@".to_string()),
        replacement: cwd.to_string_lossy().to_string(),
      },
      AliasItem {
        find: StringOrRegex::Regex(Regex::new("react$").unwrap()),
        replacement: cwd.to_string_lossy().to_string(),
      },
      AliasItem {
        find: StringOrRegex::Regex(Regex::new("^/(utils)$").unwrap()),
        replacement: cwd.join("$1").to_string_lossy().to_string(),
      },
    ];

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
