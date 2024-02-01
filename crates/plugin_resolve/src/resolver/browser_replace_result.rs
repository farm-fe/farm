pub enum BrowserReplaceResult {
  Str(String),
  Alias(String),
}

impl BrowserReplaceResult {
  pub fn as_str(self) -> Option<String> {
    match self {
      BrowserReplaceResult::Str(s) => Some(s),
      BrowserReplaceResult::Alias(_) => None,
    }
  }
}
