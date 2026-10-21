//! Authoritative configuration protocol shared by native Docs components.
mod exclusions;
mod groups;
mod not_found;
mod sidebar;
mod syndication;
mod types;
mod typescript;
mod validate;

pub use sidebar::*;
pub use syndication::validate as validate_syndication;
pub use types::*;
pub use typescript::{typescript, typescript_with_schemas, TypeGenerationError};
pub use validate::{
  public_link, ConfigError, ResolvedCollection, ResolvedStaticDir, ValidatedConfig,
};

pub const SCHEMA_VERSION: u32 = 1;

/// Framework/tool metadata is never a content or public resource source.
pub fn is_private_tooling_component(component: &str) -> bool {
  [".git", ".farm-docs", ".farm-docs-locks"]
    .iter()
    .any(|private| component.eq_ignore_ascii_case(private))
}

pub fn parse_config(json: &str, root: &std::path::Path) -> Result<ValidatedConfig, ConfigError> {
  let config: SiteConfig = serde_json::from_str(json)
    .map_err(|error| ConfigError::new("DOCS_CONFIG_PARSE", "config", error))?;
  validate::validate(config, root)
}

pub fn schema() -> schemars::schema::RootSchema {
  schemars::schema_for!(SiteConfig)
}

pub fn public_schema() -> schemars::schema::RootSchema {
  schemars::schema_for!(PublicSiteConfig)
}
