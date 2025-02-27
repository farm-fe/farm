pub mod biome;
pub mod extra;
pub mod git;
pub mod main;
pub mod render;
pub mod template;
pub mod tips;

use anyhow::Result;

use crate::{context::Context, template::Displayable};

pub trait Task {
  fn description(&self) -> &str {
    std::any::type_name::<Self>()
  }
  fn run(&mut self, ctx: &mut Context) -> Result<()>;
  fn condition(&self, _ctx: &Context) -> bool {
    true
  }
  fn next(&self) -> Option<Vec<Box<dyn Task>>> {
    None
  }
}

impl Displayable for Box<dyn Task> {
  fn display_text(&self) -> &str {
    self.description()
  }
}

pub struct Runtime;

impl Runtime {
  pub fn run(ctx: &mut Context) -> Result<()> {
    let task = Box::new(main::MainTask) as Box<dyn Task>;
    Self::run_task(ctx, task)?;
    Ok(())
  }

  pub fn run_task(ctx: &mut Context, mut task: Box<dyn Task>) -> Result<()> {
    task.run(ctx)?;
    if let Some(next_tasks) = task.next() {
      for next_task in next_tasks {
        if next_task.condition(ctx) {
          Self::run_task(ctx, next_task)?;
        }
      }
    }
    Ok(())
  }
}
