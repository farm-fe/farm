use std::sync::Arc;

use base64::{engine::general_purpose, Engine};
use farmfe_core::{context::CompilationContext, resource::ResourceType, HashMap};

use crate::codegen::{INLINE_PLACEHOLDER_PREFIX, INLINE_PLACEHOLDER_SUFFIX};

/// Scan all JS resources for inline-worker placeholders and replace them with
/// the actual base64-encoded worker chunk.
///
/// Must be called **after** resources have been generated (i.e. in `generate_end`
/// or `update_finished`).
pub fn patch_inline_worker_resources(context: &Arc<CompilationContext>) {
  let mut resources_map = context.resources_map.lock();

  // Step 1 — collect (resource_name, placeholder, entry_name) for all placeholders.
  let mut patches_needed: Vec<(String, String, String)> = vec![];

  for (name, resource) in resources_map.iter() {
    if !matches!(resource.resource_type, ResourceType::Js) {
      continue;
    }
    let content = String::from_utf8_lossy(&resource.bytes);
    let mut search_start = 0_usize;
    while let Some(prefix_pos) = content[search_start..].find(INLINE_PLACEHOLDER_PREFIX) {
      let abs_start = search_start + prefix_pos;
      let after_prefix = abs_start + INLINE_PLACEHOLDER_PREFIX.len();
      if let Some(suffix_pos) = content[after_prefix..].find(INLINE_PLACEHOLDER_SUFFIX) {
        let abs_end = after_prefix + suffix_pos + INLINE_PLACEHOLDER_SUFFIX.len();
        let placeholder = content[abs_start..abs_end].to_string();
        let entry_name = content[after_prefix..after_prefix + suffix_pos].to_string();
        patches_needed.push((name.clone(), placeholder, entry_name));
        search_start = abs_end;
      } else {
        break;
      }
    }
  }

  if patches_needed.is_empty() {
    return;
  }

  // Step 2 — for each unique entry_name, build the base64 of the worker chunk.
  let mut entry_to_base64: HashMap<String, String> = HashMap::default();
  for (_, _, entry_name) in &patches_needed {
    if entry_to_base64.contains_key(entry_name.as_str()) {
      continue;
    }
    let worker_bytes = resources_map
      .iter()
      .find(|(k, _)| k.starts_with(entry_name.as_str()))
      .map(|(_, r)| r.bytes.clone())
      .unwrap_or_default();

    if worker_bytes.is_empty() {
      continue;
    }

    let base64 = general_purpose::STANDARD.encode(&worker_bytes);
    entry_to_base64.insert(entry_name.clone(), base64);
  }

  // Step 3 — apply replacements.
  for (resource_name, placeholder, entry_name) in &patches_needed {
    if let Some(base64) = entry_to_base64.get(entry_name.as_str())
      && let Some(resource) = resources_map.get_mut(resource_name.as_str())
    {
      let content = String::from_utf8_lossy(&resource.bytes);
      resource.bytes = content
        .replace(placeholder.as_str(), base64.as_str())
        .into_bytes();
    }
  }
}
