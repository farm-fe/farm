use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::{CompilationError, Result},
  plugin::{Plugin, PluginFinalizeResourcesHookParam},
  resource::{Resource, ResourceOrigin, ResourceType},
};
use farmfe_docs_plan::DocPlan;
use farmfe_docs_plan::resource_owner;
use farmfe_macro_plugin::farm_plugin;
use serde::Deserialize;
use std::{
  path::Path,
  sync::{Arc, RwLock},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
  #[serde(default)]
  plan_path: Option<String>,
  #[serde(default)]
  development: bool,
}

fn error(message: impl std::fmt::Display) -> CompilationError {
  CompilationError::GenericError(format!("[DOCS_SEO] {message}"))
}

#[farm_plugin]
pub struct FarmPluginDocsSeo {
  options: std::result::Result<Options, String>,
  plan: RwLock<Option<DocPlan>>,
}

impl FarmPluginDocsSeo {
  fn new(_config: &Config, options: String) -> Self {
    Self {
      options: serde_json::from_str(&options).map_err(|error| error.to_string()),
      plan: RwLock::new(None),
    }
  }
}

impl Plugin for FarmPluginDocsSeo {
  fn name(&self) -> &str {
    "FarmPluginDocsSeo"
  }

  fn priority(&self) -> i32 {
    101
  }

  fn build_start(&self, context: &Arc<CompilationContext>) -> Result<Option<()>> {
    *self.plan.write().map_err(error)? = None;
    let options = self.options.as_ref().map_err(error)?;
    if options.development == options.plan_path.is_some() {
      return Err(error("provide exactly one of planPath or development:true"));
    }
    if context.config.output.target_env.is_node() {
      return Ok(None);
    }
    if options.development {
      return Ok(Some(()));
    }
    let plan_path = options.plan_path.as_deref().unwrap_or_default();
    if plan_path.is_empty() {
      return Err(error("planPath must be nonempty"));
    }
    let path = Path::new(plan_path);
    let path = if path.is_absolute() {
      path.to_path_buf()
    } else {
      Path::new(&context.config.root).join(path)
    };
    let plan = farmfe_docs_plan::shared::read_plan(&path, context.shared_resources.as_deref())
      .map_err(error)?;
    farmfe_docs_plan::verify(&plan).map_err(error)?;
    *self.plan.write().map_err(error)? = Some(plan);
    Ok(Some(()))
  }

  fn finalize_resources(
    &self,
    param: &mut PluginFinalizeResourcesHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if context.config.output.target_env.is_node() {
      return Ok(None);
    }
    let guard = self.plan.read().map_err(error)?;
    let development_plan: DocPlan;
    let plan = if self.options.as_ref().map_err(error)?.development {
      let record = context
        .custom
        .lock()
        .get(farmfe_docs_plan::DEVELOPMENT_PLAN_CONTEXT_KEY)
        .and_then(|record| record.downcast_ref::<String>())
        .cloned()
        .ok_or_else(|| error("Rust development plan record is missing or has invalid type"))?;
      development_plan = serde_json::from_str(&record).map_err(error)?;
      &development_plan
    } else {
      guard
        .as_ref()
        .ok_or_else(|| error("prepared plan is not loaded"))?
    };
    // Every resource and collision is checked before mutating the actual map.
    // Docs runs at priority 100 and will include these resources in its manifest.
    let resources = farmfe_docs_seo::generate(plan).map_err(error)?;
    let mut candidate = param.resources_map.clone();
    candidate.retain(|name, resource| {
      name != &resource.name
        || !resource_owner::is_owned(
          resource
            .meta
            .get(resource_owner::META_KEY)
            .map(String::as_str),
          self.name(),
          name,
          &resource.bytes,
        )
    });
    let mut names: Vec<_> = candidate.keys().cloned().collect();
    names.extend(resources.keys().cloned());
    farmfe_docs_plan::validate_resource_names(&names).map_err(error)?;
    let origin = context
      .module_graph
      .read()
      .entries
      .keys()
      .next()
      .cloned()
      .ok_or_else(|| error("entry module is missing"))?;
    for (name, bytes) in resources {
      let resource_type =
        ResourceType::Custom(if name.ends_with(".xml") { "xml" } else { "txt" }.into());
      let marker = resource_owner::stamp(self.name(), &name, &bytes).map_err(error)?;
      let mut meta = farmfe_core::HashMap::default();
      meta.insert(resource_owner::META_KEY.into(), marker);
      candidate.insert(
        name.clone(),
        Resource {
          name,
          bytes,
          meta,
          resource_type,
          origin: ResourceOrigin::Module(origin.clone()),
          should_transform_output_filename: false,
          ..Default::default()
        },
      );
    }
    *param.resources_map = candidate;
    Ok(Some(()))
  }
}

#[cfg(test)]
mod tests;
