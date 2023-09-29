use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  hashbrown::HashSet,
  module::{module_group::ModuleGroupId, ModuleId, ModuleType},
  resource::{
    resource_pot::{JsResourcePotMetaData, ResourcePot, ResourcePotMetaData, ResourcePotType},
    ResourceType,
  },
  swc_common::DUMMY_SP,
  swc_ecma_ast::{Expr, ExprStmt, Module as SwcModule, ModuleItem, Stmt},
};

use farmfe_plugin_css::transform_resource_pot::transform_css_resource_pot;
use farmfe_plugin_runtime::render_resource_pot::resource_pot_to_runtime_object_lit;

use crate::generate::render_resource_pots::{
  render_resource_pot_generate_resources, render_resource_pots_and_generate_resources,
};

use super::diff_and_patch_module_graph::DiffResult;

mod generate_and_diff_resource_pots;

use generate_and_diff_resource_pots::generate_and_diff_resource_pots;

pub fn render_and_generate_update_resource(
  updated_module_ids: &Vec<ModuleId>,
  diff_result: &DiffResult,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<String> {
  let mut update_resource_pot =
    ResourcePot::new(String::from("__UPDATE_RESOURCE_POT__"), ResourcePotType::Js);
  let mut update_css_resource_pot = ResourcePot::new(
    String::from("__UPDATE_CSS_RESOURCE_POT__"),
    ResourcePotType::Css,
  );
  update_resource_pot.immutable = true;

  let mut module_graph = context.module_graph.write();

  for added in &diff_result.added_modules {
    let module = module_graph.module(added).unwrap();

    if module.external {
      continue;
    }

    if module.module_type == ModuleType::Css {
      update_css_resource_pot.add_module(added.clone());
    } else {
      update_resource_pot.add_module(added.clone());
    }
  }

  for updated in updated_module_ids {
    let module = module_graph.module(updated).unwrap();

    if module.external {
      continue;
    }

    if matches!(module.module_type, ModuleType::Css) {
      update_css_resource_pot.add_module(updated.clone());
    } else {
      update_resource_pot.add_module(updated.clone());
    }
  }

  transform_css_resource_pot(&mut update_css_resource_pot, &mut module_graph, context)?;

  for module_id in update_css_resource_pot.modules() {
    update_resource_pot.add_module(module_id.clone());
  }

  let ast = resource_pot_to_runtime_object_lit(&mut update_resource_pot, &module_graph, context)?;
  // The hmr result should alway be a js resource
  update_resource_pot.meta = ResourcePotMetaData::Js(JsResourcePotMetaData {
    ast: SwcModule {
      body: vec![ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(Expr::Object(ast)),
      }))],
      span: DUMMY_SP,
      shebang: None,
    },
  });

  let update_resources = render_resource_pot_generate_resources(
    &mut update_resource_pot,
    context,
    &Default::default(),
    true,
  )?;

  let js_resource = update_resources
    .into_iter()
    .find(|r| matches!(r.resource_type, ResourceType::Js))
    .unwrap();

  // TODO: also return sourcemap
  Ok(String::from_utf8(js_resource.bytes).unwrap())
}

pub fn regenerate_resources_for_affected_module_groups(
  affected_module_groups: HashSet<ModuleGroupId>,
  diff_result: DiffResult,
  updated_module_ids: &Vec<ModuleId>,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  // if there are deps changes, update execution order
  {
    let mut module_graph = context.module_graph.write();
    module_graph.update_execution_order_for_modules();
  }

  clear_resource_pot_of_modules_in_module_groups(&affected_module_groups, context);

  let mut affected_resource_pots_ids = generate_and_diff_resource_pots(
    &affected_module_groups,
    &diff_result,
    updated_module_ids,
    context,
  )?;

  let mut resource_pot_map = context.resource_pot_map.write();
  // always rerender the updated module's resource pot
  let module_graph = context.module_graph.read();

  for updated_module_id in updated_module_ids {
    let module = module_graph.module(updated_module_id).unwrap();
    let resource_pot_id = module.resource_pot.as_ref().unwrap();

    if !affected_resource_pots_ids.contains(resource_pot_id) {
      affected_resource_pots_ids.push(resource_pot_id.clone());
    }

    // also remove the related resources, the resources will be regenerated later
    let mut resource_maps = context.resources_map.lock();
    let resource_pot = resource_pot_map.resource_pot_mut(resource_pot_id).unwrap();

    for resource in resource_pot.resources() {
      resource_maps.remove(resource);
    }

    resource_pot.clear_resources();
  }

  let mut resource_pots = resource_pot_map
    .resource_pots_mut()
    .into_iter()
    .filter(|rp| affected_resource_pots_ids.contains(&rp.id))
    .collect::<Vec<&mut ResourcePot>>();

  drop(module_graph);

  // call process_resource_pot_map hook
  context
    .plugin_driver
    .process_resource_pots(&mut resource_pots, context)?;

  render_resource_pots_and_generate_resources(resource_pots, context, &Default::default())
}

fn clear_resource_pot_of_modules_in_module_groups(
  module_group_id: &HashSet<ModuleGroupId>,
  context: &Arc<CompilationContext>,
) {
  for module_group_id in module_group_id {
    let mut module_graph = context.module_graph.write();
    let module_group_graph = context.module_group_graph.read();
    let module_group = module_group_graph.module_group(module_group_id).unwrap();

    for module_id in module_group.modules() {
      let module = module_graph.module_mut(module_id).unwrap();
      module.resource_pot = None;
    }
  }
}
