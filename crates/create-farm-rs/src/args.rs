use crate::{lang::Lang, package_manager::PackageManager, template::Template};
use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[command(
  name = "create-farm",
  about,
  long_about = None,
  version,
)]
pub struct Args {
  #[arg(help = "Project name")]
  pub project_name: Option<String>,
  #[arg(short, long, help = "Package manager to use")]
  pub manager: Option<PackageManager>,
  #[arg(short, long, help = "Project template to use")]
  pub template: Option<Template>,
  pub language: Option<Lang>,
  #[arg(short, long, help = "Force overwrite of existing files", action = ArgAction::SetTrue)]
  pub force: bool,
}

impl Default for Args {
  fn default() -> Self {
    Self {
      project_name: Some("farm-project".to_string()),
      manager: Some(PackageManager::Npm),
      template: Some(Template::Vanilla),
      language: Some(Lang::Typescript),
      force: false,
    }
  }
}
