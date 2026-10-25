use farmfe_docs_content::ParsedContent;

pub(crate) struct MetadataSource<'a> {
  pub file: &'a str,
  pub content: Option<&'a ParsedContent>,
}
pub(crate) trait DiagnosticSource {
  fn at(&self, field: &str) -> String;
}
impl DiagnosticSource for str {
  fn at(&self, _: &str) -> String {
    self.to_owned()
  }
}
impl DiagnosticSource for String {
  fn at(&self, _: &str) -> String {
    self.clone()
  }
}
impl DiagnosticSource for MetadataSource<'_> {
  fn at(&self, field: &str) -> String {
    match self
      .content
      .and_then(|content| content.metadata_locations.position(field))
    {
      Some(position) => format!("{}:{}:{}", self.file, position.line, position.column),
      None => self.file.to_owned(),
    }
  }
}
