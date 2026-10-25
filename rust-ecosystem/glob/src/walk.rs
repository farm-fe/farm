use crate::{compile_matcher, scan, CompileOptions, MatchError, Matcher, ScanOptions};
use std::{
  fmt,
  path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum GlobError {
  Pattern(MatchError),
  Walk(walkdir::Error),
  OutsideRoot(PathBuf),
  Io(std::io::Error),
}
impl fmt::Display for GlobError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{self:?}")
  }
}
impl std::error::Error for GlobError {}

#[derive(Clone, Copy, Default)]
pub struct WalkOptions {
  pub exhaustive: bool,
  pub case_insensitive: bool,
}

/// Scoped import-glob walker. No fallback to another pattern dialect.
pub struct Glob {
  pattern: String,
  matcher: Matcher,
  options: WalkOptions,
}
impl Glob {
  pub fn new(pattern: &str) -> Result<Self, GlobError> {
    Self::with_options(pattern, WalkOptions::default())
  }
  pub fn with_options(pattern: &str, options: WalkOptions) -> Result<Self, GlobError> {
    // tinyglobby's Vite transform profile, not direct HMR's defaults.
    let compile = CompileOptions {
      windows: false,
      noextglob: true,
      posix: true,
      dot: options.exhaustive,
      nocase: options.case_insensitive,
      ..CompileOptions::default()
    };
    let matcher = compile_matcher(pattern, &compile).map_err(GlobError::Pattern)?;
    Ok(Self {
      pattern: pattern.to_owned(),
      matcher,
      options,
    })
  }
  pub fn is_match(&self, relative: &str) -> Result<bool, GlobError> {
    self.matcher.is_match(relative).map_err(GlobError::Pattern)
  }
  pub fn walk(
    &self,
    root: impl AsRef<Path>,
  ) -> std::vec::IntoIter<Result<walkdir::DirEntry, GlobError>> {
    let root = root.as_ref();
    let scanned = scan(&self.pattern, &ScanOptions::default());
    let base = if self.options.case_insensitive {
      String::new()
    } else {
      scanned.base
    };
    let start = root.join(crate::utils::remove_backslashes(&base));
    match start.try_exists() {
      Ok(false) => return Vec::new().into_iter(),
      Err(error) => return vec![Err(GlobError::Io(error))].into_iter(),
      Ok(true) => {}
    }
    let options = self.options;
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(start)
      .follow_links(true)
      .into_iter()
      .filter_entry(|entry| options.exhaustive || entry.file_name() != "node_modules")
    {
      match entry {
        Err(error) => out.push(Err(GlobError::Walk(error))),
        Ok(entry) => {
          if entry.file_type().is_dir() {
            continue;
          }
          let Ok(relative) = entry.path().strip_prefix(root) else {
            out.push(Err(GlobError::OutsideRoot(entry.path().to_owned())));
            continue;
          };
          let relative = relative.to_string_lossy().replace('\\', "/");
          if !options.exhaustive && relative.split('/').any(|part| part == "node_modules") {
            continue;
          }
          match self.is_match(&relative) {
            Ok(true) => out.push(Ok(entry)),
            Ok(false) => {}
            Err(error) => out.push(Err(error)),
          }
        }
      }
    }
    out.into_iter()
  }
}
