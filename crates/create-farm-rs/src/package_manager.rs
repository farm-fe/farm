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
}

impl PackageManager {
  /// Returns templates without flavors
  pub const fn templates_no_flavors(&self) -> &[Template] {
    match self {
      PackageManager::Pnpm | PackageManager::Yarn | PackageManager::Npm | PackageManager::Bun => &[
        Template::Vanilla,
        Template::React,
        Template::ReactTs,
        Template::Vue3,
        Template::Vue2,
        Template::Svelte,
        Template::Solid,
        Template::Lit,
        Template::Preact,
        Template::Nestjs,
        Template::Tauri2(None),
        Template::Tauri(None),
        Template::Electron(None),
      ],
    }
  }

  pub const fn install_cmd(&self) -> Option<&str> {
    match self {
      PackageManager::Pnpm => Some("pnpm install"),
      PackageManager::Yarn => Some("yarn"),
      PackageManager::Npm => Some("npm install"),
      PackageManager::Bun => Some("bun install"),
    }
  }

  pub const fn default_cmd(&self) -> &'static str {
    match self {
      PackageManager::Pnpm => "pnpm dev",
      PackageManager::Yarn => "yarn dev",
      PackageManager::Npm => "npm run dev",
      PackageManager::Bun => "bun run dev",
    }
  }
}
