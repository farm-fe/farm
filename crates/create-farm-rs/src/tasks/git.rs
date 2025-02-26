use crate::utils::prompts;

pub struct GitTask;

impl super::Task for GitTask {
  fn run(&mut self, ctx: &mut crate::utils::context::Context) -> anyhow::Result<()> {
    let target_dir: String = serde_json::from_value(ctx.get("target_dir").unwrap().clone())?;
    if prompts::confirm("Initialize Git repository", true)? {
      gix::init(target_dir)?;
    }
    Ok(())
  }

  fn next(&self) -> Option<Box<dyn super::Task>> {
    Some(Box::new(super::biome::BiomeTask))
  }
}
