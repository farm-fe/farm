use anyhow::Result;

use crate::{args, template};

#[derive(Default)]
pub struct Context {
  pub engine: tera::Tera,
  pub inner: tera::Context,
  pub options: args::Args,
  pub template: template::Template,
}

impl Context {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn new_with_options(options: args::Args) -> Self {
    Self {
      options,
      ..Self::default()
    }
  }

  pub fn insert<T: serde::Serialize>(&mut self, key: &str, value: T) {
    self.inner.insert(key, &value);
  }

  pub fn remove<T: serde::Serialize>(&mut self, key: &str) {
    self.inner.remove(key);
  }

  pub fn get(&self, key: &str) -> Option<&tera::Value> {
    self.inner.get(key)
  }

  pub fn equals(&self, key: &str, value: &tera::Value) -> bool {
    self.inner.get(key).unwrap_or(&tera::Value::Null) == value
  }

  pub fn render(&mut self, input: &str) -> Result<String> {
    Ok(self.engine.render_str(input, &self.inner)?)
  }

  pub fn set_template(&mut self, template: template::Template) {
    self.template = template;
  }

  pub fn template(&self) -> &template::Template {
    &self.template
  }
}
