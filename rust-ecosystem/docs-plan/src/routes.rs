use std::collections::{HashMap, HashSet};
use unicode_normalization::UnicodeNormalization;

/// Apply the site's URL policy without deriving physical output paths from public URLs.
pub fn public_path(value: &str, site: &farmfe_docs_config::PublicSiteConfig) -> String {
  if value.trim_end_matches('/') == site.base_path.trim_end_matches('/') {
    return site.base_path.clone();
  }
  match site.trailing_slash {
    farmfe_docs_config::TrailingSlash::Always => format!("{}/", value.trim_end_matches('/')),
    farmfe_docs_config::TrailingSlash::Never => value.trim_end_matches('/').into(),
  }
}

/// Content expansion and physical paths are complete before public identities are finalized.
/// All later alternates, sidebars and reference emission consume these same final routes.
pub fn apply_policy(
  config: &farmfe_docs_config::ValidatedConfig,
  plan: &mut crate::DocPlan,
) -> Result<(), String> {
  for route in &mut plan.routes {
    route.page.route_path = public_path(&route.page.route_path, &config.public);
    route.page.canonical_path = route.page.route_path.clone();
  }
  let by_key: HashMap<_, _> = plan
    .routes
    .iter()
    .map(|route| (route.page.key.clone(), route.page.route_path.clone()))
    .collect();
  let by_path: HashMap<_, _> = plan
    .routes
    .iter()
    .enumerate()
    .map(|(index, route)| (collision_key(&route.page.route_path), index))
    .collect();
  for route in &mut plan.routes {
    if let Some(source) = &mut route.page.fallback_source {
      source.route_path = by_key
        .get(&source.key)
        .ok_or("[DOCS_ROUTE_POLICY] missing fallback source")?
        .clone();
    }
    for navigation in [&mut route.page.previous, &mut route.page.next]
      .into_iter()
      .flatten()
    {
      navigation.href = by_key
        .get(&navigation.key)
        .ok_or("[DOCS_ROUTE_POLICY] missing navigation target")?
        .clone();
    }
  }
  let mut seen = HashSet::new();
  let mut targets = vec![];
  for (rule_index, rule) in config.canonical_overrides.iter().enumerate() {
    let locate = |path: &str| -> Result<usize, String> {
      let normalized = public_path(path, &config.public);
      let index = *by_path.get(&collision_key(&normalized)).ok_or_else(|| {
        format!("[DOCS_CANONICAL_MISSING] canonicalOverrides[{rule_index}]: page {path} is absent")
      })?;
      if plan.routes[index].page.route_path != normalized {
        return Err(format!(
          "[DOCS_CANONICAL_PATH] canonicalOverrides[{rule_index}]: spelling of {path} differs from the actual public route"
        ));
      }
      Ok(index)
    };
    let source = locate(&rule.path)?;
    let target = locate(&rule.canonical_path)?;
    if !seen.insert(source) {
      return Err(format!(
        "[DOCS_CANONICAL_DUPLICATE] canonicalOverrides[{rule_index}]: repeated page {}",
        rule.path
      ));
    }
    plan.routes[source].page.canonical_path = rule.canonical_path.clone();
    targets.push((rule_index, source, target));
  }
  for (rule_index, source, target) in targets {
    if source != target
      && plan.routes[source].page.canonical_path != plan.routes[target].page.canonical_path
    {
      return Err(format!(
        "[DOCS_CANONICAL_TARGET] canonicalOverrides[{rule_index}]: use the target's final canonical path directly; chains and cycles are not canonical"
      ));
    }
  }
  Ok(())
}

/// Compare conservatively across filesystems without changing the authored public URL.
pub fn collision_key(value: &str) -> String {
  value
    .nfc()
    .flat_map(char::to_uppercase)
    .collect::<String>()
    .nfc()
    .collect()
}

pub fn logical_id(value: &str) -> String {
  value.nfc().collect()
}

pub fn page_key(
  collection: &farmfe_docs_config::PublicCollection,
  id: &str,
) -> Result<String, String> {
  let identity = serde_json::to_vec(&(
    &collection.id,
    &collection.kind,
    &collection.locale,
    &collection.version,
    logical_id(id),
  ))
  .map_err(|error| error.to_string())?;
  Ok(crate::hash(&identity))
}

/// A route must have a portable physical representation on the supported host platforms.
pub fn validate_output(value: &str, source: &str) -> Result<(), String> {
  for part in value.split('/') {
    let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
    let device = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
      || ["COM", "LPT"].iter().any(|prefix| {
        stem.strip_prefix(prefix).is_some_and(|suffix| {
          let mut chars = suffix.chars();
          matches!(chars.next(), Some('1'..='9' | '¹' | '²' | '³')) && chars.next().is_none()
        })
      });
    if part.is_empty()
      || part == "."
      || part == ".."
      || part.ends_with(['.', ' '])
      || device
      || part
        .chars()
        .any(|ch| ch.is_control() || "<>:\\|?*\"".contains(ch))
    {
      return Err(format!(
        "[DOCS_OUTPUT_PATH] {source}: nonportable output segment {part:?} in {value}"
      ));
    }
  }
  Ok(())
}
