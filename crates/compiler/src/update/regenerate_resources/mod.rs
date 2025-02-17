use std::sync::Arc;

use farmfe_core::config::FARM_MODULE_SYSTEM;
use farmfe_core::enhanced_magic_string::collapse_sourcemap::{
  collapse_sourcemap_chain, CollapseSourcemapOptions,
};
use farmfe_core::enhanced_magic_string::magic_string::MagicString;
use farmfe_core::enhanced_magic_string::types::SourceMapOptions;
use farmfe_core::plugin::PluginHookContext;
use farmfe_core::{
  context::CompilationContext,
  error::CompilationError,
  module::{module_group::ModuleGroupId, Module, ModuleId},
  resource::resource_pot::{ResourcePot, ResourcePotType},
};
use farmfe_core::{HashMap, HashSet};
use farmfe_toolkit::html::get_farm_global_this;
use farmfe_toolkit::sourcemap::SourceMap;
use farmfe_utils::hash::base64_encode;

use crate::generate::render_resource_pots::render_resource_pot_generate_resources;
use crate::{
  generate::render_resource_pots::render_resource_pots_and_generate_resources, write_cache,
};

use super::diff_and_patch_module_graph::DiffResult;

mod generate_and_diff_resource_pots;

use generate_and_diff_resource_pots::generate_and_diff_resource_pots;

/// This function only works when targetEnv is browser
pub fn render_and_generate_update_resource(
  updated_module_ids: &Vec<ModuleId>,
  diff_result: &DiffResult,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<(String, String)> {
  let mut immutable_update_resource_pot = ResourcePot::new(
    String::from("__IMMUTABLE_UPDATE_RESOURCE_POT__"),
    ResourcePotType::Js,
  );
  immutable_update_resource_pot.immutable = true;

  let mut mutable_update_resource_pot = ResourcePot::new(
    String::from("__MUTABLE_UPDATE_RESOURCE_POT__"),
    ResourcePotType::Js,
  );
  mutable_update_resource_pot.immutable = false;

  let module_graph = context.module_graph.read();

  for added in &diff_result.added_modules {
    let module = module_graph.module(added).unwrap();

    if module.external {
      continue;
    }

    if module.immutable {
      immutable_update_resource_pot.add_module(added.clone());
    } else {
      mutable_update_resource_pot.add_module(added.clone());
    }
  }

  for updated in updated_module_ids {
    let module = module_graph.module(updated).unwrap();

    if module.external {
      continue;
    }

    if module.immutable {
      immutable_update_resource_pot.add_module(updated.clone());
    } else {
      mutable_update_resource_pot.add_module(updated.clone());
    }
  }

  let gen_resource_pot_code =
    |resource_pot: &mut ResourcePot| -> farmfe_core::error::Result<String> {
      let hook_context = PluginHookContext::default();
      let res = context
        .plugin_driver
        .render_resource_pot(resource_pot, context, &hook_context)?
        .ok_or(CompilationError::GenerateResourcesError {
          name: resource_pot.id.clone(),
          ty: resource_pot.resource_pot_type.clone(),
          source: None,
        })?;
      resource_pot.meta = res;
      let (mut updated_result, _) =
        render_resource_pot_generate_resources(resource_pot, context, &Default::default())?;
      let update_resources = updated_result.resources.remove(0);

      let sourcemap = update_resources
        .source_map
        .map(|map| SourceMap::from_slice(&map.bytes).unwrap());

      let code = String::from_utf8(update_resources.resource.bytes).unwrap();
      let global_this = get_farm_global_this(
        &context.config.runtime.namespace,
        &context.config.output.target_env,
      );
      let mut magic_string = MagicString::new(&code, None);

      // force re-register the affected modules when hmr
      magic_string.prepend(&format!("{global_this}.{FARM_MODULE_SYSTEM}._rg=true;"));
      magic_string.append(&format!("{global_this}.{FARM_MODULE_SYSTEM}._rg=false;"));

      let code = magic_string.to_string();

      let map = if let Some(sourcemap) = sourcemap {
        // the updated code will be executed in the browser using new Function(code), so we need to
        // add extra (function anonymous() {})() to wrap the code to make sure the sourcemap works as expected
        magic_string.prepend("(function anonymous(\n) {\n");
        magic_string.append("\n})");

        let map = magic_string
          .generate_map(SourceMapOptions::default())
          .unwrap();
        let sourcemap =
          collapse_sourcemap_chain(vec![sourcemap, map], CollapseSourcemapOptions::default());

        let mut buf = vec![];
        sourcemap.to_writer(&mut buf).unwrap();
        let map_code = String::from_utf8(buf).unwrap();

        // inline source map
        format!(
          "\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,{}",
          base64_encode(map_code.as_bytes())
        )
      } else {
        "".to_string()
      };

      Ok(format!("{}{}", code, map))
    };

  let immutable_update_resource = gen_resource_pot_code(&mut immutable_update_resource_pot)?;
  let mutable_update_resource = gen_resource_pot_code(&mut mutable_update_resource_pot)?;

  Ok((immutable_update_resource, mutable_update_resource))
}

pub fn regenerate_resources_for_affected_module_groups(
  affected_module_groups: HashSet<ModuleGroupId>,
  diff_result: DiffResult,
  updated_module_ids: &Vec<ModuleId>,
  removed_modules: &HashMap<ModuleId, Module>,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  // if there are deps changes, update execution order
  {
    let mut module_graph = context.module_graph.write();
    module_graph.update_execution_order_for_modules();
  }

  // skip diff resource pots if diff_result is empty
  let mut affected_resource_pots_ids = if diff_result.added_modules.is_empty()
    && diff_result.removed_modules.is_empty()
    && diff_result.deps_changes.is_empty()
  {
    vec![]
  } else {
    clear_resource_pot_of_modules_in_module_groups(&affected_module_groups, context);
    generate_and_diff_resource_pots(
      &affected_module_groups,
      &diff_result,
      updated_module_ids,
      removed_modules,
      context,
    )?
  };

  let mut resource_pot_map = context.resource_pot_map.write();
  // always rerender the updated module's resource pot
  let module_graph = context.module_graph.read();

  for updated_module_id in updated_module_ids {
    let module = module_graph.module(updated_module_id).unwrap();

    for resource_pot_id in &module.resource_pots {
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
  }

  let mut resource_pots = resource_pot_map
    .resource_pots_mut()
    .into_iter()
    .filter(|rp| affected_resource_pots_ids.contains(&rp.id))
    .collect::<Vec<&mut ResourcePot>>();

  drop(module_graph);

  // call process_resource_pots hook
  context
    .plugin_driver
    .process_resource_pots(&mut resource_pots, context)?;

  render_resource_pots_and_generate_resources(resource_pots, context, &Default::default())?;

  if context.config.persistent_cache.enabled() {
    context
      .plugin_driver
      .write_plugin_cache(context)
      .unwrap_or_else(|err| {
        eprintln!("write plugin cache error: {err:?}");
      });

    write_cache(context.clone());
  }

  Ok(())
}

fn clear_resource_pot_of_modules_in_module_groups(
  module_group_id: &HashSet<ModuleGroupId>,
  context: &Arc<CompilationContext>,
) {
  let mut module_graph = context.module_graph.write();
  let module_group_graph = context.module_group_graph.read();

  for module_group_id in module_group_id {
    let module_group = module_group_graph.module_group(module_group_id).unwrap();

    for module_id in module_group.modules() {
      let module = module_graph.module_mut(module_id).unwrap();
      module.resource_pots = Default::default();
    }
  }
}
