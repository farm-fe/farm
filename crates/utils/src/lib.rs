//! This crate provides shared utilities that is not related to any core compilation flow or data structures.
//! If you are a plugin author and look for internal helpers, [farmfe_toolkit] crate should be your first choice.

use std::path::PathBuf;

pub use pathdiff::diff_paths;

pub mod hash;

pub const PARSE_QUERY_TRUE: &str = "";

pub const FARM_IGNORE_ACTION_COMMENT: &str = "$farm-ignore";
pub const FARM_IGNORE_ACTION_COMMENTS: [&str; 2] = [FARM_IGNORE_ACTION_COMMENT, "$vite-ignore"];

pub fn is_skip_action_by_comment(comment: &str) -> bool {
  if comment.is_empty() || !comment.contains('$') {
    return false;
  }

  for action in FARM_IGNORE_ACTION_COMMENTS {
    if comment.contains(action) {
      return true;
    }
  }

  false
}

/// parse `?a=b` to `HashMap { a: b }`, `?a` to `HashMap { a: "" }`
pub fn parse_query(path: &str) -> Vec<(String, String)> {
  if !path.contains('?') {
    return vec![];
  }

  let query_str = path.split('?').last().unwrap();
  let pairs = query_str.split('&');

  let mut query = vec![];

  for pair in pairs {
    if pair.contains('=') {
      let kv = pair.split('=').collect::<Vec<_>>();

      if kv.len() != 2 {
        panic!("Invalid query: {pair}");
      }

      query.push((kv[0], kv[1]));
    } else {
      query.push((pair, PARSE_QUERY_TRUE))
    }
  }

  query
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

/// stringify `Vec<(a, b)>` to `?a=b`.
/// for example: `vec![("a", "b")]` to `?a=b`
/// `vec![("a", "")]` to `?a`
pub fn stringify_query(query: &Vec<(String, String)>) -> String {
  if query.is_empty() {
    return String::new();
  }

  let mut qs = vec![];

  for (k, v) in query {
    if v == PARSE_QUERY_TRUE || v.is_empty() {
      qs.push(k.to_string());
    } else {
      qs.push(format!("{k}={v}"));
    }
  }

  format!("?{}", qs.join("&"))
}

pub fn file_url_to_path(url: &str) -> String {
  let url = url.replace("file://", "");

  if cfg!(windows) {
    if let Some(url) = url.strip_prefix('/') {
      url.replace('/', "\\")
    } else {
      url.replace("/", "\\")
    }
  } else {
    url
  }
}

/// get platform independent relative path
/// for example: from = "/desktop/farm/projects", to = "/desktop/farm/documents/report.txt"
/// the result will be "../documents/report.txt"
pub fn relative(from: &str, to: &str) -> String {
  let from = file_url_to_path(from);
  let to = file_url_to_path(to);

  if !PathBuf::from(&from).is_absolute() {
    panic!("from path must be absolute. from: {from} to: {to}");
  }

  let rp = diff_paths(&to, &from).unwrap_or_else(|| {
    if !PathBuf::from(&to).is_absolute() {
      return PathBuf::from(&to);
    }
    panic!("failed to get relative path from {from} to {to}");
  });

  // make sure the relative path is platform independent
  // this can ensure that the relative path and hash stable across platforms
  let mut result = String::new();

  for comp in rp.components() {
    match comp {
      std::path::Component::Prefix(_)
      | std::path::Component::RootDir
      | std::path::Component::CurDir => {
        if result.is_empty() {
          result += ".";
        } else {
          unreachable!("Invalid relative: {from} -> {to}");
        }
      }
      std::path::Component::ParentDir => {
        if result.is_empty() {
          result += "..";
        } else {
          result += "/.."
        }
      }
      std::path::Component::Normal(c) => {
        let c = c.to_string_lossy().to_string();

        if result.is_empty() {
          result += &c;
        } else {
          result = format!("{result}/{c}");
        }
      }
    }
  }

  result
}

pub fn transform_string_to_static_str(s: String) -> &'static str {
  Box::leak(s.into_boxed_str())
}

#[cfg(test)]
mod tests {
  use crate::{relative, stringify_query, PARSE_QUERY_TRUE};

  use super::parse_query;

  #[test]
  fn parse_query_t() {
    let str = "./a.png?inline&b=c";
    let parsed_query = parse_query(str);
    assert_eq!(
      parsed_query,
      vec![
        ("inline".to_string(), "".to_string()),
        ("b".to_string(), "c".to_string())
      ]
    );

    let str = "?a=b";
    let parsed_query = parse_query(str);
    assert_eq!(parsed_query, vec![("a".to_string(), "b".to_string())]);

    let str = "./a";
    let parsed_query = parse_query(str);
    assert_eq!(parsed_query, vec![]);
  }

  #[test]
  fn stringify_query_t() {
    let query = vec![("inline".to_string(), PARSE_QUERY_TRUE.to_string())];
    let str = stringify_query(&query);
    assert_eq!(str, "?inline");

    let query = vec![("a".to_string(), "b".to_string())];
    let str = stringify_query(&query);
    assert_eq!(str, "?a=b".to_string());
  }

  #[test]
  fn test_relative() {
    let from = if cfg!(windows) {
      "C:\\desktop\\farm\\projects"
    } else {
      "/desktop/farm/projects"
    };
    let to = if cfg!(windows) {
      "C:\\desktop\\farm\\documents\\report.txt"
    } else {
      "/desktop/farm/documents/report.txt"
    };

    assert_eq!(relative(from, to), "../documents/report.txt");
  }
}
