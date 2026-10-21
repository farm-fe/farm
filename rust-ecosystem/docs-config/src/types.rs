use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn schema_version() -> u32 {
  crate::SCHEMA_VERSION
}
fn default_locale() -> String {
  "en".into()
}
fn default_locales() -> Vec<String> {
  vec![default_locale()]
}
fn default_version() -> String {
  "current".into()
}
fn root_path() -> String {
  "/".into()
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SiteConfig {
  #[serde(default = "schema_version")]
  #[schemars(range(min = 1, max = 1))]
  pub schema_version: u32,
  pub title: String,
  pub url: String,
  #[serde(default = "root_path")]
  pub base_path: String,
  #[serde(default)]
  pub trailing_slash: TrailingSlash,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub canonical_overrides: Vec<CanonicalOverride>,
  #[serde(default)]
  pub static_dirs: Vec<StaticDirConfig>,
  #[serde(default)]
  pub locales: Locales,
  #[serde(default)]
  pub docs: DocsConfig,
  #[serde(default)]
  pub blog: Option<CollectionGroup>,
  #[serde(default)]
  pub pages: Option<PagesConfig>,
  #[serde(default)]
  pub theme: ThemeConfig,
  #[serde(default)]
  pub search: Option<SearchConfig>,
  #[serde(default)]
  pub ssg: SsgConfig,
  #[serde(default)]
  pub not_found: NotFoundConfig,
  #[serde(default)]
  pub seo: SeoConfig,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub link_exceptions: Vec<LinkException>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FallbackIndexing {
  #[default]
  Noindex,
  Index,
}

#[derive(Clone, Debug, Default, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SeoConfig {
  #[serde(default)]
  pub fallback_indexing: FallbackIndexing,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[schemars(with = "AtomConfig")]
  pub atom: Option<AtomConfig>,
}

impl<'de> Deserialize<'de> for SeoConfig {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    // serde's derived struct visitor also accepts sequences. This public JSON
    // configuration is explicitly an object, matching its generated schema.
    struct SeoVisitor;
    impl<'de> serde::de::Visitor<'de> for SeoVisitor {
      type Value = SeoConfig;
      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an SEO configuration object")
      }
      fn visit_map<M: serde::de::MapAccess<'de>>(
        self,
        mut map: M,
      ) -> Result<Self::Value, M::Error> {
        let mut policy = None;
        let mut atom = None;
        while let Some(key) = map.next_key::<String>()? {
          if key == "atom" {
            if atom.is_some() {
              return Err(serde::de::Error::duplicate_field("atom"));
            }
            atom = Some(map.next_value::<AtomConfig>()?);
            continue;
          }
          if key != "fallbackIndexing" {
            return Err(serde::de::Error::unknown_field(
              &key,
              &["fallbackIndexing", "atom"],
            ));
          }
          if policy.is_some() {
            return Err(serde::de::Error::duplicate_field("fallbackIndexing"));
          }
          policy = Some(map.next_value()?);
        }
        Ok(SeoConfig {
          fallback_indexing: policy.unwrap_or_default(),
          atom,
        })
      }
    }
    deserializer.deserialize_map(SeoVisitor)
  }
}

fn default_ssg_concurrency() -> u32 {
  4
}
fn default_ssg_timeout_ms() -> u32 {
  30_000
}

