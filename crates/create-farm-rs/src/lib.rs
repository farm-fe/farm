mod args;
mod package_manager;
mod tasks;
mod template;
pub mod utils;

use clap::Parser;
use std::{ffi::OsString, process::exit};
use tasks::Runtime;
use utils::context;

use crate::utils::colors::*;

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

const DEFAULT_PROJECT_NAME: &str = "farm-project";

fn run_cli<I, A>(
  args: I,
  bin_name: Option<String>,
  _detected_manager: Option<String>,
) -> anyhow::Result<()>
where
  I: IntoIterator<Item = A>,
  A: Into<OsString> + Clone,
{
  // Clap will auto parse the `bin_name` as the first argument, so we need to add it to the args
  let args = args::Args::parse_from(
    std::iter::once(OsString::from(bin_name.unwrap_or_default()))
      .chain(args.into_iter().map(Into::into)),
  );

  let mut context = context::Context::new_with_options(args);

  Runtime::run(&mut context)?;

  Ok(())
}
