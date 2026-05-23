use std::path::Path;

use farmfe_core::config::Config;
use farmfe_toolkit::fs::{
  transform_output_entry_filename, transform_output_filename, TransformOutputFileNameParams,
};
use regress::Regex as JsRegex;

use crate::constants::WORKER_OR_SHARED_WORKER_RE;

// Placeholder embedded in the inline worker module.  It is replaced with the
// actual base64-encoded bundle bytes in the `generate_end` hook after Farm has
// compiled the corresponding DynamicEntry resource.
pub const INLINE_PLACEHOLDER_PREFIX: &str = "__FARM_INLINE_WORKER:";
pub const INLINE_PLACEHOLDER_SUFFIX: &str = ":END__";

// ---------------------------------------------------------------------------
// URL / entry-name helpers
// ---------------------------------------------------------------------------

/// Compute a deterministic DynamicEntry name for a worker source file.
///
/// The name is used as `[entryName]` in `output.entry_filename`.  It is derived from
/// `output.assets_filename` applied to the worker source file, using `resolved_path` bytes
/// as the hash source (to disambiguate files with identical base names), then the file
/// extension is stripped.
///
/// Example with `assets_filename = "[name].[hash].[ext]"`:
///   resolved_path  = "/src/workers/upload.ts"
///   → `"upload.abc12345"`  (extension stripped)
pub fn compute_worker_entry_name(
  resolved_path: &str,
  _module_id: &str,
  compiler_config: &Config,
) -> String {
  let normalized_resolved_path = resolved_path.replace('\\', "/");
  let file_name_ext = normalized_resolved_path
    .rsplit('/')
    .next()
    .unwrap_or_default()
    .to_string();
  let (stem, ext) = file_name_ext
    .split_once('.')
    .unwrap_or((&file_name_ext, "js"));
  let full_filename = transform_output_filename(TransformOutputFileNameParams {
    filename_config: compiler_config.output.assets_filename.clone(),
    name: stem,
    name_hash: "",
    bytes: normalized_resolved_path.as_bytes(),
    ext,
    special_placeholders: &Default::default(),
  });
  // Strip extension so the name becomes a bare [entryName] token.
  // e.g. "upload.abc12345.js" → "upload.abc12345"
  Path::new(&full_filename)
    .file_stem()
    .map(|s| s.to_string_lossy().into_owned())
    .unwrap_or(full_filename)
}

/// Compute the public URL at which a worker bundle will be served.
///
/// Applies `output.entry_filename` with `entry_name` substituted for `[entryName]`.
/// A `public_path` prefix is prepended when configured.
pub fn compute_worker_url(entry_name: &str, compiler_config: &Config) -> String {
  let filename = transform_output_entry_filename(
    entry_name,
    TransformOutputFileNameParams {
      filename_config: compiler_config.output.entry_filename.clone(),
      name: entry_name,
      name_hash: "",
      bytes: &[],
      ext: "js",
      special_placeholders: &Default::default(),
    },
  );
  if compiler_config.output.public_path.is_empty() {
    format!("/{filename}")
  } else {
    let base = compiler_config.output.public_path.trim_end_matches('/');
    format!("{base}/{filename}")
  }
}

// ---------------------------------------------------------------------------
// Module code generators
// ---------------------------------------------------------------------------

fn worker_constructor(module_id: &str) -> &'static str {
  let m = JsRegex::new(WORKER_OR_SHARED_WORKER_RE)
    .unwrap()
    .find(module_id)
    .unwrap();
  match &module_id[m.group(1).unwrap()] {
    "sharedworker" => "SharedWorker",
    _ => "Worker",
  }
}

/// Generate the placeholder wrapper module for an inline (`?worker&inline`) worker.
///
/// The placeholder string `__FARM_INLINE_WORKER:<entry_name>:END__` is replaced
/// with the actual base64-encoded, self-contained worker bundle in the
/// `generate_end` hook after Farm has compiled the DynamicEntry resource.
pub fn get_inline_worker_placeholder_code(module_id: &str, entry_name: &str) -> String {
  let constructor = worker_constructor(module_id);
  let placeholder = format!("{INLINE_PLACEHOLDER_PREFIX}{entry_name}{INLINE_PLACEHOLDER_SUFFIX}");
  let encoded_decl = format!(r#"const encodedJs = "{placeholder}";"#);

  if constructor == "Worker" {
    format!(
      r#"{encoded_decl}
const decodeBase64 = (base64) => Uint8Array.from(atob(base64), c => c.charCodeAt(0));
const blob = typeof self !== "undefined" && self.Blob
  && new Blob([decodeBase64(encodedJs)], {{ type: "text/javascript;charset=utf-8" }});
export default function WorkerWrapper(options) {{
  let objURL;
  try {{
    objURL = blob && (self.URL || self.webkitURL).createObjectURL(blob);
    if (!objURL) throw '';
    const worker = new {constructor}(objURL, options);
    worker.addEventListener("error", () => {{
      (self.URL || self.webkitURL).revokeObjectURL(objURL);
    }});
    return worker;
  }} catch(e) {{
    return new {constructor}("data:text/javascript;base64," + encodedJs, options);
  }} finally {{
    objURL && (self.URL || self.webkitURL).revokeObjectURL(objURL);
  }}
}}"#
    )
  } else {
    format!(
      r#"{encoded_decl}
export default function WorkerWrapper(options) {{
  return new {constructor}("data:text/javascript;base64," + encodedJs, options);
}}"#
    )
  }
}

/// Generate the JS wrapper module for a non-inline, URL-based worker.
///
/// Worker constructor options are forwarded as-is, so callers can choose
/// `{ type: "module" }` when the worker output contains ESM imports.
///
/// - With `?url`: exports the URL string directly.
/// - Otherwise: exports a `WorkerWrapper` factory function.
pub fn get_worker_module_code(module_id: &str, worker_url: &str, is_url: bool) -> String {
  let constructor = worker_constructor(module_id);
  if is_url {
    return format!(r#"export default "{worker_url}""#);
  }
  format!(
    r#"export default function WorkerWrapper(options) {{
  return new {constructor}("{worker_url}", options);
}}"#
  )
}

#[cfg(test)]
mod tests {
  use farmfe_core::config::Config;

  use super::compute_worker_entry_name;

  #[test]
  fn worker_entry_name_is_stable_for_same_source_with_different_queries() {
    let mut config = Config::default();
    config.output.assets_filename = "asserts/[resourceName].[hash].[ext]".to_string();
    let resolved_path = "C:\\project\\src\\worker\\vue.worker.ts";

    let worker_entry =
      compute_worker_entry_name(resolved_path, "src/worker/vue.worker.ts?worker", &config);
    let worker_url_entry = compute_worker_entry_name(
      resolved_path,
      "src/worker/vue.worker.ts?worker&url",
      &config,
    );

    assert_eq!(worker_entry, worker_url_entry);
  }

  #[test]
  fn worker_entry_name_is_stable_for_same_source_with_different_path_separators() {
    let mut config = Config::default();
    config.output.assets_filename = "asserts/[resourceName].[hash].[ext]".to_string();

    let backslash_entry = compute_worker_entry_name(
      "C:\\project\\src\\worker\\vue.worker.ts",
      "src/worker/vue.worker.ts?worker",
      &config,
    );
    let slash_entry = compute_worker_entry_name(
      "C:/project/src/worker/vue.worker.ts",
      "src/worker/vue.worker.ts?worker",
      &config,
    );

    assert_eq!(backslash_entry, slash_entry);
  }
}
