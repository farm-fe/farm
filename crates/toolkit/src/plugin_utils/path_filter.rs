use farmfe_core::config::config_regex::ConfigRegex;

pub struct PathFilter<'a> {
  include: &'a Vec<ConfigRegex>,
  exclude: &'a Vec<ConfigRegex>,
}

impl<'a> PathFilter<'a> {
  pub fn new(include: &'a Vec<ConfigRegex>, exclude: &'a Vec<ConfigRegex>) -> Self {
    Self { include, exclude }
  }

  pub fn execute(&self, path: &str) -> bool {
    (self.include.is_empty() || self.include.iter().any(|regex| regex.is_match(path)))
      && (self.exclude.is_empty() || !self.exclude.iter().any(|regex| regex.is_match(path)))
  }
}
