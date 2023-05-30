use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  hashbrown::HashSet,
  module::{module_group::ModuleGroupId, ModuleId},
  plugin::PluginHookContext,
  resource::{
    resource_pot::{
      JsResourcePotMetaData, ResourcePot, ResourcePotId, ResourcePotMetaData, ResourcePotType,
    },
    ResourceType,
  },
  swc_common::DUMMY_SP,
  swc_ecma_ast::{Expr, ExprStmt, Module as SwcModule, ModuleItem, Stmt},
};

use farmfe_plugin_runtime::render_resource_pot::resource_pot_to_runtime_object_lit;

use crate::generate::{
  partial_bundling::call_partial_bundling_hook,
  render_resource_pots::{
    render_resource_pot_generate_resources, render_resource_pots_and_generate_resources,
  },
};

use super::diff_and_patch_module_graph::DiffResult;

pub fn render_and_generate_update_resource(
  updated_module_ids: &Vec<ModuleId>,
  diff_result: &DiffResult,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<String> {
  let mut update_resource_pot = ResourcePot::new(
    ResourcePotId::new(String::from("__UPDATE_RESOURCE_POT__")),
    ResourcePotType::Js,
  );
  update_resource_pot.immutable = true;

  let module_graph = context.module_graph.read();

  for added in &diff_result.added_modules {
    if module_graph.module(added).unwrap().external {
      continue;
    }
    update_resource_pot.add_module(added.clone());
  }

  for updated in updated_module_ids {
    if module_graph.module(updated).unwrap().external {
      continue;
    }
    update_resource_pot.add_module(updated.clone());
  }

  drop(module_graph);

  let module_graph = context.module_graph.read();
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

  if context.config.sourcemap.is_all() {
    // find sourceMappingUrl= and remove it
    let str = String::from_utf8(js_resource.bytes).unwrap();
    let mut lines = str.lines();
    // remove the last line
    lines.next_back();
    let new_str = lines.collect::<Vec<_>>().join("\n");
    return Ok(new_str);
  }

  // TODO: also return sourcemap
  Ok(String::from_utf8(js_resource.bytes).unwrap())
}

pub fn regenerate_resources_for_affected_module_groups(
  affected_module_groups: HashSet<ModuleGroupId>,
  updated_module_ids: &Vec<ModuleId>,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  clear_resource_pot_of_modules_in_module_groups(&affected_module_groups, context);

  let mut affected_resource_pots_ids =
    generate_and_diff_resource_pots(&affected_module_groups, context)?;

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

  let resource_pots = resource_pot_map
    .resource_pots_mut()
    .into_iter()
    .filter(|rp| affected_resource_pots_ids.contains(&rp.id))
    .collect::<Vec<&mut ResourcePot>>();

  render_resource_pots_and_generate_resources(resource_pots, context, &Default::default())
}

fn generate_and_diff_resource_pots(
  module_groups: &HashSet<ModuleGroupId>,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<Vec<ResourcePotId>> {
  let mut module_group_graph = context.module_group_graph.write();
  // TODO: Make swc helpers for commonjs module like default and wildcard exports embedded in the module system to optimize the HMR time, as these two modules may be imported by most modules
  let modules = module_groups
    .iter()
    .fold(HashSet::new(), |mut acc, module_group_id| {
      let module_group = module_group_graph.module_group(module_group_id).unwrap();
      acc.extend(module_group.modules().clone());
      acc
    })
    .into_iter()
    .collect::<Vec<_>>();

  let resources_pots =
    call_partial_bundling_hook(&modules, context, &PluginHookContext::default())?;
  let resources_pots_ids = resources_pots
    .iter()
    .map(|rp| rp.id.clone())
    .collect::<Vec<_>>();

  let module_graph = context.module_graph.read();
  let mut resource_pot_map = context.resource_pot_map.write();

  let mut new_resource_pot_ids = vec![];

  for mut resource_pot in resources_pots {
    let mut module_groups = HashSet::new();

    for module_id in resource_pot.modules() {
      let module = module_graph.module(module_id).unwrap();
      module_groups.extend(module.module_groups.clone());
    }

    resource_pot.module_groups = module_groups.clone();

    for module_group_id in module_groups {
      let module_group = module_group_graph
        .module_group_mut(&module_group_id)
        .unwrap();
      let mut resources_pots_to_remove = vec![];

      // Remove the old resource pots
      for resource_pot in module_group.resource_pots() {
        if !resources_pots_ids.contains(resource_pot) {
          resources_pots_to_remove.push(resource_pot.clone());

          if resource_pot_map.has_resource_pot(resource_pot) {
            let resource_pot = resource_pot_map
              .remove_resource_pot(resource_pot)
              .unwrap_or_else(|| {
                panic!(
                  "The resource pot {:?} should be in the resource pot map",
                  resource_pot
                )
              });

            // also remove the related resource
            let mut resource_maps = context.resources_map.lock();

            for resource in resource_pot.resources() {
              resource_maps.remove(resource);
            }
          }
        }
      }

      for resource_pot in resources_pots_to_remove {
        module_group.remove_resource_pot(&resource_pot);
      }

      if module_group.has_resource_pot(&resource_pot.id) {
        // the resource pot is already in the module group
        continue;
      } else {
        new_resource_pot_ids.push(resource_pot.id.clone());
        module_group.add_resource_pot(resource_pot.id.clone());
      }
    }

    if !resource_pot_map.has_resource_pot(&resource_pot.id) {
      new_resource_pot_ids.push(resource_pot.id.clone());
      resource_pot_map.add_resource_pot(resource_pot);
    }
  }

  Ok(new_resource_pot_ids)
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
