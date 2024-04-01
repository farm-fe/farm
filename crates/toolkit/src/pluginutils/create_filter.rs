use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::error::Error;

pub fn create_filter<'a>(
  include_patterns: Option<&'a [&str]>,
  exclude_patterns: Option<&'a [&str]>,
) -> Result<impl Fn(&str) -> bool + 'a, Box<dyn Error>> {
  let include_set = patterns_builder(include_patterns)?;
  let exclude_set = patterns_builder(exclude_patterns)?;

  Ok(move |path: &str| {
    let match_include = match include_patterns {
      Some(patterns) => patterns.is_empty() || include_set.is_match(path),
      None => true,
    };
    let match_exclude = match exclude_patterns {
      Some(patterns) => !patterns.is_empty() && exclude_set.is_match(path),
      None => false,
    };
    match_include && !match_exclude
  })
}

fn patterns_builder<'a>(patterns: Option<&'a [&str]>) -> Result<GlobSet, Box<dyn Error>> {
  let mut builder = GlobSetBuilder::new();
  if let Some(patterns) = patterns {
    for pattern in patterns {
      builder.add(GlobBuilder::new(pattern.as_ref()).build()?);
    }
  }
  Ok(builder.build()?)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::pluginutils::normalize_path::normalize_path;
  use std::path::PathBuf;

  #[test]
  fn includes_by_default() -> Result<(), Box<dyn Error>> {
    let filter = create_filter(None, None)?;
    assert!(filter("index.html"));
    Ok(())
  }

  #[test]
  fn excludes_items_not_included_if_include_patterns_provided() -> Result<(), Box<dyn Error>> {
    let filter = create_filter(Some(&["*.y"]), None)?;
    assert!(!filter("x"));
    assert!(filter("a.y"));
    Ok(())
  }

  #[test]
  fn patterns_with_wildcards() -> Result<(), Box<dyn Error>> {
    let filter = create_filter(Some(&["*.y", "a.?"]), None)?;
    assert!(filter("c.y"));
    assert!(!filter("c.z"));
    assert!(filter("a.x"));
    assert!(!filter("b.x"));
    Ok(())
  }

  #[test]
  fn excludes_items_when_exclude_pattern_provided() -> Result<(), Box<dyn Error>> {
    let filter = create_filter(None, Some(&["*.tmp"]))?;
    assert!(filter("a.out"));
    assert!(!filter("b.tmp"));
    Ok(())
  }

  #[test]
  fn properly_handles_inclusion_and_exclusion_patterns() -> Result<(), Box<dyn Error>> {
    let filter = create_filter(Some(&["*.js"]), Some(&["*.min.js"]))?;
    assert!(filter("app.js"));
    assert!(!filter("app.min.js"));
    assert!(!filter("app.ts"));
    Ok(())
  }

  #[test]
  fn handles_relative_paths_correctly() -> Result<(), Box<dyn Error>> {
    let filter = create_filter(Some(&["src/*.js"]), Some(&["src/*.test.js"]))?;
    assert!(filter(&PathBuf::from("src/main.js").to_string_lossy()));
    assert!(!filter(
      &PathBuf::from("src/main.test.js").to_string_lossy()
    ));
    assert!(!filter(&PathBuf::from("lib/main.js").to_string_lossy()));
    Ok(())
  }

  #[test]
  fn handles_relative_paths() -> Result<(), Box<dyn Error>> {
    let filter = create_filter(Some(&["./index.js"]), Some(&["'./foo/../a.js'"]))?;
    assert!(!filter(&normalize_path("index.js")));
    assert!(!filter(&normalize_path("a.js")));
    assert!(!filter(&normalize_path("foo/a.js")));
    Ok(())
  }
}
