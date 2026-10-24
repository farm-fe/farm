use crate::*;
use std::{
  collections::HashSet,
  fmt,
  path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct ConfigError {
  pub code: &'static str,
  pub field: String,
  pub message: String,
}
impl ConfigError {
  pub(crate) fn new(code: &'static str, field: &str, message: impl fmt::Display) -> Self {
    Self {
      code,
      field: field.into(),
      message: message.to_string(),
    }
  }
}
impl fmt::Display for ConfigError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      formatter,
      "[{}] {}: {}",
      self.code, self.field, self.message
    )
  }
}
impl std::error::Error for ConfigError {}

/// Private compiler inputs deliberately do not implement Serialize.
pub struct ResolvedCollection {
  pub public: PublicCollection,
  pub directory: PathBuf,
  pub excluded_files: HashSet<PathBuf>,
  pub sidebar: Option<PathBuf>,
  pub format: ContentFormat,
  pub fallback_locale: Option<String>,
  pub blog_labels: BlogLabels,
  pub blog_list_content: BlogListContent,
  pub admonition_labels: std::collections::BTreeMap<String, String>,
}
pub struct ValidatedConfig {
  pub site_root: PathBuf,
  pub public: PublicSiteConfig,
  pub collections: Vec<ResolvedCollection>,
  pub pages_directory: Option<PathBuf>,
  pub custom_css: Vec<PathBuf>,
  pub theme_components: Option<PathBuf>,
  pub link_exceptions: Vec<LinkException>,
  pub canonical_overrides: Vec<CanonicalOverride>,
  pub static_dirs: Vec<ResolvedStaticDir>,
  pub ssg: SsgConfig,
  pub build: BuildConfig,
  pub code: farmfe_syntax_highlight::CodeThemeConfig,
}

impl SsgConfig {
  pub fn validate(&self) -> Result<(), ConfigError> {
    for (field, value, min, max) in [
      ("ssg.concurrency", self.concurrency, 1, 32),
      ("ssg.timeoutMs", self.timeout_ms, 100, 300_000),
    ] {
      if !(min..=max).contains(&value) {
        return Err(ConfigError::new(
          "DOCS_CONFIG_SSG",
          field,
          format!("require an integer in {min}..={max}"),
        ));
      }
    }
    Ok(())
  }
}
pub struct ResolvedStaticDir {
  pub directory: PathBuf,
  pub route_base: String,
}

fn nonempty(value: &str, field: &str) -> Result<(), ConfigError> {
  if value.trim().is_empty() {
    return Err(ConfigError::new(
      "DOCS_CONFIG_VALUE",
      field,
      "must not be empty",
    ));
  }
  Ok(())
}
fn tag(value: &str) -> bool {
  !value.is_empty()
    && !value.contains("..")
    && value
      .chars()
      .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
}

fn web_url(value: &str, field: &str, origin_only: bool) -> Result<url::Url, ConfigError> {
  let invalid = |message| ConfigError::new("DOCS_CONFIG_URL", field, message);
  let url = url::Url::parse(value).map_err(|_| invalid("require an absolute http(s) URL"))?;
  if !matches!(url.scheme(), "http" | "https")
    || url.host_str().is_none()
    || !url.username().is_empty()
    || url.password().is_some()
  {
    return Err(invalid("require an http(s) URL without credentials"));
  }
  if origin_only && (url.path() != "/" || url.query().is_some() || url.fragment().is_some()) {
    return Err(invalid(
      "site URL must be an origin; use basePath for a path prefix",
    ));
  }
  Ok(url)
}

fn url_path(value: &str, field: &str) -> Result<String, ConfigError> {
  if !value.starts_with('/')
    || value.starts_with("//")
    || value.contains(['\\', '?', '#', '%'])
    || value.chars().any(char::is_whitespace)
    || value.chars().any(char::is_control)
    || value.split('/').any(|part| part == "." || part == "..")
    || value.contains("//")
  {
    return Err(ConfigError::new(
      "DOCS_CONFIG_PATH",
      field,
      "require an unescaped absolute URL path without traversal, query or fragment",
    ));
  }
  Ok(if value.ends_with('/') {
    value.into()
  } else {
    format!("{value}/")
  })
}

