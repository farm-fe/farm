//! Diagnostic-only plan preparation; official native builds remain the acceptance gate.
use std::{env, fs, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
  let root = PathBuf::from(env::args().nth(1).ok_or("provide site root")?).canonicalize()?;
  let source = fs::read_to_string(root.join("farm-docs.site.json"))?;
  let config = farmfe_docs_config::parse_config(&source, &root)?;
  let plan = farmfe_docs_plan::prepare(&config).map_err(std::io::Error::other)?;
  println!(
    "{}",
    serde_json::json!({"scope":"Rust diagnostic only; not native build acceptance","routes":plan.routes.len(),"snapshotId":plan.snapshot_id,"routePaths":plan.routes.iter().map(|route|&route.page.route_path).collect::<Vec<_>>(),"searchTitles":plan.routes.iter().filter(|route|route.page.id=="search").map(|route|serde_json::json!({"route":route.page.route_path,"title":plan.seo[&route.page.key].title})).collect::<Vec<_>>()})
  );
  Ok(())
}
