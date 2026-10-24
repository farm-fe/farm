use crate::{DocPlan, PublicPage, prepare::farm_path, routes};
use farmfe_docs_config::ValidatedConfig;
use farmfe_docs_content::{ContentReference, ParsedContent, ReferenceKind};
use std::{
  collections::{HashMap, HashSet},
  path::{Path, PathBuf},
  sync::Arc,
};

pub struct Index<'a> {
  config: &'a ValidatedConfig,
  plan: &'a DocPlan,
  files: HashMap<String, Vec<usize>>,
  urls: HashMap<String, usize>,
  source_aliases: HashMap<String, Vec<usize>>,
  anchors: HashMap<String, HashSet<String>>,
}
fn error(reference: &ContentReference, code: &str, message: impl std::fmt::Display) -> String {
  format!(
    "[{code}] {}:{}:{}: {message}; target {:?}\n{}",
    reference.file, reference.line, reference.column, reference.target, reference.snippet
  )
}
pub(crate) fn decode(reference: &ContentReference, value: &str) -> Result<String, String> {
  let bytes = value.as_bytes();
  for (index, byte) in bytes.iter().enumerate() {
    if *byte == b'%'
      && (index + 2 >= bytes.len()
        || !bytes[index + 1].is_ascii_hexdigit()
        || !bytes[index + 2].is_ascii_hexdigit())
    {
      return Err(error(
        reference,
        "DOCS_LINK_ENCODING",
        "invalid percent escape",
      ));
    }
  }
  percent_encoding::percent_decode_str(value)
    .decode_utf8()
    .map(|value| value.into_owned())
    .map_err(|cause| error(reference, "DOCS_LINK_ENCODING", cause))
}
fn lexical(parent: &Path, value: &str) -> Option<PathBuf> {
  let mut path = parent.to_path_buf();
  for part in value.split('/') {
    match part {
      "" | "." => {}
      ".." => {
        if !path.pop() {
          return None;
        }
      }
      part => path.push(part),
    }
  }
  Some(path)
}
impl<'a> Index<'a> {
  pub fn is_resource_reference(&self, reference: &ContentReference, resolved: &str) -> bool {
    if resolved != reference.target || reference.target.starts_with("//") {
      return false;
    }
    let path = reference.target.split(['?', '#']).next().unwrap_or("");
    if path.contains(':') {
      return false;
    }
    if reference.kind == ReferenceKind::Link
      && self
        .urls
        .contains_key(&routes::collision_key(&routes::public_path(
          path,
          &self.config.public,
        )))
    {
      return false;
    }
    reference.kind == ReferenceKind::Image
      || (!path.ends_with('/')
        && Path::new(path)
          .extension()
          .is_some_and(|extension| extension != "md" && extension != "mdx"))
  }
  pub fn new(
    config: &'a ValidatedConfig,
    plan: &'a DocPlan,
    documents: &HashMap<(String, String), Arc<ParsedContent>>,
  ) -> Result<Self, String> {
    let mut files: HashMap<String, Vec<usize>> = HashMap::new();
    let mut anchors = HashMap::new();
    let mut source_aliases: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, (route, module)) in plan.routes.iter().zip(&plan.modules).enumerate() {
      if route.page.fallback_source.is_none() && module.generated_source.is_none() {
        files
          .entry(module.source_path.clone())
          .or_default()
          .push(index);
      }
      let collection = route
        .page
        .fallback_source
        .as_ref()
        .map(|source| &source.collection)
        .unwrap_or(&route.page.collection);
      let ids =
        if let Some(document) = documents.get(&(module.source_path.clone(), collection.clone())) {
          document.anchors()?
        } else if module.language != crate::ContentLanguage::Markdown {
          module.static_anchors.iter().cloned().collect()
        } else {
          return Err("[DOCS_INDEX] missing parsed anchors".into());
        };
      anchors.insert(route.page.key.clone(), ids);
      if module.generated_source.is_some() || module.language != crate::ContentLanguage::Markdown {
        continue;
      }
      let source_collection = config
        .collections
        .iter()
        .find(|candidate| &candidate.public.id == collection)
        .ok_or("[DOCS_INDEX] source collection is absent")?;
      let owner = config
        .collections
        .iter()
        .find(|candidate| candidate.public.id == route.page.collection)
        .ok_or("[DOCS_INDEX] page collection is absent")?;
      let directory = farm_path(&source_collection.directory)?;
      let relative = Path::new(&module.source_path)
        .strip_prefix(Path::new(&directory))
        .map_err(|cause| format!("[DOCS_INDEX] source alias is outside its collection: {cause}"))?;
      let alias = format!(
        "{}{}{}",
        config.public.base_path,
        owner.public.route_base.trim_start_matches('/'),
        farm_path(relative)?
      );
      source_aliases
        .entry(routes::collision_key(&alias))
        .or_default()
        .push(index);
    }
    Ok(Self {
      config,
      plan,
      files,
      anchors,
      source_aliases,
      urls: plan
        .routes
        .iter()
        .enumerate()
        .map(|(index, route)| (routes::collision_key(&route.page.route_path), index))
        .collect(),
    })
  }
  fn finish(
    &self,
    reference: &ContentReference,
    page: &PublicPage,
    query: &str,
    fragment: Option<&str>,
    validate_anchor: bool,
  ) -> Result<String, String> {
    if let Some(fragment) = fragment.filter(|fragment| validate_anchor && !fragment.is_empty()) {
      let id = decode(reference, fragment)?;
      if !self
        .anchors
        .get(&page.key)
        .is_some_and(|anchors| anchors.contains(&id))
      {
        return Err(error(
          reference,
          "DOCS_LINK_ANCHOR",
          format!("anchor {id:?} is absent from {}", page.route_path),
        ));
      }
    }
    Ok(format!(
      "{}{}{}",
      page.route_path,
      query,
      fragment
        .map(|fragment| format!("#{fragment}"))
        .unwrap_or_default()
    ))
  }
  pub fn resolve(
    &self,
    reference: &ContentReference,
    context: &PublicPage,
  ) -> Result<String, String> {
    self.resolve_inner(reference, context, true)
  }
  pub(crate) fn resolve_without_anchor_validation(
    &self,
    reference: &ContentReference,
    context: &PublicPage,
  ) -> Result<String, String> {
    self.resolve_inner(reference, context, false)
  }
  fn resolve_inner(
    &self,
    reference: &ContentReference,
    context: &PublicPage,
    validate_anchor: bool,
  ) -> Result<String, String> {
    let target = &reference.target;
    if target.starts_with("//") {
      return Ok(target.clone());
    }
    if let Some((scheme, _)) = target.split_once(':').filter(|(scheme, _)| {
      scheme
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphabetic)
        && scheme
          .chars()
          .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
    }) {
      if ["http", "https", "mailto", "tel"].contains(&scheme.to_ascii_lowercase().as_str())
        || (scheme.eq_ignore_ascii_case("data") && reference.kind == ReferenceKind::Image)
      {
        return Ok(target.clone());
      }
      return Err(error(
        reference,
        "DOCS_LINK_PROTOCOL",
        format!("unsupported scheme {scheme}"),
      ));
    }
    if reference.kind == ReferenceKind::Image {
      return Ok(target.clone());
    } // Asset graph conversion belongs to P05.
    let (without_fragment, fragment) = target
      .split_once('#')
      .map(|(path, fragment)| (path, Some(fragment)))
      .unwrap_or((target, None));
    let (path, query) = without_fragment
      .split_once('?')
      .map(|(path, query)| (path, format!("?{query}")))
      .unwrap_or((without_fragment, String::new()));
    if path.is_empty() {
      return self.finish(reference, context, &query, fragment, validate_anchor);
    }
    let path = decode(reference, path)?;
    if path.contains('\\') || path.chars().any(char::is_control) {
      return Err(error(
        reference,
        "DOCS_LINK_ESCAPE",
        "invalid path separator or control character",
      ));
    }
    if path.starts_with('/') {
      if path.contains("//") || path.split('/').any(|part| part == "." || part == "..") {
        return Err(error(
          reference,
          "DOCS_LINK_ESCAPE",
          "public path contains an escape or ambiguous separator",
        ));
      }
      let base = &self.config.public.base_path;
      let path = if path == base.trim_end_matches('/') || path.starts_with(base) {
        path
      } else {
        format!(
          "{}{}",
          self.config.public.base_path,
          path.trim_start_matches('/')
        )
      };
      let canonical = routes::public_path(&path, &self.config.public);
      if let Some(index) = self.urls.get(&routes::collision_key(&canonical)) {
        return self.finish(
          reference,
          &self.plan.routes[*index].page,
          &query,
          fragment,
          validate_anchor,
        );
      }
      if let Some(owners) = self.source_aliases.get(&routes::collision_key(&path)) {
        if owners.len() != 1 {
          let targets = owners
            .iter()
            .map(|index| self.plan.routes[*index].page.route_path.as_str())
            .collect::<Vec<_>>()
            .join(", ");
          return Err(error(
            reference,
            "DOCS_LINK_AMBIGUOUS",
            format!("source alias {path} has multiple public targets: {targets}"),
          ));
        }
        return self.finish(
          reference,
          &self.plan.routes[owners[0]].page,
          &query,
          fragment,
          validate_anchor,
        );
      }
      if Path::new(&path)
        .extension()
        .is_some_and(|extension| extension != "md" && extension != "mdx")
      {
        return Ok(target.clone());
      } // Explicit asset URL, handled by P05.
      return Err(error(
        reference,
        "DOCS_LINK_MISSING",
        format!("public page {canonical} is absent"),
      ));
    }
    let parent = Path::new(&reference.file)
      .parent()
      .ok_or_else(|| error(reference, "DOCS_LINK_ESCAPE", "source has no parent"))?;
    let target_path = lexical(parent, &path)
      .ok_or_else(|| error(reference, "DOCS_LINK_ESCAPE", "path leaves filesystem root"))?;
    let inside = self.config.collections.iter().any(|collection| {
      farm_path(&collection.directory).is_ok_and(|directory| {
        farm_path(&target_path)
          .is_ok_and(|path| path == directory || path.starts_with(&format!("{directory}/")))
      })
    });
    if !inside {
      if target_path
        .extension()
        .is_some_and(|extension| extension != "md" && extension != "mdx")
      {
        return Ok(target.clone()); // Rust asset registry applies the site-root boundary.
      }
      return Err(error(
        reference,
        "DOCS_LINK_ESCAPE",
        "relative document path leaves configured collections",
      ));
    }
    let mut candidates = vec![target_path.clone()];
    if !matches!(
      target_path
        .extension()
        .and_then(|extension| extension.to_str()),
      Some("md" | "mdx")
    ) {
      candidates.extend([
        PathBuf::from(format!("{}.md", target_path.display())),
        PathBuf::from(format!("{}.mdx", target_path.display())),
        target_path.join("index.md"),
        target_path.join("index.mdx"),
      ]);
    }
    let index = candidates.iter().find_map(|candidate| {
      candidate
        .canonicalize()
        .ok()
        .and_then(|file| farm_path(&file).ok())
        .and_then(|file| self.files.get(&file))
    });
    let Some(index) = index else {
      // A real localized page can refer to a missing physical translation whose
      // logical page was explicitly supplied by Rust's fallback expansion.
      // Resolve only that collection's existing fallback source aliases; never
      // infer a route from the filename or silently cross locale/version.
      if let Some(owner) = self
        .config
        .collections
        .iter()
        .find(|owner| owner.public.id == context.collection)
      {
        let directory = farm_path(&owner.directory)?;
        let mut fallback_targets = std::collections::BTreeSet::new();
        for candidate in &candidates {
          let candidate = farm_path(candidate)?;
          let Ok(relative) = Path::new(&candidate).strip_prefix(Path::new(&directory)) else {
            continue;
          };
          let alias = format!(
            "{}{}{}",
            self.config.public.base_path,
            owner.public.route_base.trim_start_matches('/'),
            farm_path(relative)?
          );
          if let Some(owners) = self.source_aliases.get(&routes::collision_key(&alias)) {
            fallback_targets.extend(owners.iter().copied().filter(|index| {
              let page = &self.plan.routes[*index].page;
              page.collection == context.collection && page.fallback_source.is_some()
            }));
          }
        }
        if fallback_targets.len() > 1 {
          return Err(error(
            reference,
            "DOCS_LINK_AMBIGUOUS",
            "multiple fallback documents own the relative source alias",
          ));
        }
        if let Some(index) = fallback_targets.into_iter().next() {
          return self.finish(
            reference,
            &self.plan.routes[index].page,
            &query,
            fragment,
            validate_anchor,
          );
        }
      }
      if target_path
        .extension()
        .is_some_and(|extension| extension != "md" && extension != "mdx")
      {
        return Ok(target.clone());
      } // Explicit asset, not an unknown Markdown file.
      return Err(error(
        reference,
        "DOCS_LINK_MISSING",
        format!(
          "document {} is absent from the index",
          target_path.display()
        ),
      ));
    };
    let origin = if let Some(source) = &context.fallback_source {
      &self
        .plan
        .routes
        .iter()
        .find(|route| route.page.key == source.key)
        .ok_or_else(|| error(reference, "DOCS_INDEX", "fallback source is absent"))?
        .page
    } else {
      context
    };
    let local = index
      .iter()
      .copied()
      .find(|index| self.plan.routes[*index].page.collection == origin.collection);
    let matched: Vec<_> = index
      .iter()
      .copied()
      .filter(|index| {
        let page = &self.plan.routes[*index].page;
        page.locale == origin.locale && page.version == origin.version
      })
      .collect();
    let selected = if let Some(local) = local {
      local
    } else if matched.len() == 1 {
      matched[0]
    } else if matched.len() > 1 {
      return Err(error(
        reference,
        "DOCS_LINK_AMBIGUOUS",
        "multiple collections own the target; use an explicit public URL",
      ));
    } else {
      return Err(error(
        reference,
        "DOCS_LINK_CONTEXT",
        format!(
          "file link changes locale/version from {} {:?} to {} {:?}; use an explicit public URL",
          origin.locale,
          origin.version,
          self.plan.routes[index[0]].page.locale,
          self.plan.routes[index[0]].page.version
        ),
      ));
    };
    let mut page = &self.plan.routes[selected].page;
    if context.fallback_source.is_some() {
      if let Some(key) = page
        .alternates
        .iter()
        .find(|alternate| {
          alternate.locale == context.locale && alternate.version == context.version
        })
        .and_then(|alternate| alternate.key.as_ref())
      {
        page = &self
          .plan
          .routes
          .iter()
          .find(|route| &route.page.key == key)
          .ok_or_else(|| error(reference, "DOCS_INDEX", "alternate target is absent"))?
          .page;
      }
    }
    self.finish(reference, page, &query, fragment, validate_anchor)
  }
}
