//! This crate provides shared utilities that is not related to any core compilation flow or data structures.
//! If you are a plugin author and look for internal helpers, [farmfe_toolkit] crate should be your first choice.

pub use pathdiff::diff_paths;

pub const PARSE_QUERY_TRUE: &str = "true";

/// parse `?a=b` to `HashMap { a: b }`, `?a` to `HashMap { a: "true" }`
pub fn parse_query(path: &str) -> Vec<(String, String)> {
  if !path.contains("?") {
    return vec![];
  }

  let query_str = path.split('?').last().unwrap();
  let pairs = query_str.split("&");

  let mut query = vec![];

  for pair in pairs {
    if pair.contains("=") {
      let kv = pair.split("=").collect::<Vec<_>>();

      if kv.len() != 2 {
        panic!("Invalid query: {}", pair);
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

/// stringify `HashMap { a: b }` to `?a=b`
pub fn stringify_query(query: &Vec<(String, String)>) -> String {
  if query.is_empty() {
    return String::new();
  }

  let mut qs = vec![];

  for (k, v) in query {
    if v == PARSE_QUERY_TRUE {
      qs.push(k.to_string());
    } else {
      qs.push(format!("{}={}", k, v));
    }
  }

  format!("?{}", qs.join("&").to_string())
}

// get platform independent relative path
pub fn relative(from: &str, to: &str) -> String {
  let rp =
    diff_paths(to, from).unwrap_or_else(|| panic!("{} or {} is not absolute path", from, to));

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
          unreachable!();
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
          result = format!("{}/{}", result, c);
        }
      }
    }
  }

  result
}

#[cfg(test)]
mod tests {
  use crate::{stringify_query, PARSE_QUERY_TRUE};

  use super::parse_query;

  #[test]
  fn parse_query_t() {
    let str = "./a.png?inline";
    let parsed_query = parse_query(str);
    assert_eq!(
      parsed_query,
      vec![("inline".to_string(), "true".to_string())]
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
}
