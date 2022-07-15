use std::sync::Arc;

use farmfe_core::{config::Config, context::CompilationContext, plugin::Plugin};
use update::UpdateOutput;

pub mod build;
pub mod generate;
pub mod update;

pub struct Compiler {
  context: CompilationContext,
}

impl Compiler {
  pub fn new(config: Config, plugins: Vec<Arc<dyn Plugin + Send + Sync>>) -> Self {
    Self {
      context: CompilationContext::new(config, plugins),
    }
  }

  pub fn compile(&self) {
    println!("call compile, config {:?}", self.context.config.input);
  }

  pub fn update(&self, paths: Vec<String>) -> UpdateOutput {
    UpdateOutput {}
  }
}
