//! This crate provides shared utilities that is not related to any core compilation flow or data structures.
//! If you are a plugin author and look for internal helpers, [farmfe_toolkit] crate should be your first choice.

#![feature(trivial_bounds)]

use std::path::PathBuf;

pub use pathdiff::diff_paths;

pub mod bytes_str;
pub mod hash;
pub use nanoid;

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

  let query_str = path.split('?').next_back().unwrap();
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
  let file_path = url.strip_prefix("file://");
  let path = file_path.unwrap_or(url);

  if cfg!(windows) {
    let is_drive =
      |path: &str| matches!(path.as_bytes(), [drive, b':', ..] if drive.is_ascii_alphabetic());
    // A file URL adds one slash before a drive letter. UNC and verbatim
    // paths need both leading separators to retain their absolute prefix.
    let path = match path.strip_prefix('/') {
      Some(drive_path) if is_drive(drive_path) => drive_path,
      _ => path,
    };
    let normalized = path.replace('/', "\\");
    if file_path.is_some() && !path.starts_with(['/', '\\']) && !is_drive(path) {
      // file://server/share names a UNC host rather than a relative path.
      format!("\\\\{normalized}")
    } else {
      normalized
    }
  } else {
    path.to_string()
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

  // Different Windows volumes (or UNC shares) have no relative path. Keep
  // pathdiff's absolute target so module IDs can resolve it without losing
  // its volume identity. ModuleId::resolved_path already handles this form.
  if rp.is_absolute() {
    return rp.to_string_lossy().replace('\\', "/");
  }
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
  use crate::{PARSE_QUERY_TRUE, relative, stringify_query};

  use super::{file_url_to_path, parse_query};

  #[cfg(windows)]
  #[test]
  fn file_url_preserves_windows_absolute_prefixes() {
    for (input, expected) in [
      (r"C:\farm\index.js", r"C:\farm\index.js"),
      ("C:/farm/index.js", r"C:\farm\index.js"),
      ("file:///C:/farm/index.js", r"C:\farm\index.js"),
      ("file://C:/farm/index.js", r"C:\farm\index.js"),
      ("//?/C:/farm/index.js", r"\\?\C:\farm\index.js"),
      (r"\\?\C:\farm\index.js", r"\\?\C:\farm\index.js"),
      ("//server/share/index.js", r"\\server\share\index.js"),
      (r"\\server\share\index.js", r"\\server\share\index.js"),
      ("file://server/share/index.js", r"\\server\share\index.js"),
      (
        "//?/UNC/server/share/index.js",
        r"\\?\UNC\server\share\index.js",
      ),
    ] {
      let actual = file_url_to_path(input);
      assert_eq!(actual, expected, "{input}");
      assert!(std::path::Path::new(&actual).is_absolute(), "{input}");
    }
    assert_eq!(
      file_url_to_path("C:/farm/file://marker"),
      r"C:\farm\file:\\marker"
    );
  }

  #[cfg(windows)]
  #[test]
  fn relative_preserves_original_long_algolia_side_effects_path() {
    let from = "//?/C:/Users/lj_bright/Documents/Codex/2026-10-20/task/farm-docs-presentation/node_modules/.pnpm/@algolia+autocomplete-plugin-algolia-insights@1.9.3_@algolia+client-search@4.27.0_algoliasear_n4ixl6ospdcyccrc27rssdkaga/node_modules/@algolia/autocomplete-plugin-algolia-insights";
    let to = format!("{from}/dist/esm/createAlgoliaInsightsPlugin.js");
    assert!(from.len() > 260);
    assert_eq!(
      relative(from, &to),
      "dist/esm/createAlgoliaInsightsPlugin.js"
    );
    assert_eq!(
      relative(&from.replace('/', "\\"), &to.replace('/', "\\")),
      "dist/esm/createAlgoliaInsightsPlugin.js"
    );
  }

  #[cfg(windows)]
  #[test]
  fn relative_windows_prefixes_preserve_volume_round_trip() {
    for (from, to) in [
      ("C:/consumer/site", "D:/farm/runtime.js"),
      ("file:///C:/consumer/site", "file:///D:/farm/runtime.js"),
      ("//?/C:/consumer/site", "//?/D:/farm/runtime.js"),
      ("//server/first/site", "//server/second/runtime.js"),
    ] {
      let result = relative(from, to);
      let restored = file_url_to_path(&result);
      assert!(std::path::Path::new(&restored).is_absolute());
      assert_eq!(restored, file_url_to_path(to));
    }
  }

  #[cfg(not(windows))]
  #[test]
  fn file_url_preserves_posix_paths_and_embedded_scheme() {
    assert_eq!(
      file_url_to_path("file:///tmp/farm/index.js"),
      "/tmp/farm/index.js"
    );
    assert_eq!(file_url_to_path("/tmp/file://marker"), "/tmp/file://marker");
    assert_eq!(
      file_url_to_path("//server/share/index.js"),
      "//server/share/index.js"
    );
    assert_eq!(
      relative("file:///tmp/farm", "file:///tmp/other/index.js"),
      "../other/index.js"
    );
  }

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

  #[cfg(windows)]
  #[test]
  fn test_relative_cross_volume_preserves_absolute_target() {
    assert_eq!(
      relative(r"C:\consumer\site", r"D:\farm\runtime.js.farm-runtime"),
      "D:/farm/runtime.js.farm-runtime"
    );
    assert_eq!(
      relative("file:///C:/consumer/site", "file:///D:/farm/runtime.js?raw"),
      "D:/farm/runtime.js?raw"
    );
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
