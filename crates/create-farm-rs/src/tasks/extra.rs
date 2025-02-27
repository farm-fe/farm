use std::cell::RefCell;

use crate::utils::prompts;

#[derive(Default)]
pub struct ExtraSelectTask {
  pub selected: RefCell<Option<Vec<Box<dyn super::Task>>>>,
}

impl ExtraSelectTask {
  pub fn new() -> Self {
    Self::default()
  }
}

impl super::Task for ExtraSelectTask {
  fn run(&mut self, _ctx: &mut crate::utils::context::Context) -> anyhow::Result<()> {
    let extras = vec![Box::new(super::biome::BiomeTask) as Box<dyn super::Task>];
    let selected = prompts::multi_select("Setup extra tools", extras, Some(&[false]))?;
    self.selected = RefCell::new(Some(selected));
    Ok(())
  }

  fn next(&self) -> Option<Vec<Box<dyn super::Task>>> {
    let mut selected = self.selected.take().unwrap();
    selected.push(Box::new(super::render::RenderTask::new()));
    Some(selected)
  }
}
