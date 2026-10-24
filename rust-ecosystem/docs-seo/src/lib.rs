//! Deterministic SEO resources from an already prepared Rust content plan.
mod syndication;
use chrono::Datelike;
use farmfe_docs_config::CollectionKind;
use farmfe_docs_plan::{seo_url, validate_resource_names, DocPlan, PublicBlogPost, PublicSeoHead};
use quick_xml::{
  events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
  Writer,
};
use std::collections::{BTreeMap, BTreeSet};

type Result<T> = std::result::Result<T, String>;

struct Xml(Writer<Vec<u8>>);
impl Xml {
  fn new() -> Result<Self> {
    let mut xml = Self(Writer::new(Vec::new()));
    xml.write(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    Ok(xml)
  }
  fn write(&mut self, event: Event<'_>) -> Result<()> {
    self
      .0
      .write_event(event)
      .map_err(|error| format!("[DOCS_SEO_XML] {error}"))
  }
  fn start(&mut self, name: &str, attributes: &[(&str, &str)]) -> Result<()> {
    let mut element = BytesStart::new(name);
    for &(key, value) in attributes {
      xml_text(value)?;
      element.push_attribute((key, value));
    }
    self.write(Event::Start(element))
  }
  fn end(&mut self, name: &str) -> Result<()> {
    self.write(Event::End(BytesEnd::new(name)))
  }
  fn element(&mut self, name: &str, value: &str) -> Result<()> {
    xml_text(value)?;
    self.start(name, &[])?;
    self.write(Event::Text(BytesText::new(value)))?;
    self.end(name)
  }
  fn empty(&mut self, name: &str, attributes: &[(&str, &str)]) -> Result<()> {
    let mut element = BytesStart::new(name);
    for &(key, value) in attributes {
      xml_text(value)?;
      element.push_attribute((key, value));
    }
    self.write(Event::Empty(element))
  }
  fn finish(self) -> Vec<u8> {
    self.0.into_inner()
  }
}

fn xml_text(value: &str) -> Result<()> {
  if value.chars().any(|ch| {
    (ch < '\u{20}' && !matches!(ch, '\n' | '\r' | '\t')) || matches!(ch, '\u{FFFE}' | '\u{FFFF}')
  }) {
    return Err("[DOCS_SEO_XML] metadata contains a character forbidden by XML 1.0".into());
  }
  Ok(())
}

fn sitemap(plan: &DocPlan) -> Result<Vec<u8>> {
  let by_path: BTreeMap<_, _> = plan
    .routes
    .iter()
    .map(|route| (route.page.route_path.trim_end_matches('/'), &route.page))
    .collect();
  let mut pages: BTreeMap<&str, &PublicSeoHead> = BTreeMap::new();
  for route in &plan.routes {
    let head = plan
      .seo
      .get(&route.page.key)
      .ok_or("[DOCS_SEO_PLAN] missing page head")?;
    if head.robots != "index,follow" {
      continue;
    }
    let canonical = by_path
      .get(route.page.canonical_path.trim_end_matches('/'))
      .ok_or("[DOCS_SEO_PLAN] absent physical canonical page")?;
    let target = plan
      .seo
      .get(&canonical.key)
      .ok_or("[DOCS_SEO_PLAN] missing canonical head")?;
    if target.robots != "index,follow" {
      continue;
    }
    if target.canonical_url != head.canonical_url {
      return Err("[DOCS_SEO_PLAN] canonical chain differs from physical target".into());
    }
    pages.insert(&head.canonical_url, target);
  }
  let mut xml = Xml::new()?;
  xml.start(
    "urlset",
    &[
      ("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9"),
      ("xmlns:xhtml", "http://www.w3.org/1999/xhtml"),
    ],
  )?;
  for (url, head) in pages {
    xml.start("url", &[])?;
    xml.element("loc", url)?;
    for alternate in &head.alternates {
      xml.empty(
        "xhtml:link",
        &[
          ("rel", "alternate"),
          ("hreflang", &alternate.locale),
          ("href", &alternate.href),
        ],
      )?;
    }
    xml.end("url")?;
  }
  xml.end("urlset")?;
  Ok(xml.finish())
}

fn publication_date(post: &PublicBlogPost) -> Result<Option<String>> {
  if let Some(value) = &post.published_at {
    let date = chrono::DateTime::parse_from_rfc3339(value)
      .map_err(|error| format!("[DOCS_SEO_DATE] {error}"))?;
    let utc = date.with_timezone(&chrono::Utc);
    if (1..=9999).contains(&utc.year()) {
      Ok(Some(utc.to_rfc2822()))
    } else if (1..=9999).contains(&date.year()) {
      // A valid local year can cross the calendar boundary when normalized.
      // Preserve its original offset and instant instead of making chrono's
      // RFC2822 formatter panic or rejecting accepted blog metadata.
      Ok(Some(date.to_rfc2822()))
    } else {
      Err("[DOCS_SEO_DATE] publication year is outside 0001..9999".into())
    }
  } else if let Some(value) = &post.date {
    let date = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
      .map_err(|error| format!("[DOCS_SEO_DATE] {error}"))?;
    if !(1..=9999).contains(&date.year()) {
      return Err("[DOCS_SEO_DATE] publication year is outside 0001..9999".into());
    }
    let date = date
      .and_hms_opt(0, 0, 0)
      .ok_or("[DOCS_SEO_DATE] invalid midnight")?;
    Ok(Some(date.and_utc().to_rfc2822()))
  } else {
    Ok(None)
  }
}

fn rss(
  plan: &DocPlan,
  collection: &farmfe_docs_config::PublicCollection,
  output: &str,
) -> Result<Vec<u8>> {
  let mut xml = Xml::new()?;
  xml.start(
    "rss",
    &[
      ("version", "2.0"),
      ("xmlns:atom", "http://www.w3.org/2005/Atom"),
    ],
  )?;
  xml.start("channel", &[])?;
  xml.element("title", &format!("{} — {}", plan.site.title, collection.id))?;
  xml.element(
    "link",
    &seo_url(
      &plan.site,
      &format!(
        "{}{}",
        plan.site.base_path,
        collection.route_base.trim_start_matches('/')
      ),
    )?,
  )?;
  xml.element("description", &plan.site.title)?;
  xml.element("language", &collection.locale)?;
  xml.empty(
    "atom:link",
    &[
      (
        "href",
        &seo_url(&plan.site, &format!("{}{output}", plan.site.base_path))?,
      ),
      ("rel", "self"),
      ("type", "application/rss+xml"),
    ],
  )?;
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
    xml.start("item", &[])?;
    xml.element("title", &page.title)?;
    xml.element("link", &head.canonical_url)?;
    xml.element("guid", &head.canonical_url)?;
    if let Some(description) = &page.description {
      xml.element("description", description)?;
    }
    if let Some(date) = publication_date(post)? {
      xml.element("pubDate", &date)?;
    }
    for tag in &post.tags {
      xml.element("category", &tag.label)?;
    }
    xml.end("item")?;
  }
  xml.end("channel")?;
  xml.end("rss")?;
  Ok(xml.finish())
}

