//! Prepared content plans shared by the native Docs plugin and its workflow.
mod application;
mod assets;
mod blog;
mod blog_views;
mod brand_home;
mod content_cache;
mod link_exceptions;
mod links;
mod metadata;
mod not_found;
mod prepare;
pub mod resource_owner;
mod routes;
mod script_pages;
mod seo;
mod sidebars;
mod static_files;
pub mod syndication;
mod translations;
mod types;
pub use application::{application, browser_application};
pub use content_cache::ContentCache;
pub use prepare::{prepare, prepare_with_cache};
pub use seo::seo_url;
use sha2::{Digest, Sha256};
pub use types::*;

/// Private Rust context record, never a browser resource or public configuration.
pub const DEVELOPMENT_PLAN_CONTEXT_KEY: &str = "farm-docs:development-plan:v2";

/// Resolve a root URL against the immutable, Rust-scanned static namespace.
pub fn static_resource_url(
  plan: &DocPlan,
  target: &str,
  file: &str,
  line: usize,
  column: usize,
) -> Result<String, String> {
  let reference = farmfe_docs_content::ContentReference {
    kind: farmfe_docs_content::ReferenceKind::Image,
    target: target.into(),
    file: file.into(),
    line,
    column,
    snippet: target.into(),
  };
  let asset =
    assets::Registry::with_static(&plan.static_files).static_file(&plan.site, &reference)?;
  let end = target.find(['?', '#']).unwrap_or(target.len());
  Ok(format!(
    "{}{}",
    asset_url(&plan.site, &asset),
    &target[end..]
  ))
}

