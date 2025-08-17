use crate::{template::Displayable, utils::colors::*};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Lang {
  Javascript,
  Typescript,
}

impl Default for Lang {
  fn default() -> Self {
    Lang::Javascript
  }
}

impl Display for Lang {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Lang::Javascript => write!(f, "JavaScript"),
      Lang::Typescript => write!(f, "TypeScript"),
    }
  }
}

impl Displayable for Lang {
  fn display_text(&self) -> &str {
    match self {
      Lang::Javascript => "JavaScript",
      Lang::Typescript => "TypeScript",
    }
  }
}

impl<'a> Lang {
  pub const ALL: &'a [Lang] = &[Lang::Javascript, Lang::Typescript];
}

impl FromStr for Lang {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "JavaScript" => Ok(Lang::Javascript),
      "TypeScript" => Ok(Lang::Typescript),
      _ => Err(format!(
        "{YELLOW}{s}{RESET} is not a valid language. Valid languages are [{}]",
        Lang::ALL
          .iter()
          .map(|e| format!("{GREEN}{e}{RESET}"))
          .collect::<Vec<_>>()
          .join(", ")
      )),
    }
  }
}
