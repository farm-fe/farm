use std::{collections::BTreeMap, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  module::{
    module_group::{ModuleGroupId, ModuleGroupType},
    ModuleId,
  },
  resource::{Resource, ResourceType},
  HashMap,
};
use farmfe_docs_plan::DocPlan;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct PageResources {
  scripts: Vec<String>,
  styles: Vec<String>,
}

pub(crate) fn entry_path(key: &str) -> String {
  // A nonnumeric portable basename also remains a valid namespace stem when
  // Farm emits readable (unminified) runtime code for this dynamic entry.
  format!("virtual:farm-docs/pages/page_{key}.js")
}

/// The browser's finalized graph is authoritative for both lazy JS and no-JS CSS.
pub(crate) fn collect(
  plan: &DocPlan,
  context: &Arc<CompilationContext>,
  resources: &HashMap<String, Resource>,
) -> Result<Option<BTreeMap<String, PageResources>>, String> {
  let timing = (std::env::var_os("FARM_DOCS_UPDATE_TIMING").as_deref()
    == Some(std::ffi::OsStr::new("1")))
  .then(std::time::Instant::now);
  let graph = context.module_graph.read();
  if !graph.has_module(&ModuleId::new(
    super::APP_VIRTUAL_ID,
    "",
    &context.config.root,
  )) {
    return Ok(None);
  }
  let groups = context.module_group_graph.read();
  let pots = context.resource_pot_map.read();
  let mut result = BTreeMap::new();
  for module in &plan.modules {
    let id = ModuleId::new(&entry_path(&module.key), "", &context.config.root);
    let group = groups
      .module_group(&ModuleGroupId::new(&id, &ModuleGroupType::DynamicImport))
      .ok_or_else(|| {
        format!(
          "[DOCS_PAGE_RESOURCES] missing lazy module group: {}",
          module.key
        )
      })?;
    let mut page = PageResources {
      scripts: vec![],
      styles: vec![],
    };
    for pot_id in group.sorted_resource_pots(&graph, &pots) {
      let pot = pots
        .resource_pot(&pot_id)
        .ok_or_else(|| format!("[DOCS_PAGE_RESOURCES] missing resource pot: {pot_id}"))?;
      let mut names = pot.resources();
      names.sort();
      for name in names {
        let resource = resources
          .get(name)
          .ok_or_else(|| format!("[DOCS_PAGE_RESOURCES] missing resource: {name}"))?;
        let destination = match resource.resource_type {
          ResourceType::Js => &mut page.scripts,
          ResourceType::Css => &mut page.styles,
          _ => continue,
        };
        if !destination.contains(name) {
          destination.push(name.clone());
        }
      }
    }
    result.insert(module.key.clone(), page);
  }
  if let Some(start) = timing {
    use std::io::Write;
    let _ = writeln!(std::io::stderr(), "[FARM_UPDATE_TIMING] scope=page_resources stage=collected elapsed_us={} thread={:?} pages={}", start.elapsed().as_micros(), std::thread::current().id(), result.len());
  }
  Ok(Some(result))
}
