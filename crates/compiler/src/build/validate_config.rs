use farmfe_core::config::Config;
use farmfe_toolkit::fs::{ENTRY_NAME, RESOURCE_NAME};

/// Check possible errors in config
pub fn validate_config(config: &Config) {
  let mut errors = vec![];

  if config.input.len() > 2 && !config.output.entry_filename.contains(ENTRY_NAME) {
    errors.push(format!(
      "When `input` is more than one, `output.entry_filename` must contain {}",
      ENTRY_NAME
    ));
  }

  if config.partial_bundling.enforce_resources.len() <= 1
    && !config.output.filename.contains(RESOURCE_NAME)
  {
    errors.push(format!(
      "`output.filename` must contain {} when `partial_bundling.module_buckets` is not configured",
      RESOURCE_NAME
    ));
  }

  if !errors.is_empty() {
    panic!("Config Validation Error: \n{}", errors.join("\n"));
  }
}