pub fn public_link(value: &str, field: &str) -> Result<(), ConfigError> {
  if value.chars().any(char::is_control) {
    return Err(ConfigError::new(
      "DOCS_CONFIG_URL",
      field,
      "URL must not contain control characters",
    ));
  }
  if value.starts_with('/') {
    let path_end = value.find(['?', '#']).unwrap_or(value.len());
    url_path(&value[..path_end], field)?;
  } else if value.starts_with('#') {
    if value.len() < 2 || value.chars().any(char::is_control) {
      return Err(ConfigError::new(
        "DOCS_CONFIG_URL",
        field,
        "require a nonempty fragment",
      ));
    }
  } else {
    web_url(value, field, false)?;
  }
  Ok(())
}

fn input_path(
  root: &Path,
  value: &str,
  field: &str,
  directory: bool,
) -> Result<PathBuf, ConfigError> {
  let relative = value.replace('\\', "/");
  let invalid = |message| ConfigError::new("DOCS_CONFIG_PATH", field, message);
  if relative.trim().is_empty()
    || relative.starts_with('/')
    || relative.contains(':')
    || relative.split('/').any(|part| part == "..")
  {
    return Err(invalid("require a relative input inside the site root"));
  }
  if relative.split('/').any(is_private_tooling_component) {
    return Err(invalid(
      "private tooling directories cannot be source inputs",
    ));
  }
  let resolved = root
    .join(&relative)
    .canonicalize()
    .map_err(|error| ConfigError::new("DOCS_CONFIG_PATH", field, error))?;
  if !resolved.starts_with(root)
    || (directory && !resolved.is_dir())
    || (!directory && !resolved.is_file())
  {
    return Err(invalid(
      "input escapes the site root or has the wrong file type",
    ));
  }
  if resolved.strip_prefix(root).is_ok_and(|relative| {
    relative
      .components()
      .any(|part| is_private_tooling_component(&part.as_os_str().to_string_lossy()))
  }) {
    return Err(invalid(
      "private tooling directories cannot be source inputs",
    ));
  }
  Ok(resolved)
}

