use std::path::PathBuf;

use crate::{
  package_manager::PackageManager,
  utils::{colors::handle_brand_text, common::get_run_cmd},
};

pub struct TipsTask;

impl super::Task for TipsTask {
  fn run(&mut self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
    let target_dir: PathBuf = serde_json::from_value(ctx.get("target_dir").unwrap().clone())?;
    let cwd: PathBuf = serde_json::from_value(ctx.get("cwd").unwrap().clone())?;
    let project_name: String = serde_json::from_value(ctx.get("project_name").unwrap().clone())?;
    let pkg_manager: PackageManager =
      serde_json::from_value::<String>(ctx.get("pkg_manager").unwrap().clone())?
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;
    let template = ctx.template();

    handle_brand_text(" >  Initial Farm Project created successfully ✨ ✨ \n");

    if target_dir != cwd {
      handle_brand_text(&format!(
        "    cd {} \n",
        if project_name.contains(' ') {
          format!("\"{project_name}\"")
        } else {
          project_name.to_string()
        }
      ));
    }
    if let Some(cmd) = pkg_manager.install_cmd() {
      handle_brand_text(&format!("    {cmd} \n"));
    }
    handle_brand_text(&format!("    {} \n", get_run_cmd(&pkg_manager, &template)));

    Ok(())
  }
}
