use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{module_graph::ModuleGraph, ModuleId},
  parking_lot::Mutex,
  rayon::iter::{IntoParallelIterator, ParallelIterator},
  resource::{meta_data::js::RenderModuleResult, resource_pot::ResourcePot},
  swc_common::{Mark, SourceMap},
  HashMap,
};

use render_module::RenderModuleOptions;
use scope_hoisting::build_scope_hoisted_module_groups;

use self::render_module::render_module;

pub(crate) mod external;
pub(crate) mod merge_rendered_module;
mod render_module;
mod scope_hoisting;
mod source_replacer;
mod transform_async_module;

type ResourcePotModules = (Vec<RenderModuleResult>, HashMap<ModuleId, Arc<SourceMap>>);
pub fn render_resource_pot_modules(
  resource_pot: &ResourcePot,
  module_graph: &ModuleGraph,
  context: &Arc<CompilationContext>,
) -> Result<ResourcePotModules> {
  let modules = Mutex::new(vec![]);

  // group modules in the same group that can perform scope hoisting
  let scope_hoisting_module_groups =
    build_scope_hoisted_module_groups(resource_pot, module_graph, context);

  scope_hoisting_module_groups
    .into_par_iter()
    .try_for_each(|hoisted_group| {
      let module = module_graph
        .module(&hoisted_group.target_hoisted_module_id)
        .unwrap_or_else(|| {
          panic!(
            "Module not found: {:?}",
            &hoisted_group.target_hoisted_module_id
          )
        });

      let merged_globals: Option<farmfe_core::swc_common::Globals>;
      let original_globals = context.meta.get_globals(&module.id);

      let (render_module_options, module_ids) = if hoisted_group.hoisted_module_ids.len() > 1 {
        let result = hoisted_group.scope_hoist(module_graph, context)?;
        merged_globals = Some(result.globals);
        (
          RenderModuleOptions {
            module_id: module.id.clone(),
            ast: result.ast,
            comments: result.comments,
            hoisted_sourcemap: result.source_map,
            hoisted_external_modules: result.external_modules,
            globals: merged_globals.as_ref().unwrap(),
            unresolved_mark: result.unresolved_mark,
            top_level_mark: result.top_level_mark,
            module_graph,
            context,
          },
          result.module_ids,
        )
      } else {
        let meta = module.meta.as_script();
        (
          RenderModuleOptions {
            module_id: module.id.clone(),
            ast: meta.ast.clone(),
            comments: meta.comments.clone(),
            hoisted_sourcemap: context.meta.get_module_source_map(&module.id),
            hoisted_external_modules: Default::default(),
            globals: original_globals.value(),
            unresolved_mark: Mark::from_u32(meta.unresolved_mark),
            top_level_mark: Mark::from_u32(meta.top_level_mark),
            module_graph,
            context,
          },
          vec![module.id.clone()],
        )
      };

      let hoisted_sourcemap = render_module_options.hoisted_sourcemap.clone();

      let mut render_module_result = render_module(render_module_options)?;

      render_module_result.hoisted_module_ids = module_ids;

      modules
        .lock()
        .push((render_module_result, hoisted_sourcemap));

      Ok::<(), CompilationError>(())
    })?;

  // sort props by module id to make sure the order is stable
  let modules = modules.into_inner();

  let (mut modules, source_maps) = modules.into_iter().fold(
    (vec![], HashMap::default()),
    |(mut modules, mut source_maps), (result, map)| {
      source_maps.insert(result.module_id.clone(), map);
      modules.push(result);

      (modules, source_maps)
    },
  );

  modules.sort_by(|a, b| {
    a.module_id
      .id(context.config.mode)
      .cmp(&b.module_id.id(context.config.mode))
  });

  Ok((modules, source_maps))
}
