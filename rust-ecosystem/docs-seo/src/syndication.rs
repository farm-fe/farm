//! Opt-in syndication resources projected exclusively from verified Rust plans.
use super::{Result, Xml};
use chrono::{DateTime, Datelike, FixedOffset};
use farmfe_docs_config::{AtomConfig, PublicCollection};
use farmfe_docs_plan::{seo_url, syndication::OpenSearchTarget, DocPlan, PublicBlogPost};
use std::collections::{BTreeMap, BTreeSet};

fn timestamp(value: &str) -> Result<DateTime<FixedOffset>> {
  let date =
    DateTime::parse_from_rfc3339(value).map_err(|error| format!("[DOCS_SEO_DATE] {error}"))?;
  if !(1..=9999).contains(&date.year()) {
    return Err("[DOCS_SEO_DATE] publication year is outside 0001..9999".into());
  }
  // Keep the original offset: UTC normalization can cross year 0001 or 9999.
  Ok(date)
}

fn published(post: &PublicBlogPost) -> Result<Option<DateTime<FixedOffset>>> {
  if let Some(value) = &post.published_at {
    return timestamp(value).map(Some);
  }
  post
    .date
    .as_ref()
    .map(|value| timestamp(&format!("{value}T00:00:00Z")))
    .transpose()
}

pub(super) fn atom(
  plan: &DocPlan,
  collection: &PublicCollection,
  output: &str,
  config: &AtomConfig,
) -> Result<Vec<u8>> {
  let configured = timestamp(&config.updated)?;
  let mut updated = configured;
  let index = plan
    .blog_indexes
    .iter()
    .find(|index| index.collection == collection.id)
    .ok_or("[DOCS_SEO_PLAN] blog index missing")?;
  let by_key: BTreeMap<_, _> = plan
    .routes
    .iter()
    .map(|route| (&route.page.key, &route.page))
    .collect();
  let mut emitted = BTreeSet::new();
  let mut entries = Vec::new();
  for key in &index.posts {
    let page = by_key
      .get(key)
      .ok_or("[DOCS_SEO_PLAN] blog article missing")?;
    if page.fallback_source.is_some() {
      continue;
    }
    if page.kind != "blog" || page.collection != collection.id {
      return Err("[DOCS_SEO_PLAN] blog index contains a non-article or wrong collection".into());
    }
    let post = page
      .blog
      .as_ref()
      .ok_or("[DOCS_SEO_PLAN] article metadata missing")?;
    let head = plan
      .seo
      .get(key)
      .ok_or("[DOCS_SEO_PLAN] article head missing")?;
    if head.robots != "index,follow" || !emitted.insert(&head.canonical_url) {
      continue;
    }
    let publication = published(post)?;
    let entry_updated = publication.unwrap_or(configured);
    if entry_updated > updated {
      updated = entry_updated;
    }
    entries.push((*page, post, head, publication, entry_updated));
  }
  let root = seo_url(
    &plan.site,
    &format!(
      "{}{}",
      plan.site.base_path,
      collection.route_base.trim_start_matches('/')
    ),
  )?;
  let self_url = seo_url(&plan.site, &format!("{}{output}", plan.site.base_path))?;
  let mut xml = Xml::new()?;
  xml.start(
    "feed",
    &[
      ("xmlns", "http://www.w3.org/2005/Atom"),
      ("xml:lang", &collection.locale),
    ],
  )?;
  xml.element("id", &root)?;
  xml.element("title", &format!("{} — {}", plan.site.title, collection.id))?;
  xml.element("updated", &updated.to_rfc3339())?;
  xml.empty(
    "link",
    &[("rel", "alternate"), ("type", "text/html"), ("href", &root)],
  )?;
  xml.empty(
    "link",
    &[
      ("rel", "self"),
      ("type", "application/atom+xml"),
      ("href", &self_url),
    ],
  )?;
  xml.start("author", &[])?;
  xml.element("name", &config.author)?;
  xml.end("author")?;
  for (page, post, head, publication, entry_updated) in entries {
    xml.start("entry", &[])?;
    xml.element("id", &head.canonical_url)?;
    xml.element("title", &page.title)?;
    xml.empty(
      "link",
      &[
        ("rel", "alternate"),
        ("type", "text/html"),
        ("href", &head.canonical_url),
      ],
    )?;
    xml.element("updated", &entry_updated.to_rfc3339())?;
    if let Some(date) = publication {
      xml.element("published", &date.to_rfc3339())?;
    }
    if let Some(description) = &page.description {
      xml.element("summary", description)?;
    }
    for author in &post.authors {
      xml.start("author", &[])?;
      xml.element("name", &author.name)?;
      if let Some(url) = &author.url {
        xml.element("uri", url)?;
      }
      xml.end("author")?;
    }
    for tag in &post.tags {
      xml.empty("category", &[("term", &tag.id), ("label", &tag.label)])?;
    }
    xml.end("entry")?;
  }
  xml.end("feed")?;
  Ok(xml.finish())
}

pub(super) fn open_search(plan: &DocPlan, target: &OpenSearchTarget) -> Result<Vec<u8>> {
  let config = plan
    .site
    .search
    .as_ref()
    .and_then(|search| search.open_search.as_ref())
    .ok_or("[DOCS_SEO_PLAN] OpenSearch configuration missing")?;
  let template = format!(
    "{}?q={{searchTerms}}",
    seo_url(&plan.site, &target.route_path)?
  );
  let self_url = seo_url(
    &plan.site,
    &format!("{}{}", plan.site.base_path, target.output_path),
  )?;
  let mut xml = Xml::new()?;
  xml.start(
    "OpenSearchDescription",
    &[("xmlns", "http://a9.com/-/spec/opensearch/1.1/")],
  )?;
  xml.element("ShortName", &config.short_name)?;
  xml.element("Description", &config.description)?;
  xml.element("InputEncoding", "UTF-8")?;
  xml.element("Language", &target.locale)?;
  xml.empty(
    "Url",
    &[
      ("type", "text/html"),
      ("method", "get"),
      ("template", &template),
    ],
  )?;
  xml.empty(
    "Url",
    &[
      ("type", "application/opensearchdescription+xml"),
      ("rel", "self"),
      ("template", &self_url),
    ],
  )?;
  xml.end("OpenSearchDescription")?;
  Ok(xml.finish())
}
