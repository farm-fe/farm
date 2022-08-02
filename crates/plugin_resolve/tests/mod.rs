use std::path::PathBuf;

use farmfe_core::{config::ResolveConfig, plugin::ResolveKind};
use farmfe_plugin_resolve::resolver::Resolver;
use testing_macros::fixture;

#[fixture("tests/fixtures/**/index.*")]
fn resolve_relative_specifier(file: PathBuf) {
  let resolver = Resolver::new(ResolveConfig::default());
  let cwd = file.parent().unwrap().to_path_buf();

  let resolved = resolver.resolve("./index", cwd.clone(), &ResolveKind::Entry);
  assert!(resolved.is_ok());
  let resolved = resolved.unwrap();
  assert_eq!(
    resolved.id,
    cwd.join("index.ts").to_string_lossy().to_string()
  );
}
