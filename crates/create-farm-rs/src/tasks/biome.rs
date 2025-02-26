use std::{fs, path::PathBuf};

use crate::utils::prompts;

pub struct BiomeTask;

const BIOME_JSON: &str = include_str!("./biome.json");

impl super::Task for BiomeTask {
  fn run(&mut self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
    if prompts::confirm("Setup biome to format and lint", true)? {
      let target_dir: PathBuf = serde_json::from_value(ctx.get("target_dir").unwrap().clone())?;
      fs::write(target_dir.join("biome.json"), BIOME_JSON)?;
    }
    Ok(())
  }
}
