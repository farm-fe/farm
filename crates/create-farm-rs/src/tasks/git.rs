use std::path::PathBuf;

use crate::utils::prompts;

pub struct GitTask;

const GIT_IGNORE: &str = include_str!("./gitignore.txt");

impl super::Task for GitTask {
  fn run(&mut self, ctx: &mut crate::utils::context::Context) -> anyhow::Result<()> {
    let target_dir: PathBuf = serde_json::from_value(ctx.get("target_dir").unwrap().clone())?;
    if prompts::confirm("Initialize Git repository", true)? {
      gix::init(&target_dir)?;
      std::fs::write(target_dir.join(".gitignore"), GIT_IGNORE)?;
    }
    Ok(())
  }

  fn next(&self) -> Option<Vec<Box<dyn super::Task>>> {
    Some(vec![Box::new(super::tips::TipsTask)])
  }
}
