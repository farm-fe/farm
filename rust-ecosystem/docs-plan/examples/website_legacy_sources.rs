//! Website migration diagnostic, not a replacement for native build acceptance.
use std::{env, fs, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
  let root = PathBuf::from(env::args().nth(1).ok_or("provide website root")?).canonicalize()?;
  let config = farmfe_docs_config::parse_config(
    &fs::read_to_string(root.join("farm-docs.site.json"))?,
    &root,
  )?;
  let plan = farmfe_docs_plan::prepare(&config).map_err(std::io::Error::other)?;
  let mut rows = Vec::new();
  for (target, source) in [("zh-current", "en-current"), ("zh-1x", "en-1x")] {
    for id in ["benchmark", "using-plugins"] {
      let (index, route) = plan
        .routes
        .iter()
        .enumerate()
        .find(|(_, route)| route.page.collection == target && route.page.id == id)
        .ok_or("missing target")?;
      let (original, source_route) = plan
        .routes
        .iter()
        .enumerate()
        .find(|(_, route)| route.page.collection == source && route.page.id == id)
        .ok_or("missing source")?;
      let fallback = route.page.fallback_source.as_ref().ok_or_else(|| {
        format!(
          "legacy source mismatch: {} must retain English baseline",
          route.page.route_path
        )
      })?;
      if fallback.collection != source
        || plan.modules[index].code != plan.modules[original].code
        || plan.modules[index].source_path != plan.modules[original].source_path
      {
        return Err("fallback must retain exact real English module".into());
      }
      rows.push(serde_json::json!({"route":route.page.route_path,"sourceRoute":source_route.page.route_path,"source":plan.modules[index].source_path,"compiledBodyEqualsSource":true}));
    }
  }
  println!(
    "{}",
    serde_json::json!({"scope":"Rust diagnostic only; actual native full build and browser comparison still required","pages":plan.routes.len(),"rows":rows})
  );
  Ok(())
}
