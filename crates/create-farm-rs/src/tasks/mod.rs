pub mod biome;
pub mod git;
pub mod main;
pub mod render;
pub mod template;
pub mod tips;

use anyhow::Result;

use crate::context::Context;

pub trait Task {
  fn run(&mut self, ctx: &mut Context) -> Result<()>;
  fn condition(&self, _ctx: &Context) -> bool {
    true
  }
  fn next(&self) -> Option<Box<dyn Task>> {
    None
  }
}

pub struct Runtime;

impl Runtime {
  pub fn run(ctx: &mut Context) -> Result<()> {
    let mut task = Box::new(main::MainTask) as Box<dyn Task>;
    task.run(ctx)?;
    while let Some(next_task) = task.next() {
      task = next_task;
      if task.condition(ctx) {
        task.run(ctx)?;
      }
    }
    Ok(())
  }
}
