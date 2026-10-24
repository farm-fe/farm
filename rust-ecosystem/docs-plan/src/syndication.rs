//! Auxiliary SEO output names and real search targets, shared by all native consumers.
use crate::DocPlan;
use farmfe_docs_config::{CollectionKind, PublicCollection, PublicSiteConfig};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenSearchTarget {
  pub locale: String,
  pub output_path: String,
  pub route_path: String,
}

pub fn atom_output(collection: &PublicCollection) -> String {
  let prefix = collection.route_base.trim_matches('/');
  if prefix.is_empty() {
    "atom.xml".into()
  } else {
    format!("{prefix}/atom.xml")
  }
}

pub fn open_search_output(site: &PublicSiteConfig, locale: &str) -> String {
  if locale == site.locales.default {
    "opensearch.xml".into()
  } else {
    format!("{locale}/opensearch.xml")
  }
}

/// Additional reserved paths, without reserving disabled optional features.
pub fn output_paths(site: &PublicSiteConfig) -> Vec<String> {
  let mut paths = Vec::new();
  if site.seo.atom.is_some() {
    paths.extend(
      site
        .collections
        .iter()
        .filter(|collection| collection.kind == CollectionKind::Blog)
        .map(atom_output),
    );
  }
  if site
    .search
    .as_ref()
    .and_then(|search| search.open_search.as_ref())
    .is_some()
  {
    paths.extend(
      site
        .locales
        .items
        .iter()
        .map(|locale| open_search_output(site, locale)),
    );
  }
  paths
}

pub fn open_search_targets(plan: &DocPlan) -> Result<Vec<OpenSearchTarget>, String> {
  let Some(config) = plan
    .site
    .search
    .as_ref()
    .and_then(|search| search.open_search.as_ref())
  else {
    return Ok(Vec::new());
  };
  plan.site.locales.items.iter().map(|locale| {
    let pages: Vec<_> = plan.routes.iter().filter(|route| route.page.kind == "page" && route.page.id == config.page_id && &route.page.locale == locale && route.page.fallback_source.is_none()).collect();
    if pages.len() != 1 {
      return Err(format!("[DOCS_SEARCH_PAGE] search.openSearch.pageId {:?}: require exactly one real page for locale {locale}, found {}", config.page_id, pages.len()));
    }
    let output_path = open_search_output(&plan.site, locale);
    crate::routes::validate_output(&output_path, "search.openSearch")?;
    Ok(OpenSearchTarget { locale: locale.clone(), output_path, route_path: pages[0].page.route_path.clone() })
  }).collect()
}

pub(crate) fn validate(plan: &DocPlan) -> Result<(), String> {
  farmfe_docs_config::validate_syndication(&plan.site.seo, plan.site.search.as_ref())
    .map_err(|error| error.to_string())?;
  let paths = output_paths(&plan.site);
  for path in &paths {
    crate::routes::validate_output(path, "SEO auxiliary output")?;
  }
  crate::static_files::validate_names(&paths)?;
  open_search_targets(plan)?;
  Ok(())
}

pub(crate) fn discovery_links(
  plan: &DocPlan,
  locale: &str,
) -> Result<Vec<crate::PublicDiscoveryLink>, String> {
  let mut links = Vec::new();
  for collection in plan
    .site
    .collections
    .iter()
    .filter(|collection| collection.kind == CollectionKind::Blog && collection.locale == locale)
  {
    let prefix = collection.route_base.trim_matches('/');
    let rss = if prefix.is_empty() {
      "rss.xml".into()
    } else {
      format!("{prefix}/rss.xml")
    };
    let mut feeds = vec![("application/rss+xml", "RSS", rss)];
    if plan.site.seo.atom.is_some() {
      feeds.push(("application/atom+xml", "Atom", atom_output(collection)));
    }
    for (resource_type, label, output) in feeds {
      links.push(crate::PublicDiscoveryLink {
        rel: "alternate".into(),
        resource_type: resource_type.into(),
        title: format!("{} — {} {label}", plan.site.title, collection.id),
        href: crate::seo_url(&plan.site, &format!("{}{output}", plan.site.base_path))?,
      });
    }
  }
  if let Some(config) = plan
    .site
    .search
    .as_ref()
    .and_then(|search| search.open_search.as_ref())
  {
    let target = open_search_targets(plan)?
      .into_iter()
      .find(|target| target.locale == locale)
      .ok_or_else(|| format!("[DOCS_SEARCH_PAGE] no target for locale {locale}"))?;
    links.push(crate::PublicDiscoveryLink {
      rel: "search".into(),
      resource_type: "application/opensearchdescription+xml".into(),
      title: config.short_name.clone(),
      href: crate::seo_url(
        &plan.site,
        &format!("{}{}", plan.site.base_path, target.output_path),
      )?,
    });
  }
  Ok(links)
}

pub(crate) fn discovery_html(links: &[crate::PublicDiscoveryLink]) -> String {
  links
    .iter()
    .map(|link| {
      format!(
        "<link data-farm-docs-discovery=\"\" rel=\"{}\" type=\"{}\" title=\"{}\" href=\"{}\">",
        crate::seo::html_attribute(&link.rel),
        crate::seo::html_attribute(&link.resource_type),
        crate::seo::html_attribute(&link.title),
        crate::seo::html_attribute(&link.href)
      )
    })
    .collect()
}
