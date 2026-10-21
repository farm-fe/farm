use crate::{links::Index, routes::logical_id, DocPlan, PublicPage};
use farmfe_docs_config::{LinkException, LinkExceptionCode, ValidatedConfig};
use farmfe_docs_content::ContentReference;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct AppliedDiagnostics {
  pub private: Vec<String>,
  pub public: Vec<String>,
}

pub struct Registry<'a> {
  rules: &'a [LinkException],
  by_reference: HashMap<(String, String, String), usize>,
  used: Vec<bool>,
  diagnostics: AppliedDiagnostics,
}
impl<'a> Registry<'a> {
  pub fn new(config: &'a ValidatedConfig, plan: &DocPlan) -> Result<Self, String> {
    let sources: HashSet<_> = plan
      .routes
      .iter()
      .map(|route| (route.page.collection.clone(), logical_id(&route.page.id)))
      .collect();
    let mut by_reference = HashMap::new();
    for (index, rule) in config.link_exceptions.iter().enumerate() {
      let id = logical_id(&rule.id);
      if !sources.contains(&(rule.collection.clone(), id.clone())) {
        return Err(format!(
          "[DOCS_LINK_EXCEPTION_SOURCE] linkExceptions[{index}]: document {:?} is absent from collection {}",
          rule.id, rule.collection
        ));
      }
      if by_reference
        .insert((rule.collection.clone(), id, rule.target.clone()), index)
        .is_some()
      {
        return Err(format!(
          "[DOCS_CONFIG_LINK_EXCEPTION] linkExceptions[{index}]: duplicate canonical collection/id/target"
        ));
      }
    }
    Ok(Self {
      rules: &config.link_exceptions,
      by_reference,
      used: vec![false; config.link_exceptions.len()],
      diagnostics: AppliedDiagnostics::default(),
    })
  }
  pub fn resolve(
    &mut self,
    resolver: &Index<'_>,
    reference: &ContentReference,
    page: &PublicPage,
  ) -> Result<String, String> {
    let cause = match resolver.resolve(reference, page) {
      Ok(href) => return Ok(href),
      Err(cause) => cause,
    };
    let Some(index) = self
      .by_reference
      .get(&(
        page.collection.clone(),
        logical_id(&page.id),
        reference.target.clone(),
      ))
      .copied()
    else {
      return Err(cause);
    };
    let rule = &self.rules[index];
    // The actual resolver must first report the exact permitted error. This cannot
    // bypass protocol, encoding, escape, ambiguity or locale/version checks.
    if !cause.starts_with(&format!("[{}] ", rule.code.as_str())) {
      return Err(cause);
    }
    let href = match rule.code {
      LinkExceptionCode::Anchor => resolver.resolve_without_anchor_validation(reference, page)?,
      LinkExceptionCode::Missing => reference.target.clone(),
    };
    self.used[index] = true;
    self.diagnostics.private.push(format!(
      "[DOCS_LINK_EXCEPTION] linkExceptions[{index}] {}/{}: {}\n{cause}",
      page.collection, page.id, rule.reason
    ));
    self.diagnostics.public.push(format!(
      "[DOCS_LINK_EXCEPTION] {}/{}:{}:{} -> {:?}: {}; configured historical exception",
      page.collection,
      page.id,
      reference.line,
      reference.column,
      reference.target,
      rule.code.as_str()
    ));
    Ok(href)
  }
  pub fn finish(self) -> Result<AppliedDiagnostics, String> {
    if let Some(index) = self.used.iter().position(|used| !used) {
      let rule = &self.rules[index];
      return Err(format!(
        "[DOCS_LINK_EXCEPTION_UNUSED] linkExceptions[{index}] {}/{} -> {:?}: expected {} was not produced; remove or update the stale exception",
        rule.collection,
        rule.id,
        rule.target,
        rule.code.as_str()
      ));
    }
    Ok(self.diagnostics)
  }
}
