//! Compiled include/exclude/custom-element filters.
//!
//! Mirrors `vite`'s `createFilter` semantics on the patterns surfaced by
//! `options.rs`: a path matches when at least one `include` pattern matches
//! and no `exclude` pattern matches.

use regex::Regex;

use crate::consts::{DEFAULT_CUSTOM_ELEMENT_PATTERN, DEFAULT_INCLUDE_PATTERN};
use crate::options::{CustomElementMatcher, PatternList, VuePluginOptions};

#[derive(Debug)]
pub struct Filter {
  include: Vec<Regex>,
  exclude: Vec<Regex>,
}

impl Filter {
  pub fn new(include: Option<PatternList>, exclude: Option<PatternList>) -> Self {
    let include = compile_patterns(
      include
        .map(PatternList::into_sources)
        .unwrap_or_else(|| vec![DEFAULT_INCLUDE_PATTERN.to_string()]),
    );
    let exclude = compile_patterns(exclude.map(PatternList::into_sources).unwrap_or_default());
    Self { include, exclude }
  }

  pub fn matches(&self, path: &str) -> bool {
    let normalized = normalize_path(path);
    let paths = if normalized == path {
      vec![path]
    } else {
      vec![path, normalized.as_str()]
    };

    if self
      .exclude
      .iter()
      .any(|re| paths.iter().any(|path| re.is_match(path)))
    {
      return false;
    }

    self
      .include
      .iter()
      .any(|re| paths.iter().any(|path| re.is_match(path)))
  }
}

/// Custom-element filter, combining a default `\.ce\.vue$` suffix matcher,
/// the optional `customElement` boolean, and the optional pattern list from
/// `features.customElement`.
#[derive(Debug)]
pub enum CustomElementFilter {
  Always,
  Never,
  Patterns(Vec<Regex>),
}

impl CustomElementFilter {
  pub fn new(
    legacy: Option<CustomElementMatcher>,
    from_features: Option<CustomElementMatcher>,
  ) -> Self {
    let matcher = from_features.or(legacy);
    match matcher {
      Some(CustomElementMatcher::Boolean(true)) => CustomElementFilter::Always,
      Some(CustomElementMatcher::Boolean(false)) => CustomElementFilter::Never,
      Some(CustomElementMatcher::Patterns(list)) => {
        CustomElementFilter::Patterns(compile_patterns(list.into_sources()))
      }
      None => CustomElementFilter::Patterns(compile_patterns(vec![
        DEFAULT_CUSTOM_ELEMENT_PATTERN.to_string()
      ])),
    }
  }

  pub fn matches(&self, path: &str) -> bool {
    match self {
      CustomElementFilter::Always => true,
      CustomElementFilter::Never => false,
      CustomElementFilter::Patterns(patterns) => {
        let normalized = normalize_path(path);
        patterns
          .iter()
          .any(|re| re.is_match(path) || re.is_match(&normalized))
      }
    }
  }
}

fn normalize_path(path: &str) -> String {
  path.replace('\\', "/")
}

fn compile_patterns(sources: Vec<String>) -> Vec<Regex> {
  sources
    .into_iter()
    .filter_map(|src| Regex::new(&src).ok())
    .collect()
}

impl From<&VuePluginOptions> for Filter {
  fn from(opts: &VuePluginOptions) -> Self {
    Filter::new(opts.include.clone(), opts.exclude.clone())
  }
}