pub(crate) fn validate(config: SiteConfig, root: &Path) -> Result<ValidatedConfig, ConfigError> {
  crate::syndication::validate(&config.seo, config.search.as_ref())?;
  if config.schema_version != SCHEMA_VERSION {
    return Err(ConfigError::new(
      "DOCS_CONFIG_VERSION",
      "schemaVersion",
      "unsupported version; expected 1",
    ));
  }
  config.ssg.validate()?;
  nonempty(&config.title, "title")?;
  let url = web_url(&config.url, "url", true)?
    .origin()
    .ascii_serialization();
  let base_path = url_path(&config.base_path, "basePath")?;
  for (index, rule) in config.canonical_overrides.iter().enumerate() {
    for (property, value) in [
      ("path", &rule.path),
      ("canonicalPath", &rule.canonical_path),
    ] {
      let field = format!("canonicalOverrides[{index}].{property}");
      url_path(value, &field)
        .map_err(|error| ConfigError::new("DOCS_CANONICAL_PATH", &field, error))?;
      if value != base_path.trim_end_matches('/') && !value.starts_with(&base_path) {
        return Err(ConfigError::new(
          "DOCS_CANONICAL_PATH",
          &field,
          "must stay within basePath",
        ));
      }
    }
  }
  let root = root
    .canonicalize()
    .map_err(|error| ConfigError::new("DOCS_CONFIG_PATH", "root", error))?;
  if !root.is_dir() {
    return Err(ConfigError::new(
      "DOCS_CONFIG_PATH",
      "root",
      "require a directory",
    ));
  }
  let mut static_dirs: Vec<ResolvedStaticDir> = vec![];
  for (index, source) in config.static_dirs.iter().enumerate() {
    let field = format!("staticDirs[{index}]");
    if source
      .dir
      .replace('\\', "/")
      .split('/')
      .any(is_private_tooling_component)
    {
      return Err(ConfigError::new(
        "DOCS_STATIC_CONFIG",
        &field,
        "private tooling directories cannot be published",
      ));
    }
    let directory = input_path(&root, &source.dir, &format!("{field}.dir"), true)?;
    if directory == root
      || static_dirs.iter().any(|other| {
        directory.starts_with(&other.directory) || other.directory.starts_with(&directory)
      })
    {
      return Err(ConfigError::new(
        "DOCS_STATIC_CONFIG",
        &field,
        "static directories must be dedicated, non-overlapping site directories",
      ));
    }
    let mut cursor = root.clone();
    for part in source
      .dir
      .replace('\\', "/")
      .split('/')
      .filter(|part| !part.is_empty() && *part != ".")
    {
      if is_private_tooling_component(part) {
        return Err(ConfigError::new(
          "DOCS_STATIC_CONFIG",
          &field,
          "private tooling directories cannot be published",
        ));
      }
      cursor.push(part);
      let metadata = std::fs::symlink_metadata(&cursor)
        .map_err(|cause| ConfigError::new("DOCS_STATIC_CONFIG", &field, cause))?;
      let reparse = metadata.file_type().is_symlink();
      #[cfg(windows)]
      let reparse = {
        use std::os::windows::fs::MetadataExt;
        reparse || metadata.file_attributes() & 0x400 != 0
      };
      if reparse {
        return Err(ConfigError::new(
          "DOCS_STATIC_CONFIG",
          &field,
          "symlinks and junctions cannot be static sources",
        ));
      }
    }
    static_dirs.push(ResolvedStaticDir {
      directory,
      route_base: url_path(&source.route_base, &format!("{field}.routeBase"))?,
    });
  }
  let mut locales = HashSet::new();
  for locale in &config.locales.items {
    if !tag(locale) || locale.contains(['_', '.']) || !locales.insert(locale.clone()) {
      return Err(ConfigError::new(
        "DOCS_CONFIG_LOCALE",
        "locales.items",
        "require unique locale tags",
      ));
    }
  }
  if !locales.contains(&config.locales.default) {
    return Err(ConfigError::new(
      "DOCS_CONFIG_LOCALE",
      "locales.default",
      "must be declared in locales.items",
    ));
  }
  if !tag(&config.docs.default_version) {
    return Err(ConfigError::new(
      "DOCS_CONFIG_VERSION",
      "docs.defaultVersion",
      "require a version tag",
    ));
  }
  let not_found = crate::not_found::resolve(&config.not_found, &config.locales.items)?;
  if let Some(ai) = &config.ai {
    if let Some(guide) = &ai.agent_guide_url {
      public_link(guide, "ai.agentGuideUrl")?;
    }
  }
  let mut ids = HashSet::new();
  let mut collections = Vec::new();
  let mut has_default_version = config.docs.collections.is_empty();
  for (section, group, kind) in [
    (
      "docs",
      config.docs.collections.as_slice(),
      CollectionKind::Doc,
    ),
    (
      "blog",
      config
        .blog
        .as_ref()
        .map(|blog| blog.collections.as_slice())
        .unwrap_or_default(),
      CollectionKind::Blog,
    ),
  ] {
    for (index, collection) in group.iter().enumerate() {
      let field = format!("{section}.collections[{index}]");
      if kind != CollectionKind::Blog && collection.blog_list_content.is_some() {
        return Err(ConfigError::new(
          "DOCS_CONFIG_BLOG_CONTENT",
          &format!("{field}.blogListContent"),
          "blog list content requires a blog collection",
        ));
      }
      if kind != CollectionKind::Blog && collection.blog_labels.is_some() {
        return Err(ConfigError::new(
          "DOCS_CONFIG_BLOG_LABELS",
          &format!("{field}.blogLabels"),
          "blog labels require a blog collection",
        ));
      }
      let blog_labels = collection.blog_labels.clone().unwrap_or_default();
      for (name, label) in [
        ("title", &blog_labels.title),
        ("archive", &blog_labels.archive),
        ("tags", &blog_labels.tags),
        ("undated", &blog_labels.undated),
        ("noPosts", &blog_labels.no_posts),
      ] {
        if label.trim().is_empty() || label.chars().any(char::is_control) {
          return Err(ConfigError::new(
            "DOCS_CONFIG_BLOG_LABELS",
            &format!("{field}.blogLabels.{name}"),
            "require nonempty text without control characters",
          ));
        }
      }
      if kind == CollectionKind::Blog && collection.sidebar.is_some() {
        return Err(ConfigError::new(
          "DOCS_CONFIG_SIDEBAR",
          &format!("{field}.sidebar"),
          "blog collections use date/tag/archive indexes, not document sidebars",
        ));
      }
      if !tag(&collection.id) || !ids.insert(collection.id.clone()) {
        return Err(ConfigError::new(
          "DOCS_CONFIG_COLLECTION",
          &format!("{field}.id"),
          "require a unique collection identifier",
        ));
      }
      let locale = collection
        .locale
        .clone()
        .unwrap_or_else(|| config.locales.default.clone());
      if !locales.contains(&locale) {
        return Err(ConfigError::new(
          "DOCS_CONFIG_LOCALE",
          &format!("{field}.locale"),
          "locale is not declared",
        ));
      }
      let version = if matches!(kind, CollectionKind::Doc) {
        let version = collection
          .version
          .clone()
          .unwrap_or_else(|| config.docs.default_version.clone());
        if !tag(&version) {
          return Err(ConfigError::new(
            "DOCS_CONFIG_VERSION",
            &format!("{field}.version"),
            "require a version tag",
          ));
        }
        has_default_version |= version == config.docs.default_version;
        Some(version)
      } else {
        if collection.version.is_some() {
          return Err(ConfigError::new(
            "DOCS_CONFIG_VERSION",
            &format!("{field}.version"),
            "blog collections do not have versions",
          ));
        }
        None
      };
      let public = PublicCollection {
        id: collection.id.clone(),
        group: collection.group.clone(),
        kind,
        locale,
        version,
        route_base: url_path(&collection.route_base, &format!("{field}.routeBase"))?,
      };
      if collection.group.as_ref().is_some_and(|group| !tag(group)) {
        return Err(ConfigError::new(
          "DOCS_CONFIG_GROUP",
          &format!("{field}.group"),
          "require a group identifier",
        ));
      }
      let directory = input_path(&root, &collection.dir, &format!("{field}.dir"), true)?;
      let excluded_files = crate::exclusions::resolve(&directory, &collection.exclude, &field)?;
      collections.push(ResolvedCollection {
        public,
        directory,
        excluded_files,
        sidebar: collection
          .sidebar
          .as_deref()
          .map(|path| input_path(&root, path, &format!("{field}.sidebar"), false))
          .transpose()?,
        format: collection.format,
        fallback_locale: collection.fallback_locale.clone(),
        blog_labels,
        blog_list_content: collection.blog_list_content.unwrap_or_default(),
        admonition_labels: collection.admonition_labels.to_map(),
      });
    }
  }
  let pages_directory = config
    .pages
    .as_ref()
    .map(|pages| input_path(&root, &pages.dir, "pages.dir", true))
    .transpose()?;
  if let Some(directory) = &pages_directory {
    for locale in &config.locales.items {
      let id = format!("@pages:{locale}");
      ids.insert(id.clone());
      collections.push(ResolvedCollection {
        public: PublicCollection {
          id,
          group: Some("@pages".into()),
          kind: CollectionKind::Page,
          route_base: if locale == &config.locales.default {
            "/".into()
          } else {
            format!("/{locale}/")
          },
          locale: locale.clone(),
          version: None,
        },
        directory: directory.clone(),
        excluded_files: HashSet::new(),
        sidebar: None,
        format: config
          .pages
          .as_ref()
          .map(|pages| pages.format)
          .unwrap_or_default(),
        fallback_locale: None,
        blog_labels: BlogLabels::default(),
        blog_list_content: BlogListContent::default(),
        admonition_labels: AdmonitionLabels::default().to_map(),
      });
    }
  }
  crate::groups::validate(&collections)?;
  let mut exception_keys = HashSet::new();
  for (index, exception) in config.link_exceptions.iter().enumerate() {
    let field = format!("linkExceptions[{index}]");
    for (name, value) in [
      ("id", &exception.id),
      ("target", &exception.target),
      ("reason", &exception.reason),
    ] {
      if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(ConfigError::new(
          "DOCS_CONFIG_LINK_EXCEPTION",
          &format!("{field}.{name}"),
          "require nonempty text without control characters",
        ));
      }
    }
    if !ids.contains(&exception.collection) {
      return Err(ConfigError::new(
        "DOCS_CONFIG_LINK_EXCEPTION",
        &format!("{field}.collection"),
        "collection is not configured",
      ));
    }
    if !exception_keys.insert((&exception.collection, &exception.id, &exception.target)) {
      return Err(ConfigError::new(
        "DOCS_CONFIG_LINK_EXCEPTION",
        &field,
        "duplicate collection/id/target; one exact error code is permitted",
      ));
    }
  }
  if !has_default_version {
    return Err(ConfigError::new(
      "DOCS_CONFIG_VERSION",
      "docs.defaultVersion",
      "no document collection provides this version",
    ));
  }
  if collections.is_empty() && pages_directory.is_none() {
    return Err(ConfigError::new(
      "DOCS_CONFIG_COLLECTION",
      "docs.collections",
      "require at least one collection or pages directory",
    ));
  }
  for (section, links) in [
    ("theme.navbar", &config.theme.navbar),
    ("theme.footer.links", &config.theme.footer.links),
  ] {
    for (index, link) in links.iter().enumerate() {
      nonempty(&link.label, &format!("{section}[{index}].label"))?;
      public_link(&link.to, &format!("{section}[{index}].to"))?;
    }
  }
  if let Some(logo) = &config.theme.logo {
    public_link(&logo.src, "theme.logo.src")?;
    public_link(&logo.href, "theme.logo.href")?;
  }
  if let Some(search) = &config.search {
    match (&search.provider, &search.options) {
      (SearchProvider::None, None) => {}
      (SearchProvider::Algolia, Some(options)) => {
        for (field, value) in [
          ("appId", &options.app_id),
          ("apiKey", &options.api_key),
          ("indexName", &options.index_name),
        ] {
          nonempty(value, &format!("search.options.{field}"))?;
        }
      }
      _ => {
        return Err(ConfigError::new(
          "DOCS_CONFIG_SEARCH",
          "search.options",
          "options must match the selected provider",
        ));
      }
    }
  }
  let custom_css = config
    .theme
    .custom_css
    .iter()
    .enumerate()
    .map(|(index, path)| input_path(&root, path, &format!("theme.customCss[{index}]"), false))
    .collect::<Result<_, _>>()?;
  let theme_components = config
    .theme
    .components
    .as_deref()
    .map(|path| input_path(&root, path, "theme.components", false))
    .transpose()?;
  let public = PublicSiteConfig {
    ai: config.ai,
    schema_version: SCHEMA_VERSION,
    title: config.title,
    url,
    base_path,
    trailing_slash: config.trailing_slash,
    locales: PublicLocales {
      default: config.locales.default,
      items: config.locales.items,
    },
    not_found,
    docs: PublicDocsConfig {
      default_version: config.docs.default_version,
    },
    collections: collections
      .iter()
      .map(|collection| collection.public.clone())
      .collect(),
    theme: PublicThemeConfig {
      logo: config.theme.logo.map(|logo| PublicLogoConfig {
        src: logo.src,
        alt: logo.alt,
        href: logo.href,
      }),
      navbar: config.theme.navbar,
      footer: PublicFooterConfig {
        links: config.theme.footer.links,
        copyright: config.theme.footer.copyright,
      },
      color_mode: config.theme.color_mode,
    },
    search: config.search,
    seo: config.seo,
  };
  Ok(ValidatedConfig {
    site_root: root,
    public,
    collections,
    pages_directory,
    custom_css,
    theme_components,
    link_exceptions: config.link_exceptions,
    canonical_overrides: config.canonical_overrides,
    static_dirs,
    ssg: config.ssg,
    build: config.build,
    code: config.code,
  })
}
