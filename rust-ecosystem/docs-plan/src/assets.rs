use crate::{hash, prepare::farm_path, routes, PreparedAsset};
use farmfe_docs_config::ValidatedConfig;
use farmfe_docs_content::ContentReference;
use std::{
  collections::BTreeMap,
  fs,
  path::{Component, Path, PathBuf},
};

#[derive(Default)]
pub struct Registry {
  pub files: BTreeMap<String, PreparedAsset>,
  pub static_files: BTreeMap<String, PreparedAsset>,
}
pub struct ResolvedAsset {
  pub url: String,
  pub key: String,
  pub suffix: String,
}
fn error(reference: &ContentReference, code: &str, message: impl std::fmt::Display) -> String {
  format!(
    "[{code}] {}:{}:{}: {message}; target {:?}\n{}",
    reference.file, reference.line, reference.column, reference.target, reference.snippet
  )
}
impl Registry {
  pub fn with_static(files: &[PreparedAsset]) -> Self {
    Self {
      files: BTreeMap::new(),
      static_files: files
        .iter()
        .map(|file| (file.output_path.clone(), file.clone()))
        .collect(),
    }
  }
  pub(crate) fn static_file(
    &self,
    config: &farmfe_docs_config::PublicSiteConfig,
    reference: &ContentReference,
  ) -> Result<PreparedAsset, String> {
    let raw = reference.target.split(['?', '#']).next().unwrap_or("");
    let decoded = crate::links::decode(reference, raw)?;
    if !decoded.starts_with('/')
      || decoded.contains(['\\', ':'])
      || decoded.contains("//")
      || decoded.chars().any(char::is_control)
      || decoded.split('/').any(|part| matches!(part, "." | ".."))
    {
      return Err(error(
        reference,
        "DOCS_STATIC_PATH",
        "require a safe public resource path",
      ));
    }
    for part in raw.split('/') {
      let value = crate::links::decode(reference, part)?;
      if value.contains(['/', '\\']) {
        return Err(error(
          reference,
          "DOCS_STATIC_PATH",
          "encoded separators are not supported",
        ));
      }
    }
    let output = decoded
      .strip_prefix(&config.base_path)
      .unwrap_or_else(|| decoded.trim_start_matches('/'));
    self.static_files.get(output).cloned().ok_or_else(|| {
      error(
        reference,
        "DOCS_STATIC_MISSING",
        format!("no declared static file owns {decoded}"),
      )
    })
  }
  pub fn metadata_url(
    &self,
    config: &ValidatedConfig,
    reference: &ContentReference,
  ) -> Result<String, String> {
    if reference.target.starts_with('/') && !reference.target.starts_with("//") {
      let asset = self.static_file(&config.public, reference)?;
      let end = reference
        .target
        .find(['?', '#'])
        .unwrap_or(reference.target.len());
      Ok(format!(
        "{}{}",
        crate::static_files::public_url(&config.public, &asset.output_path),
        &reference.target[end..]
      ))
    } else {
      Ok(reference.target.clone())
    }
  }
  pub fn resolve(
    &mut self,
    config: &ValidatedConfig,
    reference: &ContentReference,
  ) -> Result<ResolvedAsset, String> {
    let target = &reference.target;
    let end = target.find(['?', '#']).unwrap_or(target.len());
    let raw = &target[..end];
    if raw.starts_with('/') {
      let asset = self.static_file(&config.public, reference)?;
      self.files.insert(asset.source_path.clone(), asset.clone());
      return Ok(ResolvedAsset {
        url: format!(
          "{}{}",
          crate::static_files::public_url(&config.public, &asset.output_path),
          &target[end..]
        ),
        key: asset.key,
        suffix: target[end..].into(),
      });
    }
    let decoded = crate::links::decode(reference, raw)?;
    if decoded.is_empty() || decoded.contains(['\\', ':']) || decoded.chars().any(char::is_control)
    {
      return Err(error(
        reference,
        "DOCS_ASSET_PATH",
        "require a local relative resource path",
      ));
    }
    for segment in raw.split('/') {
      let value = crate::links::decode(reference, segment)?;
      if segment != value && (matches!(value.as_str(), "." | "..") || value.contains(['/', '\\'])) {
        return Err(error(
          reference,
          "DOCS_ASSET_PATH",
          "encoded traversal or separators are not supported",
        ));
      }
    }
    let root = PathBuf::from(farm_path(&config.site_root)?);
    let parent = Path::new(&reference.file)
      .parent()
      .ok_or_else(|| error(reference, "DOCS_ASSET_PATH", "source has no parent"))?;
    let mut file = parent.to_path_buf();
    for part in decoded.split('/') {
      match part {
        "" | "." => {}
        ".." => {
          file.pop();
        }
        part => file.push(part),
      }
    }
    let relative = file.strip_prefix(&root).map_err(|_| {
      error(
        reference,
        "DOCS_ASSET_PATH",
        "resource leaves the site root",
      )
    })?;
    let mut cursor = root.clone();
    for part in relative.components() {
      if !matches!(part, Component::Normal(_)) {
        return Err(error(
          reference,
          "DOCS_ASSET_PATH",
          "ambiguous resource path",
        ));
      }
      if farmfe_docs_config::is_private_tooling_component(&part.as_os_str().to_string_lossy()) {
        return Err(error(
          reference,
          "DOCS_ASSET_PATH",
          "private tooling inputs cannot become public resources",
        ));
      }
      cursor.push(part);
      let metadata = fs::symlink_metadata(&cursor)
        .map_err(|cause| error(reference, "DOCS_ASSET_MISSING", cause))?;
      let reparse = metadata.file_type().is_symlink();
      #[cfg(windows)]
      let reparse = {
        use std::os::windows::fs::MetadataExt;
        reparse || metadata.file_attributes() & 0x400 != 0
      };
      if reparse {
        return Err(error(
          reference,
          "DOCS_ASSET_PATH",
          "symlinks and junctions are not supported",
        ));
      }
    }
    if !file.is_file() {
      return Err(error(
        reference,
        "DOCS_ASSET_PATH",
        "resource must be a file",
      ));
    }
    let source_path = farm_path(&file)?;
    let asset = if let Some(asset) = self.files.get(&source_path) {
      asset.clone()
    } else if let Some(asset) = self
      .static_files
      .values()
      .find(|asset| asset.source_path == source_path)
    {
      let asset = asset.clone();
      self.files.insert(source_path.clone(), asset.clone());
      asset
    } else {
      let bytes = fs::read(&file).map_err(|cause| error(reference, "DOCS_ASSET_READ", cause))?;
      let source_hash = hash(&bytes);
      let key = hash(
        &serde_json::to_vec(&(farm_path(relative)?, &source_hash)).map_err(|e| e.to_string())?,
      );
      let extension = file
        .extension()
        .and_then(|s| s.to_str())
        .filter(|s| {
          !s.is_empty() && s.len() <= 16 && s.chars().all(|ch| ch.is_ascii_alphanumeric())
        })
        .unwrap_or("bin")
        .to_ascii_lowercase();
      let output_path = format!("assets/docs/{key}.{extension}");
      routes::validate_output(&output_path, &reference.file)?;
      let asset = PreparedAsset {
        key,
        source_path: source_path.clone(),
        source_hash,
        output_path,
        is_static: false,
      };
      self.files.insert(source_path, asset.clone());
      asset
    };
    Ok(ResolvedAsset {
      url: format!(
        "{}{}",
        crate::static_files::public_url(&config.public, &asset.output_path),
        &target[end..]
      ),
      key: asset.key,
      suffix: target[end..].into(),
    })
  }
}
