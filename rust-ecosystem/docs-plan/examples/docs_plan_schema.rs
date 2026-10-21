//! A unique example name avoids collisions with docs-config's schema generator.
use std::{env, fs, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
  let repository = PathBuf::from(env::args().nth(1).ok_or("provide repository root")?);
  let directory = repository.join("packages/docs/schema");
  fs::create_dir_all(&directory)?;
  for (name, schema) in [
    ("input", farmfe_docs_config::schema()),
    ("public", farmfe_docs_config::public_schema()),
    ("plan", farmfe_docs_plan::schema()),
    ("sidebar", farmfe_docs_config::sidebar_schema()),
    ("page", farmfe_docs_plan::page_schema()),
  ] {
    fs::write(
      directory.join(format!("{name}.json")),
      serde_json::to_string_pretty(&schema)? + "\n",
    )?;
  }
  fs::write(
    repository.join("packages/docs/src/config.generated.ts"),
    farmfe_docs_config::typescript_with_schemas(vec![
      ("DocPlan".into(), farmfe_docs_plan::schema()),
      ("SidebarConfig".into(), farmfe_docs_config::sidebar_schema()),
      ("PageMetadata".into(), farmfe_docs_plan::page_schema()),
    ])?,
  )?;
  Ok(())
}