/// Private build execution limits; never part of PublicSiteConfig.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SsgConfig {
  #[serde(default = "default_ssg_concurrency")]
  #[schemars(range(min = 1, max = 32))]
  pub concurrency: u32,
  #[serde(default = "default_ssg_timeout_ms")]
  #[schemars(range(min = 100, max = 300000))]
  pub timeout_ms: u32,
}
impl Default for SsgConfig {
  fn default() -> Self {
    Self {
      concurrency: default_ssg_concurrency(),
      timeout_ms: default_ssg_timeout_ms(),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema)]
pub enum LinkExceptionCode {
  #[serde(rename = "DOCS_LINK_MISSING")]
  Missing,
  #[serde(rename = "DOCS_LINK_ANCHOR")]
  Anchor,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TrailingSlash {
  #[default]
  Always,
  Never,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CanonicalOverride {
  pub path: String,
  pub canonical_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaticDirConfig {
  pub dir: String,
  #[serde(default = "root_path")]
  pub route_base: String,
}
impl LinkExceptionCode {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Missing => "DOCS_LINK_MISSING",
      Self::Anchor => "DOCS_LINK_ANCHOR",
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LinkException {
  pub collection: String,
  pub id: String,
  pub target: String,
  pub code: LinkExceptionCode,
  pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Locales {
  #[serde(default = "default_locale")]
  pub default: String,
  #[serde(default = "default_locales")]
  pub items: Vec<String>,
}
impl Default for Locales {
  fn default() -> Self {
    Self {
      default: default_locale(),
      items: default_locales(),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DocsConfig {
  #[serde(default = "default_version")]
  pub default_version: String,
  #[serde(default)]
  pub collections: Vec<CollectionConfig>,
}
impl Default for DocsConfig {
  fn default() -> Self {
    Self {
      default_version: default_version(),
      collections: vec![],
    }
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectionGroup {
  #[serde(default)]
  pub collections: Vec<CollectionConfig>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ContentFormat {
  #[default]
  Auto,
  Markdown,
  Mdx,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectionConfig {
  pub id: String,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub exclude: Vec<String>,
  #[serde(default)]
  pub group: Option<String>,
  #[serde(default)]
  pub fallback_locale: Option<String>,
  pub dir: String,
  pub route_base: String,
  #[serde(default)]
  pub locale: Option<String>,
  #[serde(default)]
  pub version: Option<String>,
  #[serde(default)]
  pub format: ContentFormat,
  #[serde(default)]
  pub sidebar: Option<String>,
  #[serde(default)]
  pub blog_labels: Option<BlogLabels>,
  #[serde(default)]
  pub blog_list_content: Option<BlogListContent>,
  #[serde(default)]
  pub admonition_labels: AdmonitionLabels,
}

/// Private collection compilation labels. Explicit author titles take priority.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct AdmonitionLabels {
  pub note: String,
  pub tip: String,
  pub info: String,
  pub warning: String,
  pub danger: String,
  pub caution: String,
  pub important: String,
  pub success: String,
}
impl<'de> Deserialize<'de> for AdmonitionLabels {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    struct LabelsVisitor;
    impl<'de> serde::de::Visitor<'de> for LabelsVisitor {
      type Value = AdmonitionLabels;
      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an admonition label object")
      }
      fn visit_map<M: serde::de::MapAccess<'de>>(
        self,
        mut map: M,
      ) -> Result<Self::Value, M::Error> {
        let mut labels = AdmonitionLabels::default();
        let mut seen = std::collections::HashSet::new();
        while let Some(key) = map.next_key::<String>()? {
          if !seen.insert(key.clone()) {
            return Err(serde::de::Error::custom(format!(
              "duplicate admonition label {key}"
            )));
          }
          let label = match key.as_str() {
            "note" => &mut labels.note,
            "tip" => &mut labels.tip,
            "info" => &mut labels.info,
            "warning" => &mut labels.warning,
            "danger" => &mut labels.danger,
            "caution" => &mut labels.caution,
            "important" => &mut labels.important,
            "success" => &mut labels.success,
            _ => {
              return Err(serde::de::Error::unknown_field(
                &key,
                &[
                  "note",
                  "tip",
                  "info",
                  "warning",
                  "danger",
                  "caution",
                  "important",
                  "success",
                ],
              ));
            }
          };
          *label = map.next_value()?;
        }
        Ok(labels)
      }
    }
    deserializer.deserialize_map(LabelsVisitor)
  }
}
impl Default for AdmonitionLabels {
  fn default() -> Self {
    Self {
      note: "note".into(),
      tip: "tip".into(),
      info: "info".into(),
      warning: "warning".into(),
      danger: "danger".into(),
      caution: "caution".into(),
      important: "important".into(),
      success: "success".into(),
    }
  }
}
impl AdmonitionLabels {
  pub fn to_map(&self) -> std::collections::BTreeMap<String, String> {
    [
      ("note", &self.note),
      ("tip", &self.tip),
      ("info", &self.info),
      ("warning", &self.warning),
      ("danger", &self.danger),
      ("caution", &self.caution),
      ("important", &self.important),
      ("success", &self.success),
    ]
    .into_iter()
    .map(|(kind, label)| (kind.into(), label.clone()))
    .collect()
  }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum BlogListContent {
  #[default]
  Links,
  Full,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct BlogLabels {
  pub title: String,
  pub archive: String,
  pub tags: String,
  pub undated: String,
  pub no_posts: String,
}
impl Default for BlogLabels {
  fn default() -> Self {
    Self {
      title: "Blog".into(),
      archive: "Archive".into(),
      tags: "Tags".into(),
      undated: "Undated".into(),
      no_posts: "No posts.".into(),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PagesConfig {
  pub dir: String,
  #[serde(default)]
  pub format: ContentFormat,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NavLink {
  pub label: String,
  pub to: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LogoConfig {
  pub src: String,
  #[serde(default)]
  pub alt: String,
  #[serde(default = "root_path")]
  pub href: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
  #[default]
  System,
  Light,
  Dark,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FooterConfig {
  #[serde(default)]
  pub links: Vec<NavLink>,
  #[serde(default)]
  pub copyright: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThemeConfig {
  #[serde(default)]
  pub logo: Option<LogoConfig>,
  #[serde(default)]
  pub navbar: Vec<NavLink>,
  #[serde(default)]
  pub footer: FooterConfig,
  #[serde(default)]
  pub color_mode: ColorMode,
  #[serde(default)]
  pub custom_css: Vec<String>,
  #[serde(default)]
  pub components: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SearchProvider {
  None,
  Algolia,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlgoliaOptions {
  pub app_id: String,
  /// Browser search key, part of the explicitly public configuration.
  pub api_key: String,
  pub index_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchConfig {
  pub provider: SearchProvider,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub options: Option<AlgoliaOptions>,
  #[serde(
    default,
    skip_serializing_if = "Option::is_none",
    deserialize_with = "crate::syndication::present"
  )]
  #[schemars(with = "OpenSearchConfig")]
  pub open_search: Option<OpenSearchConfig>,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AtomConfig {
  pub updated: String,
  pub author: String,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OpenSearchConfig {
  pub page_id: String,
  pub short_name: String,
  pub description: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CollectionKind {
  Doc,
  Blog,
  Page,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicCollection {
  pub id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub group: Option<String>,
  pub kind: CollectionKind,
  pub route_base: String,
  pub locale: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub version: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicThemeConfig {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub logo: Option<PublicLogoConfig>,
  pub navbar: Vec<NavLink>,
  pub footer: PublicFooterConfig,
  pub color_mode: ColorMode,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicLocales {
  pub default: String,
  pub items: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotFoundConfig {
  #[serde(default)]
  pub labels: std::collections::BTreeMap<String, NotFoundLabels>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct NotFoundLabels {
  pub title: String,
  pub description: String,
  pub home_label: String,
}

impl Default for NotFoundLabels {
  fn default() -> Self {
    Self {
      title: "Page not found".into(),
      description: "The requested page could not be found.".into(),
      home_label: "Return home".into(),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicLogoConfig {
  pub src: String,
  pub alt: String,
  pub href: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicFooterConfig {
  pub links: Vec<NavLink>,
  pub copyright: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicDocsConfig {
  pub default_version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicSiteConfig {
  #[schemars(range(min = 1, max = 1))]
  pub schema_version: u32,
  pub title: String,
  pub url: String,
  pub base_path: String,
  pub trailing_slash: TrailingSlash,
  pub seo: SeoConfig,
  pub locales: PublicLocales,
  pub not_found: NotFoundConfig,
  pub docs: PublicDocsConfig,
  pub collections: Vec<PublicCollection>,
  pub theme: PublicThemeConfig,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub search: Option<SearchConfig>,
}
