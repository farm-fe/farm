use anyhow::Context;
use clap::Parser;
use std::{ffi::OsString, fs, process::exit};
use utils::prompts;

use crate::{
  package_manager::PackageManager,
  template::{ElectronSubTemplate, TauriSubTemplate, Template},
  utils::colors::*,
};

mod args;
mod package_manager;
mod template;
pub mod utils;

pub fn run<I, A>(args: I, bin_name: Option<String>, detected_manager: Option<String>)
where
  I: IntoIterator<Item = A>,
  A: Into<OsString> + Clone,
{
  if let Err(e) = run_cli(args, bin_name, detected_manager) {
    println!();
    eprintln!("\n {BOLD}{RED}error{RESET}: {e:#}\n");
    exit(1);
  }
}

fn run_cli<I, A>(
  args: I,
  bin_name: Option<String>,
  detected_manager: Option<String>,
) -> anyhow::Result<()>
where
  I: IntoIterator<Item = A>,
  A: Into<OsString> + Clone,
{
  let detected_manager = detected_manager.and_then(|p| p.parse::<PackageManager>().ok());
  // Clap will auto parse the `bin_name` as the first argument, so we need to add it to the args
  let args = args::Args::parse_from(
    std::iter::once(OsString::from(bin_name.unwrap_or_default()))
      .chain(args.into_iter().map(Into::into)),
  );

  handle_brand_text("\n ⚡ Welcome To Farm ! \n");
  let defaults = args::Args::default();
  let args::Args {
    manager,
    project_name,
    template,
    force,
  } = args;

  let cwd = std::env::current_dir()?;
  let mut default_project_name = "farm-project";
  let project_name = match project_name {
    Some(name) => to_valid_pkg_name(&name),
    None => loop {
      let input = prompts::input("Project name", Some(default_project_name), false)?
        .trim()
        .to_string();
      if !is_valid_pkg_name(&input) {
        eprintln!(
          "{BOLD}{RED}✘{RESET} Invalid project name: {BOLD}{YELLOW}{input}{RESET}, package name should only include lowercase alphanumeric character and hyphens \"-\" and doesn't start with numbers",
        );
        default_project_name = to_valid_pkg_name(&input).leak();
        continue;
      };
      break input;
    },
  };
  let target_dir = cwd.join(&project_name);

  if target_dir.exists() && target_dir.read_dir()?.next().is_some() {
    let overwrite = force
      || prompts::confirm(
        &format!(
          "{} directory is not empty, do you want to overwrite?",
          if target_dir == cwd {
            "Current".to_string()
          } else {
            target_dir
              .file_name()
              .unwrap()
              .to_string_lossy()
              .to_string()
          }
        ),
        false,
      )?;
    if !overwrite {
      eprintln!("{BOLD}{RED}✘{RESET} Directory is not empty, Operation Cancelled");
      exit(1);
    }
  };

  let pkg_manager = manager.unwrap_or(match detected_manager {
    Some(manager) => manager,
    None => defaults.manager.context("default manager not set")?,
  });

  let templates_no_flavors = pkg_manager.templates_no_flavors();

  let template = match template {
    Some(template) => template,
    None => {
      let selected_template =
        prompts::select("Select a framework:", &templates_no_flavors, Some(0))?.unwrap();

      match selected_template {
        Template::Tauri(None) => {
          let sub_templates = vec![
            TauriSubTemplate::React,
            TauriSubTemplate::Vue,
            TauriSubTemplate::Svelte,
            TauriSubTemplate::Vanilla,
            TauriSubTemplate::Solid,
            TauriSubTemplate::Preact,
          ];

          let sub_template =
            prompts::select("Select a Tauri template:", &sub_templates, Some(0))?.unwrap();

          Template::Tauri(Some(*sub_template))
        }
        Template::Tauri2(None) => {
          let sub_templates = vec![
            TauriSubTemplate::React,
            TauriSubTemplate::Vue,
            TauriSubTemplate::Svelte,
            TauriSubTemplate::Vanilla,
            TauriSubTemplate::Solid,
            TauriSubTemplate::Preact,
          ];

          let sub_template =
            prompts::select("Select a Tauri2 template:", &sub_templates, Some(0))?.unwrap();

          Template::Tauri2(Some(*sub_template))
        }
        Template::Electron(None) => {
          let sub_templates = vec![
            ElectronSubTemplate::React,
            ElectronSubTemplate::Vue,
            ElectronSubTemplate::Svelte,
            ElectronSubTemplate::Vanilla,
            ElectronSubTemplate::Solid,
            ElectronSubTemplate::Preact,
          ];

          let sub_template =
            prompts::select("Select an Electron template:", &sub_templates, Some(0))?.unwrap();

          Template::Electron(Some(*sub_template))
        }
        _ => *selected_template,
      }
    }
  };

  if target_dir.exists() {
    #[inline(always)]
    fn clean_dir(dir: &std::path::PathBuf) -> anyhow::Result<()> {
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
    clean_dir(&target_dir)?;
  } else {
    fs::create_dir_all(&target_dir)?;
  }

  // Render the template
  template.render(&target_dir, pkg_manager, &project_name, &project_name)?;

  handle_brand_text("\n >  Initial Farm Project created successfully ✨ ✨ \n");

  if target_dir != cwd {
    handle_brand_text(&format!(
      "    cd {} \n",
      if project_name.contains(' ') {
        format!("\"{project_name}\"")
      } else {
        project_name.to_string()
      }
    ));
  }
  if let Some(cmd) = pkg_manager.install_cmd() {
    handle_brand_text(&format!("    {cmd} \n"));
  }
  handle_brand_text(&format!("    {} \n", get_run_cmd(&pkg_manager, &template)));

  Ok(())
}

fn is_valid_pkg_name(project_name: &str) -> bool {
  let mut chars = project_name.chars().peekable();
  !project_name.is_empty()
    && !chars.peek().map(|c| c.is_ascii_digit()).unwrap_or_default()
    && !chars.any(|ch| !(ch.is_alphanumeric() || ch == '-' || ch == '_') || ch.is_uppercase())
}

fn to_valid_pkg_name(project_name: &str) -> String {
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

fn get_run_cmd(pkg_manager: &PackageManager, template: &Template) -> &'static str {
  match template {
    Template::Tauri(_) => match pkg_manager {
      PackageManager::Pnpm => "pnpm tauri dev",
      PackageManager::Yarn => "yarn tauri dev",
      PackageManager::Npm => "npm run tauri dev",
      PackageManager::Bun => "bun tauri dev",
    },
    _ => pkg_manager.default_cmd(),
  }
}
