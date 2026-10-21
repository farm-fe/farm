use crate::{links, routes, DocPlan, PublicPage, PublicSidebarItem, RenderRoute};
use farmfe_docs_content::{ContentReference, ReferenceKind};

fn sidebar_page<'a>(
  items: &[PublicSidebarItem],
  routes: &'a [RenderRoute],
  collection: &str,
) -> Option<&'a PublicPage> {
  items.iter().find_map(|item| match item {
    PublicSidebarItem::Doc { key, .. } => routes
      .iter()
      .find(|route| route.page.key == *key && route.page.collection == collection)
      .map(|route| &route.page),
    PublicSidebarItem::Category { items, .. } => sidebar_page(items, routes, collection),
    PublicSidebarItem::Link { .. } => None,
  })
}

/// The site's genuine root wins; otherwise use a real default collection page.
pub(crate) fn automatic(plan: &DocPlan) -> Result<&PublicPage, String> {
  if let Some(route) = plan.routes.iter().find(|route| {
    route.page.route_path.trim_end_matches('/') == plan.site.base_path.trim_end_matches('/')
  }) {
    return Ok(&route.page);
  }
  let mut collections: Vec<_> = plan.site.collections.iter().collect();
  collections.sort_by_key(|collection| {
    (
      collection.locale != plan.site.locales.default,
      collection
        .version
        .as_deref()
        .unwrap_or(&plan.site.docs.default_version)
        != plan.site.docs.default_version,
    )
  });
  for collection in collections {
    let root = routes::public_path(
      &format!(
        "{}{}",
        plan.site.base_path,
        collection.route_base.trim_start_matches('/')
      ),
      &plan.site,
    );
    if let Some(route) = plan
      .routes
      .iter()
      .find(|route| route.page.collection == collection.id && route.page.route_path == root)
    {
      return Ok(&route.page);
    }
    if let Some(page) = plan
      .sidebars
      .iter()
      .find(|sidebar| sidebar.collection == collection.id)
      .and_then(|sidebar| sidebar_page(&sidebar.items, &plan.routes, &collection.id))
    {
      return Ok(page);
    }
    if let Some(route) = plan
      .routes
      .iter()
      .filter(|route| route.page.collection == collection.id)
      .min_by(|left, right| left.page.route_path.cmp(&right.page.route_path))
    {
      return Ok(&route.page);
    }
  }
  Err("[DOCS_BRAND_HOME] no actual page is available for the brand target".into())
}

/// Select an existing page in this locale, independently of a custom brand URL.
pub(crate) fn localized<'a>(
  plan: &'a DocPlan,
  locale: &str,
  mount: &str,
) -> Result<&'a PublicPage, String> {
  if let Some(route) = plan.routes.iter().find(|route| {
    route.page.locale == locale
      && route.page.route_path.trim_end_matches('/') == mount.trim_end_matches('/')
  }) {
    return Ok(&route.page);
  }
  let mut collections: Vec<_> = plan
    .site
    .collections
    .iter()
    .filter(|collection| collection.locale == locale)
    .collect();
  // Stable sorting retains the user's collection priority, including docs
  // before automatically appended standalone-page collections.
  collections.sort_by_key(|collection| {
    collection
      .version
      .as_deref()
      .unwrap_or(&plan.site.docs.default_version)
      != plan.site.docs.default_version
  });
  for collection in collections {
    let root = routes::public_path(
      &format!(
        "{}{}",
        plan.site.base_path,
        collection.route_base.trim_start_matches('/')
      ),
      &plan.site,
    );
    if let Some(route) = plan
      .routes
      .iter()
      .find(|route| route.page.collection == collection.id && route.page.route_path == root)
    {
      return Ok(&route.page);
    }
    if let Some(page) = plan
      .sidebars
      .iter()
      .find(|sidebar| sidebar.collection == collection.id)
      .and_then(|sidebar| sidebar_page(&sidebar.items, &plan.routes, &collection.id))
    {
      return Ok(page);
    }
    if let Some(route) = plan
      .routes
      .iter()
      .filter(|route| route.page.collection == collection.id)
      .min_by(|left, right| left.page.route_path.cmp(&right.page.route_path))
    {
      return Ok(&route.page);
    }
  }
  if let Some(route) = plan
    .routes
    .iter()
    .filter(|route| route.page.locale == locale)
    .min_by(|left, right| left.page.route_path.cmp(&right.page.route_path))
  {
    return Ok(&route.page);
  }
  automatic(plan)
}

pub(crate) fn resolve_logo(
  plan: &DocPlan,
  index: &links::Index<'_>,
) -> Result<Option<String>, String> {
  let Some(logo) = &plan.site.theme.logo else {
    return Ok(None);
  };
  let home = automatic(plan)?;
  // Same-origin absolute links use the same route validation as local links.
  let local = match (url::Url::parse(&plan.site.url), url::Url::parse(&logo.href)) {
    (Ok(site), Ok(target)) if site.origin() == target.origin() => {
      let mut value = target.path().to_string();
      if let Some(query) = target.query() {
        value.push('?');
        value.push_str(query);
      }
      if let Some(fragment) = target.fragment() {
        value.push('#');
        value.push_str(fragment);
      }
      Some(value)
    }
    _ => None,
  };
  let value = local.as_deref().unwrap_or(&logo.href);
  let end = value.find(['?', '#']).unwrap_or(value.len());
  let path = &value[..end];
  let target = if path == "/"
    || (!path.is_empty() && path.trim_end_matches('/') == plan.site.base_path.trim_end_matches('/'))
  {
    format!("{}{}", home.route_path, &value[end..])
  } else {
    value.to_string()
  };
  let reference = ContentReference {
    kind: ReferenceKind::Link,
    target,
    file: "theme.logo.href".into(),
    line: 1,
    column: 1,
    snippet: logo.href.clone(),
  };
  index
    .resolve(&reference, home)
    .map(Some)
    .map_err(|error| format!("[DOCS_BRAND_HOME] theme.logo.href: {error}"))
}

pub(crate) fn href(plan: &DocPlan) -> Result<&str, String> {
  match &plan.site.theme.logo {
    Some(logo) => Ok(&logo.href),
    None => Ok(&automatic(plan)?.route_path),
  }
}
