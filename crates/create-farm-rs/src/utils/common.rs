use std::fs;

use crate::{package_manager::PackageManager, template::Template};

pub fn is_valid_pkg_name(project_name: &str) -> bool {
  let mut chars = project_name.chars().peekable();
  !project_name.is_empty()
    && !chars.peek().map(|c| c.is_ascii_digit()).unwrap_or_default()
    && !chars.any(|ch| !(ch.is_alphanumeric() || ch == '-' || ch == '_') || ch.is_uppercase())
}

pub fn to_valid_pkg_name(project_name: &str) -> String {
  let ret = project_name
    .trim()
    .to_lowercase()
    .replace([':', ';', ' ', '~'], "-")
    .replace(['.', '\\', '/'], "");

  let ret = ret
    .chars()
    .skip_while(|ch| ch.is_ascii_digit() || *ch == '-')
    .collect::<String>();

  if ret.is_empty() || !is_valid_pkg_name(&ret) {
    "farm-project".to_string()
  } else {
    ret
  }
}

pub fn get_run_cmd(pkg_manager: &PackageManager, template: &Template) -> &'static str {
  match template {
    Template::Tauri(_) => match pkg_manager {
      PackageManager::Pnpm => "pnpm tauri dev",
      PackageManager::Yarn => "yarn tauri dev",
      PackageManager::Npm => "npm run tauri dev",
      PackageManager::Bun => "bun tauri dev",
      PackageManager::Deno => "deno dev",
    },
    _ => pkg_manager.default_cmd(),
  }
}

pub fn clean_dir(dir: &std::path::PathBuf) -> anyhow::Result<()> {
  for entry in fs::read_dir(dir)?.flatten() {
    let path = entry.path();
    if entry.file_type()?.is_dir() {
      if entry.file_name() != ".git" {
        clean_dir(&path)?;
        std::fs::remove_dir(path)?;
      }
    } else {
      fs::remove_file(path)?;
    }
  }
  Ok(())
}

pub fn to_pascal_case(s: &str) -> String {
  let mut result = String::new();
  let mut capitalize_next = false;
  for (s, c) in s.chars().enumerate() {
    if s == 0 {
      result.push(c.to_ascii_uppercase());
    } else if capitalize_next {
      result.push(c.to_ascii_uppercase());
      capitalize_next = false;
    } else if ['_', '-'].contains(&c) {
      capitalize_next = true;
    } else {
      result.push(c);
    }
  }
  result
}
