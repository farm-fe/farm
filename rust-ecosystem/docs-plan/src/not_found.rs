use crate::{DocPlan, NotFoundPlan};
use farmfe_docs_config::PublicSiteConfig;
pub(crate) const ROBOTS: &str = "noindex,follow";

/// An auxiliary output, never a content route or a fabricated PublicPage.
pub(crate) fn outputs(site: &PublicSiteConfig) -> Vec<(String, String, String)> {
  let mut locales = site.locales.items.clone();
  locales.sort_by_key(|locale| (locale != &site.locales.default, locale.clone()));
  locales
    .into_iter()
    .map(|locale| {
      let (output, mount) = if locale == site.locales.default {
        ("404.html".into(), site.base_path.clone())
      } else {
        (
          format!("{locale}/404.html"),
          format!("{}{locale}/", site.base_path),
        )
      };
      (locale, output, mount)
    })
    .collect()
}

pub(crate) fn plan(prepared: &DocPlan) -> Result<Vec<NotFoundPlan>, String> {
  let site = &prepared.site;
  outputs(site)
    .into_iter()
    .map(|(locale, output_path, route_base)| {
      crate::routes::validate_output(&output_path, &format!("notFound.{locale}"))?;
      let labels = site
        .not_found
        .labels
        .get(&locale)
        .ok_or_else(|| format!("[DOCS_PLAN_NOT_FOUND] missing normalized labels for {locale}"))?;
      let discovery_links = crate::syndication::discovery_links(prepared, &locale)?;
      Ok(NotFoundPlan {
        home: crate::brand_home::localized(prepared, &locale, &route_base)?
          .route_path
          .clone(),
        output_path,
        locale,
        route_base,
        heading: labels.title.clone(),
        home_label: labels.home_label.clone(),
        title: format!("{} | {}", labels.title, site.title),
        description: labels.description.clone(),
        head_html: format!(
          "<meta name=\"robots\" content=\"{ROBOTS}\">{}",
          crate::syndication::discovery_html(&discovery_links)
        ),
        discovery_links,
      })
    })
    .collect()
}
