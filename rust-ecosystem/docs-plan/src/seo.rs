use crate::{DocPlan, PublicSeoHead, SeoAlternate};
use farmfe_docs_config::{FallbackIndexing, PublicSiteConfig};
use std::collections::{BTreeMap, BTreeSet};

/// Serialize an unescaped Rust route in the site's origin and basePath boundary.
pub fn seo_url(site: &PublicSiteConfig, path: &str) -> Result<String, String> {
  let fail = || format!("[DOCS_SEO_URL] invalid site route {path:?}");
  if !path.starts_with('/')
    || path.contains("//")
    || path.contains(['\\', '?', '#', '%'])
    || path.chars().any(|ch| ch.is_control() || ch.is_whitespace())
    || path.split('/').any(|part| matches!(part, "." | ".."))
    || (path != site.base_path.trim_end_matches('/') && !path.starts_with(&site.base_path))
  {
    return Err(fail());
  }
  let origin = url::Url::parse(&site.url).map_err(|_| fail())?;
  if !matches!(origin.scheme(), "https" | "http")
    || origin.origin().ascii_serialization() != site.url
  {
    return Err(fail());
  }
  let url = url::Url::parse(&format!("{}{path}", site.url)).map_err(|_| fail())?;
  if url.origin() != origin.origin() || url.query().is_some() || url.fragment().is_some() {
    return Err(fail());
  }
  Ok(url.into())
}

pub(crate) fn html_attribute(value: &str) -> String {
  value
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#39;")
}

pub(crate) fn heads(plan: &DocPlan) -> Result<BTreeMap<String, PublicSeoHead>, String> {
  let by_key: BTreeMap<_, _> = plan
    .routes
    .iter()
    .map(|route| (&route.page.key, &route.page))
    .collect();
  let by_path: BTreeMap<_, _> = plan
    .routes
    .iter()
    .map(|route| (&route.page.route_path, &route.page))
    .collect();
  let mut result = BTreeMap::new();
  for route in &plan.routes {
    let page = &route.page;
    // Explicit canonical spelling can retain a trailing slash while the site's
    // navigation policy is `never`. Both spellings identify the same real page.
    if !by_path.contains_key(&crate::routes::public_path(
      &page.canonical_path,
      &plan.site,
    )) {
      return Err(format!(
        "[DOCS_PLAN_SEO] canonical page is absent: {}",
        page.canonical_path
      ));
    }
    let robots = if page.fallback_source.is_some()
      && plan.site.seo.fallback_indexing == FallbackIndexing::Noindex
    {
      "noindex,follow"
    } else {
      "index,follow"
    };
    let mut alternates = BTreeSet::new();
    for alternate in &page.alternates {
      if alternate.missing || alternate.version != page.version {
        continue;
      }
      let Some(target) = alternate.key.as_ref().and_then(|key| by_key.get(key)) else {
        continue;
      };
      if target.fallback_source.is_some()
        || target.locale != alternate.locale
        || target.version != page.version
      {
        continue;
      }
      let canonical_path = crate::routes::public_path(&target.canonical_path, &plan.site);
      let Some(canonical) = by_path.get(&canonical_path) else {
        continue;
      };
      if canonical.locale != alternate.locale
        || canonical.version != page.version
        || canonical.fallback_source.is_some()
      {
        continue;
      }
      alternates.insert(SeoAlternate {
        locale: alternate.locale.clone(),
        href: seo_url(&plan.site, &target.canonical_path)?,
      });
    }
    let alternates: Vec<_> = alternates.into_iter().collect();
    let mut html = format!("<meta name=\"robots\" content=\"{robots}\">");
    if let Some(published) = page
      .blog
      .as_ref()
      .and_then(|post| post.published_at.as_deref().or(post.date.as_deref()))
    {
      html.push_str(&format!(
        "<meta property=\"article:published_time\" content=\"{}\">",
        html_attribute(published)
      ));
    }
    for alternate in &alternates {
      html.push_str(&format!(
        "<link rel=\"alternate\" hreflang=\"{}\" href=\"{}\">",
        html_attribute(&alternate.locale),
        html_attribute(&alternate.href)
      ));
    }
    let discovery_links = crate::syndication::discovery_links(plan, &page.locale)?;
    html.push_str(&crate::syndication::discovery_html(&discovery_links));
    if result
      .insert(
        page.key.clone(),
        PublicSeoHead {
          title: Some(if page.title == plan.site.title {
            page.title.clone()
          } else {
            format!("{} | {}", page.title, plan.site.title)
          }),
          canonical_url: seo_url(&plan.site, &page.canonical_path)?,
          robots: robots.into(),
          alternates,
          html,
          discovery_links,
        },
      )
      .is_some()
    {
      return Err("[DOCS_PLAN_SEO] duplicate page identity".into());
    }
  }
  Ok(result)
}
