use farmfe_core::{
  plugin::PluginProcessModuleHookParam,
  swc_css_ast::{ImportHref, Str, Url, UrlValue},
};
use farmfe_docs_plan::{DocPlan, static_resource_url};
use farmfe_toolkit::sourcemap::SourceMap;
use farmfe_toolkit::swc_css_visit::{VisitMut, VisitMutWith};

struct StaticUrls<'a> {
  plan: &'a DocPlan,
  file: String,
  content: &'a str,
  start: u32,
  maps: Vec<SourceMap>,
  error: Option<String>,
}
impl VisitMut for StaticUrls<'_> {
  // CSS imports are stylesheet dependencies, not opaque static resources.
  fn visit_mut_import_href(&mut self, _value: &mut ImportHref) {}

  fn visit_mut_url(&mut self, url: &mut Url) {
    if self.error.is_some() {
      return;
    }
    let Some(value) = url.value.as_mut() else {
      return;
    };
    let (source, span) = match &**value {
      UrlValue::Str(value) => (value.value.to_string(), value.span),
      UrlValue::Raw(value) => (value.value.to_string(), value.span),
    };
    if !source.starts_with('/') || source.starts_with("//") {
      return;
    }
    let offset = (span.lo.0.saturating_sub(self.start) as usize).min(self.content.len());
    let prefix = self.content.get(..offset).unwrap_or("");
    let mut line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let mut column = prefix
      .rsplit('\n')
      .next()
      .unwrap_or("")
      .encode_utf16()
      .count();
    let mut file = self.file.clone();
    for map in &self.maps {
      if let Some(token) = map.lookup_token(line as u32, column as u32) {
        line = token.get_src_line() as usize;
        column = token.get_src_col() as usize;
        if let Some(source) = token.get_source() {
          file = source.to_string();
        }
      }
    }
    match static_resource_url(self.plan, &source, &file, line + 1, column + 1) {
      Ok(mapped) => {
        **value = UrlValue::Str(Str {
          span,
          value: mapped.into(),
          raw: None,
        })
      }
      Err(error) => self.error = Some(error),
    }
  }
}
pub(super) fn rewrite(
  param: &mut PluginProcessModuleHookParam,
  plan: &DocPlan,
) -> Result<(), String> {
  let ast = &mut param.meta.as_css_mut().ast;
  let mut visitor = StaticUrls {
    plan,
    file: param.module_id.to_string(),
    content: param.content.as_str(),
    start: ast.span.lo.0,
    maps: param
      .source_map_chain
      .iter()
      .rev()
      .map(|map| {
        SourceMap::from_slice(map.as_bytes()).map_err(|error| {
          format!(
            "[DOCS_CSS_SOURCEMAP] {}: {error}",
            param.module_id.to_string()
          )
        })
      })
      .collect::<Result<Vec<_>, _>>()?,
    error: None,
  };
  ast.visit_mut_with(&mut visitor);
  match visitor.error {
    Some(error) => Err(error),
    None => Ok(()),
  }
}
