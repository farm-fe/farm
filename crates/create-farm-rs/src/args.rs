use crate::{package_manager::PackageManager, template::Template, utils::colors::*};
use std::ffi::OsString;
use pico_args::Arguments;

#[derive(Debug)]
pub struct Args {
  pub project_name: Option<String>,
  pub manager: Option<PackageManager>,
  pub template: Option<Template>,
}


impl Default for Args {
  fn default() -> Self {
    Self {
      project_name: Some("farm-project".to_string()),
      manager: Some(PackageManager::Npm),
      template: Some(Template::Vanilla),
    }
  }
}


pub fn parse(argv: Vec<OsString>, bin_name: Option<String>) -> anyhow::Result<Args> {
  let mut pargs = Arguments::from_vec(argv);

  if pargs.contains(["-h", "--help"]) {
      let help = format!(
          r#""#
      );

      println!("{help}");
      std::process::exit(0);
  }
  if pargs.contains(["-v", "--version"]) {
      println!("{}", env!("CARGO_PKG_VERSION"));
      std::process::exit(0);
  }


  let args = Args {
      manager: pargs.opt_value_from_str(["-m", "--manager"])?,
      template: pargs.opt_value_from_str(["-t", "--template"])?,
      project_name: pargs.opt_free_from_str()?,
  };

  Ok(args)
}