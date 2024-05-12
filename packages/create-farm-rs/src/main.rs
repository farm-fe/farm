use std::{env::args_os, ffi::OsStr, path::Path};

fn main() {
  let mut args = args_os().peekable();
  let mut is_cargo = false;
  let bin_name = match args
    .next()
    .as_deref()
    .map(Path::new)
    .and_then(Path::file_stem)
    .and_then(OsStr::to_str)
  {
    Some("cargo-create-farm") => {
      is_cargo = true;
      if args.peek().and_then(|s| s.to_str()) == Some("create-farm") {
        // remove the extra cargo subcommand
        args.next();
        Some("cargo create-farm".into())
      } else {
        Some("cargo-create-farm".into())
      }
    }
    Some(stem) => Some(stem.to_string()),
    None => None,
  };
  create_farm::run(
    args,
    bin_name,
    if is_cargo { Some("cargo".into()) } else { None },
  );
}
