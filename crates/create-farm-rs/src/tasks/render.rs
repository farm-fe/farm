use std::{fs, path::PathBuf};

use crate::utils::common;

pub struct RenderTask;

impl RenderTask {
  pub fn new() -> Self {
    Self
  }
}

impl super::Task for RenderTask {
  fn run(&mut self, ctx: &mut crate::utils::context::Context) -> anyhow::Result<()> {
    let target_dir: PathBuf = serde_json::from_value(ctx.get("target_dir").unwrap().clone())?;
    let template = *ctx.template();

    if target_dir.exists() {
      common::clean_dir(&target_dir)?;
    } else {
      fs::create_dir_all(&target_dir)?;
    }

    template.render(&target_dir, ctx)?;
    Ok(())
  }

  fn next(&self) -> Option<Vec<Box<dyn super::Task>>> {
    Some(vec![Box::new(super::git::GitTask)])
  }
}