pub(crate) fn hash(bytes: &[u8]) -> String {
  format!("{:x}", Sha256::digest(bytes))
}
pub(crate) fn identity(plan: &DocPlan) -> Result<String, String> {
  let mut data = plan.clone();
  data.snapshot_id.clear();
  serde_json::to_vec(&data)
    .map(|bytes| hash(&bytes))
    .map_err(|error| error.to_string())
}
pub fn verify(plan: &DocPlan) -> Result<(), String> {
  syndication::validate(plan)?;
  plan
    .ssg
    .validate()
    .map_err(|error| format!("[DOCS_PLAN_SSG] {error}"))?;
  if plan.schema_version != 2 || plan.snapshot_id != identity(plan)? {
    return Err("[DOCS_PLAN_HASH] invalid schema or prepared plan digest".into());
  }
  if plan.seo != seo::heads(plan)? {
    return Err("[DOCS_PLAN_SEO] SEO registry differs from verified page metadata".into());
  }
  if plan.not_found != not_found::plan(plan)? {
    return Err("[DOCS_PLAN_NOT_FOUND] NotFound metadata differs from the Rust output plan".into());
  }
  let mut outputs: Vec<String> = plan
    .routes
    .iter()
    .map(|route| route.output_path.clone())
    .collect();
  outputs.extend(static_files::reserved(&plan.site));
  outputs.extend(
    plan
      .static_files
      .iter()
      .map(|file| file.output_path.clone()),
  );
  outputs.extend(
    plan
      .assets
      .iter()
      .filter(|file| !file.is_static)
      .map(|file| file.output_path.clone()),
  );
  static_files::validate_names(&outputs)?;
  if plan.routes.is_empty() || plan.routes.len() != plan.modules.len() {
    return Err("[DOCS_PLAN_SHAPE] routes and compiled modules must correspond".into());
  }
  for directory in &plan.content_inputs {
    let (entries_hash, _) =
      prepare::content_listing(std::path::Path::new(&directory.path), directory.scripts)
        .map_err(|error| format!("[DOCS_SOURCE_SNAPSHOT] {}: {error}", directory.path))?;
    if entries_hash != directory.entries_hash {
      return Err(format!(
        "[DOCS_SOURCE_SNAPSHOT] {}: content directory entries changed",
        directory.path
      ));
    }
  }
  for (route, module) in plan.routes.iter().zip(&plan.modules) {
    if route.page.key != module.key {
      return Err("[DOCS_PLAN_SHAPE] module identity differs from page".into());
    }
    if let Some(source) = &module.generated_source {
      if module.source_path != format!("farm-docs:generated:{}", module.key)
        || module.language != ContentLanguage::Markdown
        || !route.page.kind.starts_with("blog-")
        || !route
          .page
          .blog_view
          .as_ref()
          .is_some_and(|view| view.kind == route.page.kind)
        || hash(source.as_bytes()) != module.source_hash
      {
        return Err("[DOCS_PLAN_SOURCE] invalid generated source identity or digest".into());
      }
      continue;
    }
    if module.source_path.starts_with("farm-docs:generated:") {
      return Err("[DOCS_PLAN_SOURCE] generated source is missing".into());
    }
    let bytes = std::fs::read(&module.source_path)
      .map_err(|error| format!("[DOCS_PLAN_SOURCE] {}: {error}", module.source_path))?;
    if hash(&bytes) != module.source_hash {
      return Err(format!("[DOCS_SNAPSHOT_CHANGED] {}", module.source_path));
    }
  }
  for directory in &plan.static_inputs {
    let (entries_hash, _) = static_files::listing(&directory.path)?;
    if entries_hash != directory.entries_hash {
      return Err(format!(
        "[DOCS_SNAPSHOT_CHANGED] {}: static directory entries changed",
        directory.path
      ));
    }
  }
  for input in &plan.inputs {
    let bytes = std::fs::read(&input.path)
      .map_err(|error| format!("[DOCS_PLAN_SOURCE] {}: {error}", input.path))?;
    if hash(&bytes) != input.hash {
      return Err(format!("[DOCS_SNAPSHOT_CHANGED] {}", input.path));
    }
  }
  for asset in &plan.assets {
    routes::validate_output(&asset.output_path, &asset.source_path)?;
    if !asset.is_static
      && (!asset
        .output_path
        .starts_with(&format!("assets/docs/{}.", asset.key))
        || asset.key.len() != 64
        || !asset.key.bytes().all(|ch| ch.is_ascii_hexdigit()))
    {
      return Err("[DOCS_ASSET_PLAN] invalid asset identity".into());
    }
    if asset.is_static
      && !plan.static_files.iter().any(|file| {
        file.key == asset.key
          && file.output_path == asset.output_path
          && file.source_path == asset.source_path
          && file.source_hash == asset.source_hash
      })
    {
      return Err("[DOCS_STATIC_PLAN] bound static file is absent".into());
    }
    if !plan
      .inputs
      .iter()
      .any(|input| input.path == asset.source_path && input.hash == asset.source_hash)
    {
      return Err("[DOCS_ASSET_PLAN] asset is absent from the verified input snapshot".into());
    }
  }
  for file in &plan.static_files {
    routes::validate_output(&file.output_path, &file.source_path)?;
    if !file.is_static
      || !plan
        .inputs
        .iter()
        .any(|input| input.path == file.source_path && input.hash == file.source_hash)
    {
      return Err("[DOCS_STATIC_PLAN] invalid static source snapshot".into());
    }
  }
  Ok(())
}
pub fn asset_url(site: &farmfe_docs_config::PublicSiteConfig, asset: &PreparedAsset) -> String {
  static_files::public_url(site, &asset.output_path)
}
pub fn validate_resource_names(names: &[String]) -> Result<(), String> {
  static_files::validate_names(names)
}

/// Shared safe output projection for the manifest, report verifier and transport.
pub fn not_found_outputs(plan: &DocPlan) -> Vec<serde_json::Value> {
  plan
    .not_found
    .iter()
    .map(|page| {
      serde_json::json!({
        "locale": page.locale, "routeBase": page.route_base, "outputPath": page.output_path
      })
    })
    .collect()
}
pub fn schema() -> schemars::schema::RootSchema {
  schemars::schema_for!(DocPlan)
}
pub fn read_asset(asset: &PreparedAsset) -> Result<Vec<u8>, String> {
  let bytes = std::fs::read(&asset.source_path)
    .map_err(|cause| format!("[DOCS_ASSET_READ] {}: {cause}", asset.source_path))?;
  if hash(&bytes) != asset.source_hash {
    return Err(format!("[DOCS_SNAPSHOT_CHANGED] {}", asset.source_path));
  }
  Ok(bytes)
}
pub fn page_schema() -> schemars::schema::RootSchema {
  schemars::schema_for!(PageMetadata)
}
