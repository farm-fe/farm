use std::process;

use crate::{
  utils::{colors::*, common, prompts},
  DEFAULT_PROJECT_NAME,
};

pub struct MainTask;

impl super::Task for MainTask {
  fn run(&mut self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
    handle_brand_text("\n ⚡ Welcome To Farm ! \n");

    let project_name = match &ctx.options.project_name {
      Some(name) => common::to_valid_pkg_name(&name),
      &None => {
        let mut project_name = DEFAULT_PROJECT_NAME.to_string();
        loop {
          let input = prompts::input("Project name", Some(&project_name), false)?
            .trim()
            .to_string();
          if !common::is_valid_pkg_name(&input) {
            eprintln!(
            "{BOLD}{RED}✘{RESET} Invalid project name: {BOLD}{YELLOW}{input}{RESET}, {}",
            "package name should only include lowercase alphanumeric character and hyphens \"-\" and doesn't start with numbers"
          );
            project_name = common::to_valid_pkg_name(&input);
            continue;
          };
          break input;
        }
      }
    };
    let lib_name = format!("{}_lib", project_name.replace('-', "_"));
    let pascal_case_name = common::to_pascal_case(&project_name);

    let cwd = std::env::current_dir()?;
    let target_dir = cwd.join(&project_name);

    ctx.insert("project_name", &project_name);
    ctx.insert("lib_name", &lib_name);
    ctx.insert("pascal_case_name", &pascal_case_name);
    ctx.insert("cwd", &cwd);
    ctx.insert("target_dir", &target_dir);

    if target_dir.exists() && target_dir.read_dir()?.next().is_some() {
      let overwrite = ctx.options.force
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
        process::exit(1);
      }
    };

    Ok(())
  }

  fn next(&self) -> Option<Box<dyn super::Task>> {
    Some(Box::new(super::template::TemplateSelectTask::new()))
  }
}