/// The plan verifier owns content discovery and hashes. This layer only projects
/// verified public metadata into XML/text resources; it never parses body code.
pub fn generate(plan: &DocPlan) -> Result<BTreeMap<String, Vec<u8>>> {
  farmfe_docs_plan::verify(plan)?;
  let mut result = BTreeMap::new();
  result.insert("sitemap.xml".into(), sitemap(plan)?);
  result.insert(
    "robots.txt".into(),
    format!(
      "User-agent: *\nAllow: /\nSitemap: {}\n",
      seo_url(&plan.site, &format!("{}sitemap.xml", plan.site.base_path))?
    )
    .into_bytes(),
  );
  for collection in plan
    .site
    .collections
    .iter()
    .filter(|collection| collection.kind == CollectionKind::Blog)
  {
    let prefix = collection.route_base.trim_matches('/');
    let output = if prefix.is_empty() {
      "rss.xml".into()
    } else {
      format!("{prefix}/rss.xml")
    };
    let bytes = rss(plan, collection, &output)?;
    if result.insert(output.clone(), bytes).is_some() {
      return Err(format!("[DOCS_SEO_COLLISION] {output}"));
    }
    if let Some(config) = &plan.site.seo.atom {
      let output = farmfe_docs_plan::syndication::atom_output(collection);
      let bytes = syndication::atom(plan, collection, &output, config)?;
      if result.insert(output.clone(), bytes).is_some() {
        return Err(format!("[DOCS_SEO_COLLISION] {output}"));
      }
    }
  }
  for target in farmfe_docs_plan::syndication::open_search_targets(plan)? {
    let bytes = syndication::open_search(plan, &target)?;
    if result.insert(target.output_path.clone(), bytes).is_some() {
      return Err(format!("[DOCS_SEO_COLLISION] {}", target.output_path));
    }
  }
  let mut names: Vec<_> = result.keys().cloned().collect();
  names.extend(plan.routes.iter().map(|route| route.output_path.clone()));
  names.extend(
    plan
      .static_files
      .iter()
      .map(|file| file.output_path.clone()),
  );
  names.extend(
    plan
      .assets
      .iter()
      .filter(|file| !file.is_static)
      .map(|file| file.output_path.clone()),
  );
  names.extend(
    [
      "docs-plan.json",
      "docs-public-config.json",
      "docs-manifest.json",
      "docs-build-report.json",
    ]
    .map(String::from),
  );
  validate_resource_names(&names)?;
  Ok(result)
}
