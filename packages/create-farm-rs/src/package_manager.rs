use std::{fmt::Display, str::FromStr};

use crate::{template::Template, utils::colors::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PackageManager {
  Pnpm,
  Yarn,
  Npm,
  Bun,
}

impl Default for PackageManager {
  fn default() -> Self {
    PackageManager::Pnpm
  }
}

impl Display for PackageManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PackageManager::Pnpm => write!(f, "pnpm"),
      PackageManager::Yarn => write!(f, "yarn"),
      PackageManager::Npm => write!(f, "npm"),
      PackageManager::Bun => write!(f, "bun"),
    }
  }
}

impl FromStr for PackageManager {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "pnpm" => Ok(PackageManager::Pnpm),
      "yarn" => Ok(PackageManager::Yarn),
      "npm" => Ok(PackageManager::Npm),
      "bun" => Ok(PackageManager::Bun),
      _ => Err(format!(
        "{YELLOW}{s}{RESET} is not a valid package manager. Valid package mangers are [{}]",
        PackageManager::ALL
          .iter()
          .map(|e| format!("{GREEN}{e}{RESET}"))
          .collect::<Vec<_>>()
          .join(", ")
      )),
    }
  }
}


impl<'a> PackageManager {
  pub const ALL: &'a [PackageManager] = &[
      PackageManager::Pnpm,
      PackageManager::Yarn,
      PackageManager::Npm,
      PackageManager::Bun,
  ];

  /// Node.js managers
  pub const NODE: &'a [PackageManager] = &[
      PackageManager::Pnpm,
      PackageManager::Yarn,
      PackageManager::Npm,
      PackageManager::Bun,
  ];
}