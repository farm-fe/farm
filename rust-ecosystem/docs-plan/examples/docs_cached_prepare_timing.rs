//! Diagnostic only: compare full fresh/cold/warm plans, never write site output.
use farmfe_docs_plan::{prepare, prepare_with_cache, ContentCache};
use std::{env, fs, path::PathBuf, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let root = PathBuf::from(env::args().nth(1).ok_or("provide site root")?).canonicalize()?;
  let source = fs::read_to_string(root.join("farm-docs.site.json"))?;
  let config = farmfe_docs_config::parse_config(&source, &root)?;
  let start = Instant::now();
  let fresh = prepare(&config).map_err(std::io::Error::other)?;
  let fresh_ms = start.elapsed().as_millis();
  let mut cache = ContentCache::default();
  let start = Instant::now();
  let cold = prepare_with_cache(&config, &mut cache).map_err(std::io::Error::other)?;
  let cold_ms = start.elapsed().as_millis();
  let start = Instant::now();
  let warm = prepare_with_cache(&config, &mut cache).map_err(std::io::Error::other)?;
  let warm_ms = start.elapsed().as_millis();
  let fresh_bytes = serde_json::to_vec(&fresh)?;
  if serde_json::to_vec(&cold)? != fresh_bytes || serde_json::to_vec(&warm)? != fresh_bytes {
    return Err("cached and fresh complete plan bytes differ".into());
  }
  println!(
    "{}",
    serde_json::json!({
      "scope":"Rust diagnostic only; complete fresh/cold/warm plan equality, not native ABI or website dev acceptance",
      "routes":fresh.routes.len(),"snapshotId":fresh.snapshot_id,
      "freshMs":fresh_ms,"coldMs":cold_ms,"warmMs":warm_ms,"completePlanBytesEqual":true
    })
  );
  Ok(())
}
