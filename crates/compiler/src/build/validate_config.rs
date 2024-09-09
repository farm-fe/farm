use farmfe_core::config::Config;
use farmfe_toolkit::fs::{
  CONTENT_HASH, CONTENT_HASH_NEW, ENTRY_NAME, RESOURCE_NAME, RESOURCE_NAME_NEW,
};

/// Check possible errors in config
pub fn validate_config(config: &Config) {
  let mut errors = vec![];

  if config.input.len() > 2 && !config.output.entry_filename.contains(ENTRY_NAME) {
    errors.push(format!(
      "When `input` is more than one, `output.entry_filename` must contain {}",
      ENTRY_NAME
    ));
  }

  let is_contain_name = || {
    config.output.filename.contains(RESOURCE_NAME)
      || config.output.filename.contains(RESOURCE_NAME_NEW)
  };
  let is_contain_hash = || {
    config.output.filename.contains(CONTENT_HASH)
      || config.output.filename.contains(CONTENT_HASH_NEW)
  };

  if config.partial_bundling.enforce_resources.len() <= 1
    && !(is_contain_name() || is_contain_hash())
  {
    errors.push(format!(
      "`output.filename` must contain one of {}、{}、{}、{} when `partial_bundling.module_buckets` is not configured",
      RESOURCE_NAME, RESOURCE_NAME_NEW, CONTENT_HASH, CONTENT_HASH_NEW
    ));
  }

  if !errors.is_empty() {
    panic!("Config Validation Error: \n{}", errors.join("\n"));
  }
}
