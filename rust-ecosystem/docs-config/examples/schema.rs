//! Emit schemas from the Rust protocol types for package/contract validation.
fn main() -> Result<(), Box<dyn std::error::Error>> {
  let paths: Vec<_> = std::env::args_os().skip(1).collect();
  if paths.len() != 2 && paths.len() != 3 {
    return Err("Usage: schema <input-schema.json> <public-schema.json> [types.ts]".into());
  }
  std::fs::write(
    &paths[0],
    serde_json::to_string_pretty(&farmfe_docs_config::schema())? + "\n",
  )?;
  std::fs::write(
    &paths[1],
    serde_json::to_string_pretty(&farmfe_docs_config::public_schema())? + "\n",
  )?;
  if let Some(path) = paths.get(2) {
    std::fs::write(path, farmfe_docs_config::typescript()?)?;
  }
  Ok(())
}
