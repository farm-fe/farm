use std::{cell::LazyCell, path::PathBuf, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  error::CompilationError,
  module::ModuleId,
  plugin::{PluginHookContext, PluginResolveHookParam, ResolveKind},
  regex::Regex,
};
use farmfe_utils::relative;
use sass_embedded::Exception;

pub const CSS_URL_RE: LazyCell<Regex> =
  LazyCell::new(|| Regex::new(r#"url\((\s*('[^']+'|"[^"]+")\s*|[^'")]+)\)"#).unwrap());
pub const CSS_DATA_URI_RE: LazyCell<Regex> =
  LazyCell::new(|| Regex::new(r#"data-uri\((\s*('[^']+'|"[^"]+")\s*|[^'")]+)\)"#).unwrap());
pub const IMPORT_CSS_RE: LazyCell<Regex> =
  LazyCell::new(|| Regex::new(r#"@import ('[^']+\.css'|"[^"]+\.css"|[^'")]+\.css)"#).unwrap());
pub const FUNCTION_CALL_RE: LazyCell<Regex> =
  LazyCell::new(|| Regex::new(r#"^[A-Za-z_][\w-]*\("#).unwrap());
pub const EXTERNAL_RE: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"^(https?:)?//").unwrap());
pub const DATA_URL_RE: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"^\s*data:").unwrap());

pub fn rebase_urls(
  file: &str,
  root_file: &str,
  mut content: String,
  context: &Arc<CompilationContext>,
) -> sass_embedded::Result<String> {
  let file_path = PathBuf::from(file);
  let root_path = PathBuf::from(root_file);

  let file_dir = file_path.parent();
  let root_dir = root_path.parent();

  if file_dir == root_dir {
    return Ok(content);
  }

  if CSS_URL_RE.is_match(&content) {
    content = replace_url(file, root_file, content, "url", context)?;
  }

  if CSS_DATA_URI_RE.is_match(&content) {
    content = replace_url(file, root_file, content, "data-uri", context)?;
  }

  if IMPORT_CSS_RE.is_match(&content) {
    content = replace_import(file, root_file, content, context)?;
  }

  Ok(content)
}

fn replace_url(
  file: &str,
  root_file: &str,
  content: String,
  func_name: &str,
  context: &Arc<CompilationContext>,
) -> sass_embedded::Result<String> {
  replace(content, &CSS_URL_RE, |_, matched| {
    let (wrap, raw_url) = if matched.starts_with('\'') {
      ("'", matched.trim_matches('\''))
    } else if matched.starts_with('\"') {
      ("\"", matched.trim_matches('"'))
    } else {
      ("", matched)
    };

    let new_url = resolve(raw_url, file, root_file, context)?;
    Ok(format!("{func_name}({wrap}{new_url}{wrap})"))
  })
  .map_err(|e| Box::new(Exception::new(e.to_string())))
}

fn replace_import(
  file: &str,
  root_file: &str,
  content: String,
  context: &Arc<CompilationContext>,
) -> sass_embedded::Result<String> {
  replace(content, &CSS_URL_RE, |_, matched| {
    let (wrap, raw_url) = if matched.starts_with('\'') {
      ("'", matched.trim_matches('\''))
    } else if matched.starts_with('"') {
      ("\"", matched.trim_matches('"'))
    } else {
      ("", matched)
    };

    let new_url = resolve(raw_url, file, root_file, context)?;

    Ok(format!("@import {wrap}{new_url}{wrap}"))
  })
  .map_err(|e| Box::new(Exception::new(e.to_string())))
}

fn ignore_url(url: &str) -> bool {
  EXTERNAL_RE.is_match(url)
    || DATA_URL_RE.is_match(url)
    || url.starts_with('#')
    || FUNCTION_CALL_RE.is_match(url)
}

fn resolve(
  url: &str,
  file: &str,
  root_file: &str,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<String> {
  if ignore_url(url) {
    return Ok(url.to_string());
  }

  let (resolved_path, external) = context
    .plugin_driver
    .resolve(
      &PluginResolveHookParam {
        source: url.to_string(),
        importer: Some(ModuleId::new(file, "", &context.config.root)),
        kind: ResolveKind::CssAtImport,
      },
      context,
      &PluginHookContext {
        caller: Some("rust-plugin-sass-url-rebase".to_string()),
        ..Default::default()
      },
    )?
    .ok_or_else(|| CompilationError::GenericError(format!("can not resolve {url} from {file}")))
    .map(|res| (res.resolved_path, res.external))?;

  Ok(if !external {
    let root_dir = PathBuf::from(root_file).parent().unwrap().to_path_buf();
    relative(&root_dir.to_string_lossy(), &resolved_path)
  } else {
    url.to_string()
  })
}

fn replace<R>(content: String, re: &Regex, replacer: R) -> farmfe_core::error::Result<String>
where
  R: Fn(&str, &str) -> farmfe_core::error::Result<String>,
{
  let mut content_left = content.as_str();
  let mut result = String::new();

  while let Some(caps) = re.captures(content_left) {
    let raw = &caps[0];
    let matched = &caps[1];
    let index = content_left.find(raw).unwrap();
    result.push_str(&content_left[..index]);
    result.push_str(&replacer(raw, matched)?);
    let next = index + raw.len();
    content_left = &content_left[next..];
  }

  result.push_str(content_left);
  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::CSS_URL_RE;

  #[test]
  fn replace() {
    let content = "a { b: url('c'); d: url('e');".to_string();
    let result = super::replace(content, &CSS_URL_RE, |raw, matched| {
      Ok(raw.replace(matched, "'matched'"))
    })
    .unwrap();

    assert_eq!(
      result,
      "a { b: url('matched'); d: url('matched');".to_string()
    )
  }
}
