pub struct LogStore {
  warnings: Vec<String>,
  errors: Vec<String>,
}

impl LogStore {
  pub fn new() -> Self {
    Self {
      warnings: vec![],
      errors: vec![],
    }
  }

  pub fn add_warning(&mut self, warning: String) {
    self.warnings.push(warning);
  }

  pub fn add_error(&mut self, error: String) {
    self.errors.push(error);
  }

  pub fn warnings(&self) -> &Vec<String> {
    &self.warnings
  }

  pub fn errors(&self) -> &Vec<String> {
    &self.errors
  }
}
