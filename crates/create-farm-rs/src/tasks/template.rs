use crate::{
  package_manager::PackageManager,
  template::{ElectronSubTemplate, TauriSubTemplate, Template},
  utils::prompts,
};

pub struct TemplateSelectTask;

impl TemplateSelectTask {
  pub fn new() -> Self {
    Self
  }
}

impl super::Task for TemplateSelectTask {
  fn run(&mut self, ctx: &mut crate::utils::context::Context) -> anyhow::Result<()> {
    let default_pkg_manager = ctx.options.manager.unwrap_or_default();
    let all_pkg_managers = PackageManager::all();
    let pkg_manager = prompts::select(
      "Package manager",
      &all_pkg_managers,
      Some(
        all_pkg_managers
          .iter()
          .position(|&m| m == default_pkg_manager)
          .unwrap_or(0),
      ),
    )?;

    ctx.insert("pkg_manager", &pkg_manager.to_string());

    let templates_no_flavors = pkg_manager.templates_no_flavors();

    let template = match ctx.options.template {
      Some(template) => template,
      None => {
        let selected_template =
          prompts::select("Select a framework:", &templates_no_flavors, Some(0))?;

        match selected_template {
          Template::Tauri(None) => {
            let sub_templates = vec![
              TauriSubTemplate::React,
              TauriSubTemplate::Vue,
              TauriSubTemplate::Svelte,
              TauriSubTemplate::Vanilla,
              TauriSubTemplate::Solid,
              TauriSubTemplate::Preact,
            ];

            let sub_template =
              prompts::select("Select a Tauri template:", &sub_templates, Some(0))?;

            Template::Tauri(Some(*sub_template))
          }
          Template::Electron(None) => {
            let sub_templates = vec![
              ElectronSubTemplate::React,
              ElectronSubTemplate::Vue,
              ElectronSubTemplate::Svelte,
              ElectronSubTemplate::Vanilla,
              ElectronSubTemplate::Solid,
              ElectronSubTemplate::Preact,
            ];

            let sub_template =
              prompts::select("Select an Electron template:", &sub_templates, Some(0))?;

            Template::Electron(Some(*sub_template))
          }
          _ => *selected_template,
        }
      }
    };
    ctx.insert("template", &template.to_string());

    ctx.set_template(template);

    Ok(())
  }

  fn next(&self) -> Option<Box<dyn super::Task>> {
    Some(Box::new(super::render::RenderTask::new()))
  }
}
