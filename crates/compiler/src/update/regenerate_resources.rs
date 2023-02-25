use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  hashbrown::HashSet,
  module::{module_group::ModuleGroupId, ModuleId},
  plugin::PluginHookContext,
  rayon::prelude::{IntoParallelRefIterator, ParallelIterator},
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
    "__UPDATE_MODULE_GROUP__".into(),
  );

  for added in &diff_result.added_modules {
    update_resource_pot.add_module(added.clone());
  }

  for updated in updated_module_ids {
    update_resource_pot.add_module(updated.clone());
  }

  let module_graph = context.module_graph.read();
  let ast = resource_pot_to_runtime_object_lit(&mut update_resource_pot, &*module_graph, context);
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

  Ok(String::from_utf8(js_resource.bytes).unwrap())
}

pub fn regenerate_resources_for_affected_module_groups(
  affected_module_groups: HashSet<ModuleGroupId>,
  updated_module_ids: &Vec<ModuleId>,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  affected_module_groups
    .into_iter()
    .collect::<Vec<ModuleGroupId>>()
    .par_iter()
    .try_for_each(|module_group_id| -> farmfe_core::error::Result<()> {
      let resource_pots_ids = {
        let mut module_group_graph = context.module_group_graph.write();
        let module_group = module_group_graph
          .module_group_mut(module_group_id)
          .unwrap();
        let resource_pots =
          call_partial_bundling_hook(module_group, context, &PluginHookContext::default())?;
        let resource_pots_ids = resource_pots
          .iter()
          .map(|r| r.id.clone())
          .collect::<Vec<ResourcePotId>>();

        let mut resource_pot_graph = context.resource_pot_graph.write();
        let previous_resource_pots = module_group.resource_pots().clone();

        // remove the old resource pots from the graph
        for resource_pot in &previous_resource_pots {
          if !resource_pots_ids.contains(resource_pot) {
            let resource_pot = resource_pot_graph
              .remove_resource_pot(resource_pot)
              .unwrap();

            // also remove the related resource
            let mut resource_maps = context.resources_map.lock();

            for resource in resource_pot.resources() {
              resource_maps.remove(resource);
            }
          }
        }

        let mut new_resource_pot_ids = vec![];

        // add the new resource pots to the graph
        for resource_pot in resource_pots {
          if !previous_resource_pots.contains(&resource_pot.id) {
            new_resource_pot_ids.push(resource_pot.id.clone());
            resource_pot_graph.add_resource_pot(resource_pot);
          }
        }

        module_group.set_resource_pots(HashSet::from_iter(resource_pots_ids));

        // always rerender the updated module's resource pot
        let module_graph = context.module_graph.read();

        for updated_module_id in updated_module_ids {
          let module = module_graph.module(updated_module_id).unwrap();
          let resource_pot_id = module.resource_pot.as_ref().unwrap();

          if !new_resource_pot_ids.contains(resource_pot_id) {
            new_resource_pot_ids.push(resource_pot_id.clone());
          }
        }

        new_resource_pot_ids
      };

      let mut resource_pot_graph = context.resource_pot_graph.write();
      let resource_pots = resource_pot_graph
        .resource_pots_mut()
        .into_iter()
        .filter(|rp| resource_pots_ids.contains(&rp.id))
        .collect::<Vec<&mut ResourcePot>>();

      render_resource_pots_and_generate_resources(resource_pots, context, &Default::default())?;

      Ok(())
    })
}
