use farmfe_core::swc_html_ast::{Document, Element};
use farmfe_toolkit::swc_html_visit::{VisitMut, VisitMutWith};

pub struct AbsolutePathHandler {
  pub public_path: String,
}

impl AbsolutePathHandler {
  /// Process the script and link with absolute paths manually added by users in HTML.
  /// Add the prefix of "publicPath" to its path.
  /// eg: <script src="/test.js"></script>  to  <script src="/publicPath/test.js"></script>
  pub fn add_public_path_prefix(&mut self, html_ast: &mut Document) {
    html_ast.visit_mut_with(self)
  }
}

impl VisitMut for AbsolutePathHandler {
  fn visit_mut_element(&mut self, element: &mut Element) {
    if matches!(element.tag_name.to_lowercase().as_str(), "script" | "link") {
      for attr in &mut element.attributes {
        // determine if the path start with /.
        if matches!(attr.name.to_lowercase().as_str(), "src" | "href")
          && !attr
            .value
            .clone()
            .unwrap_or_default()
            .starts_with(&self.public_path)
          && attr.value.clone().unwrap_or_default().starts_with("/")
        {
          attr.value = Some(
            format!(
              "{}{}",
              self.public_path,
              attr.value.clone().unwrap_or_default()
            )
            .replace("//", "/")
            .into(),
          );
        }
      }
    }
    element.visit_mut_children_with(self);
  }
}
