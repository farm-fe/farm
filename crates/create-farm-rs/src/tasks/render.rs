use std::path::PathBuf;

use include_json::include_json;

pub struct RenderTask;

impl RenderTask {
  pub fn new() -> Self {
    Self
  }
}

impl super::Task for RenderTask {
  fn run(&mut self, ctx: &mut crate::utils::context::Context) -> anyhow::Result<()> {
    let target_dir: PathBuf = serde_json::from_value(ctx.get("target_dir").unwrap().clone())?;
    let template = *ctx.template();

    let farm_core_pkg_json = include_json!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/../../packages/core/package.json"
    ));
    let farm_cli_pkg_json = include_json!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/../../packages/cli/package.json"
    ));

    ctx.insert("farm_core_version", farm_core_pkg_json.get("version"));
    ctx.insert("farm_cli_version", farm_cli_pkg_json.get("version"));

    template.render(&target_dir, ctx)?;

    Ok(())
  }

  fn next(&self) -> Option<Vec<Box<dyn super::Task>>> {
    Some(vec![Box::new(super::git::GitTask)])
  }
}
