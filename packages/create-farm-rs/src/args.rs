use crate::{package_manager::PackageManager, template::Template, utils::colors::*};

#[derive(Debug)]
pub struct Args {
  pub project_name: Option<String>,
  pub manager: Option<PackageManager>,
  pub template: Option<Template>,
}


impl Default for Args {
  fn default() -> Self {
    Self {
      project_name: Some("tauri-app".to_string()),
      manager: Some(PackageManager::Npm),
      template: Some(Template::Vanilla),
    }
  }
}
