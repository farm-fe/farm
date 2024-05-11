use std::{collections::HashMap, fmt::Display, fs, io::Write, path, str::FromStr};

use crate::{
  package_manager::PackageManager,
  utils::{colors::*, lte},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Template {
  Vanilla,
  Vue,
  Svelte,
  React,
  Solid,
  Preact,
}

impl Default for Template {
  fn default() -> Self {
    Template::Vanilla
  }
}

impl Display for Template {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Template::Vanilla => write!(f, "vanilla"),
      Template::Vue => write!(f, "vue"),
      Template::Svelte => write!(f, "svelte"),
      Template::React => write!(f, "react"),
      Template::Solid => write!(f, "solid"),
      Template::Preact => write!(f, "preact"),
    }
  }
}

impl FromStr for Template {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "vanilla" => Ok(Template::Vanilla),
      "vue" => Ok(Template::Vue),
      "svelte" => Ok(Template::Svelte),
      "react" => Ok(Template::React),
      "solid" => Ok(Template::Solid),
      "preact" => Ok(Template::Preact),
      _ => Err(format!(
        "{YELLOW}{s}{RESET} is not a valid template. Valid templates are [{}]",
        Template::ALL
          .iter()
          .map(|e| format!("{GREEN}{e}{RESET}"))
          .collect::<Vec<_>>()
          .join(", ")
      )),
    }
  }
}

impl Template {
  pub const fn select_text<'a>(&self) -> &'a str {
    match self {
      Template::Vanilla => "Vanilla",
      Template::Vue => "Vue - (https://vuejs.org/)",
      Template::Svelte => "Svelte - (https://svelte.dev/)",
      Template::React => "React - (https://react.dev/)",
      Template::Solid => "Solid - (https://solidjs.com/)",
      Template::Preact => "Preact - (https://preactjs.com/)",
      _ => unreachable!(),
    }
  }
}

impl<'a> Template {
  pub const ALL: &'a [Template] = &[
      Template::Vanilla,
      Template::Vue,
      Template::Svelte,
      Template::React,
      Template::Solid,
      Template::Preact,
  ];
}