use farmfe_docs_config::PublicSiteConfig;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicHeading {
  pub id: String,
  pub title: String,
  pub depth: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicPage {
  pub key: String,
  pub id: String,
  pub collection: String,
  pub kind: String,
  pub locale: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub version: Option<String>,
  pub route_path: String,
  pub canonical_path: String,
  pub title: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  pub headings: Vec<PublicHeading>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub alternates: Vec<PageAlternate>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fallback_source: Option<PageSource>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub previous: Option<PageNavigation>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub next: Option<PageNavigation>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub blog: Option<PublicBlogPost>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub blog_view: Option<PublicBlogView>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub layout: Option<PageLayout>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PageLayout {
  Default,
  None,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PageMetadata {
  pub id: Option<String>,
  pub title: Option<String>,
  #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
  pub title_by_locale: std::collections::BTreeMap<String, String>,
  pub description: Option<String>,
  pub slug: Option<String>,
  pub layout: Option<PageLayout>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ContentLanguage {
  #[default]
  Markdown,
  Jsx,
  Ts,
  Tsx,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicBlogView {
  pub kind: String,
  pub posts: Vec<String>,
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  pub full_content: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tag: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicBlogAuthor {
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub image: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicBlogTag {
  pub id: String,
  pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicBlogPost {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub date: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub published_at: Option<String>,
  pub authors: Vec<PublicBlogAuthor>,
  pub tags: Vec<PublicBlogTag>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicBlogTagIndex {
  pub id: String,
  pub label: String,
  pub posts: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicBlogYearIndex {
  pub year: String,
  pub posts: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicBlogIndex {
  pub collection: String,
  pub posts: Vec<String>,
  pub tags: Vec<PublicBlogTagIndex>,
  pub years: Vec<PublicBlogYearIndex>,
  pub undated: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PageNavigation {
  pub key: String,
  pub id: String,
  pub label: String,
  pub href: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum PublicSidebarItem {
  Doc {
    key: String,
    id: String,
    label: String,
    href: String,
  },
  Category {
    label: String,
    items: Vec<PublicSidebarItem>,
  },
  Link {
    label: String,
    href: String,
  },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicSidebar {
  pub collection: String,
  pub items: Vec<PublicSidebarItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanInput {
  pub path: String,
  pub hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PageSource {
  pub key: String,
  pub id: String,
  pub collection: String,
  pub locale: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub version: Option<String>,
  pub route_path: String,
}
impl From<&PublicPage> for PageSource {
  fn from(page: &PublicPage) -> Self {
    Self {
      key: page.key.clone(),
      id: page.id.clone(),
      collection: page.collection.clone(),
      locale: page.locale.clone(),
      version: page.version.clone(),
      route_path: page.route_path.clone(),
    }
  }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PageAlternate {
  pub collection: String,
  pub locale: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub version: Option<String>,
  pub route_path: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub key: Option<String>,
  pub missing: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RenderRoute {
  pub page: PublicPage,
  pub output_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreparedModule {
  pub key: String,
  pub source_path: String,
  pub source_hash: String,
  pub code: String,
  pub source_map: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub generated_source: Option<String>,
  #[serde(default)]
  pub language: ContentLanguage,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub static_anchors: Vec<String>,
}
/// Private build artifact. It is never emitted by browser/server child builds.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DocPlan {
  pub schema_version: u32,
  pub snapshot_id: String,
  pub ssg: farmfe_docs_config::SsgConfig,
  pub site: PublicSiteConfig,
  pub seo: std::collections::BTreeMap<String, PublicSeoHead>,
  pub not_found: Vec<NotFoundPlan>,
  pub routes: Vec<RenderRoute>,
  pub modules: Vec<PreparedModule>,
  pub styles: Vec<String>,
  pub theme_components: Option<String>,
  pub diagnostics: Vec<String>,
  /// Safe projection for the published build report; private diagnostics stay local.
  pub public_diagnostics: Vec<String>,
  pub sidebars: Vec<PublicSidebar>,
  pub inputs: Vec<PlanInput>,
  pub content_inputs: Vec<ContentDirectoryInput>,
  #[serde(default)]
  pub assets: Vec<PreparedAsset>,
  #[serde(default)]
  pub static_files: Vec<PreparedAsset>,
  #[serde(default)]
  pub static_inputs: Vec<StaticDirectoryInput>,
  pub blog_indexes: Vec<PublicBlogIndex>,
}

/// Required auxiliary rendering metadata, separate from content identities.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotFoundPlan {
  pub output_path: String,
  pub locale: String,
  pub route_base: String,
  pub home: String,
  pub heading: String,
  pub home_label: String,
  pub title: String,
  pub description: String,
  pub head_html: String,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub discovery_links: Vec<PublicDiscoveryLink>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicSeoHead {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub title: Option<String>,
  pub canonical_url: String,
  pub robots: String,
  pub alternates: Vec<SeoAlternate>,
  pub html: String,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub discovery_links: Vec<PublicDiscoveryLink>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicDiscoveryLink {
  pub rel: String,
  #[serde(rename = "type")]
  pub resource_type: String,
  pub title: String,
  pub href: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SeoAlternate {
  pub locale: String,
  pub href: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreparedAsset {
  pub key: String,
  pub source_path: String,
  pub source_hash: String,
  pub output_path: String,
  #[serde(default)]
  pub is_static: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaticDirectoryInput {
  pub path: String,
  pub entries_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentDirectoryInput {
  pub path: String,
  pub scripts: bool,
  pub entries_hash: String,
}
