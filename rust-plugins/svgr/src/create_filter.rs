use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;

#[derive(Debug)]
pub struct Filter {
  include: GlobSet,
  exclude: GlobSet,
}

impl Filter {
  pub fn new<I, E>(include: I, exclude: E) -> Result<Self, Box<dyn std::error::Error>>
  where
    I: IntoIterator,
    I::Item: AsRef<str>,
    E: IntoIterator,
    E::Item: AsRef<str>,
  {
    let mut include_builder = GlobSetBuilder::new();
    let mut exclude_builder = GlobSetBuilder::new();

    for pattern in include {
      include_builder.add(Glob::new(pattern.as_ref())?);
    }

    for pattern in exclude {
      exclude_builder.add(Glob::new(pattern.as_ref())?);
    }

    Ok(Filter {
      include: include_builder.build()?,
      exclude: exclude_builder.build()?,
    })
  }

  pub fn should_process<P: AsRef<Path>>(&self, path: P) -> bool {
    let path_str = path.as_ref().to_string_lossy();

    if self.exclude.is_match(&*path_str) {
      return false;
    }

    self.include.is_match(&*path_str)
  }
}

pub fn create_filter<I, E>(
  include: Option<I>,
  exclude: Option<E>,
) -> Result<Filter, Box<dyn std::error::Error>>
where
  I: IntoIterator + Default,
  I::Item: AsRef<str>,
  E: IntoIterator + Default,
  E::Item: AsRef<str>,
{
  Filter::new(include.unwrap_or_default(), exclude.unwrap_or_default())
}
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_empty_filter() {
    let filter = create_filter::<Vec<&str>, Vec<&str>>(None, None).unwrap();

    assert!(!filter.should_process("test.svg"));
    assert!(!filter.should_process("path/to/test.svg"));
    assert!(!filter.should_process("test.png"));
    assert!(!filter.should_process("test.svg.png"));
  }

  #[test]
  fn test_custom_filter() {
    let filter = create_filter(
      Some(vec!["**/*.svg", "**/*.icon"]),
      Some(vec!["**/ignore/**"]),
    )
    .unwrap();

    assert!(filter.should_process("test.svg"));
    assert!(filter.should_process("test.icon"));
    assert!(filter.should_process("path/to/test.svg"));
    assert!(!filter.should_process("path/ignore/test.svg"));
    assert!(!filter.should_process("test.png"));
  }
}
