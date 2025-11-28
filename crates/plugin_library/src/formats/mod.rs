use std::sync::Arc;

use farmfe_core::{
  config::ModuleFormat,
  context::CompilationContext,
  parking_lot::Mutex,
  plugin::{GeneratedResource, PluginHookContext},
  rayon::iter::{IntoParallelIterator, ParallelIterator},
  resource::resource_pot::ResourcePot,
  swc_common::Mark,
  swc_ecma_ast::Module,
  HashSet,
};
use farmfe_toolkit::swc_ecma_utils::StmtLikeInjector;

use crate::{
  formats::{cjs::emit_cjs_resources, esm::emit_esm_resources, umd::emit_umd_resources},
  utils::{inject_farm_runtime_helpers, strip_runtime_module_helper_import},
};

pub mod cjs;
pub mod esm;
pub mod umd;

pub struct GenerateLibraryFormatResourcesOptions {
  pub should_add_farm_node_require: bool,
}

pub fn generate_library_format_resources(
  resource_pot: &mut ResourcePot,
  runtime_module_helper_ast: &Module,
  all_used_helper_idents: &mut HashSet<String>,
  options: &GenerateLibraryFormatResourcesOptions,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<Vec<GeneratedResource>> {
  // remove import { defineExports } from '@farm-runtime/module-helper'; and replace it with the real function definition
  let extra_used_helper_idents =
    strip_runtime_module_helper_import(&mut resource_pot.meta.as_js_mut().ast);

  let helpers_need_inject = extra_used_helper_idents
    .iter()
    .filter(|i| !all_used_helper_idents.contains(*i))
    .cloned()
    .collect::<HashSet<_>>();

  // TODO support multiple bundle, in multiple bundle, the helper may not be in a separate resource
  if !helpers_need_inject.is_empty() {
    let meta = resource_pot.meta.as_js_mut();
    let globals = context.meta.get_resource_pot_globals(&resource_pot.id);
    let items = inject_farm_runtime_helpers(
      runtime_module_helper_ast,
      &helpers_need_inject,
      Mark::from_u32(meta.unresolved_mark),
      Mark::from_u32(meta.top_level_mark),
      globals.value(),
    );
    meta.ast.body.prepend_stmts(items);
  }

  all_used_helper_idents.extend(extra_used_helper_idents);

  let formats = context.config.output.format.as_multiple();

  // Do not clone resource pot if there is only one format to emit for performance reason
  if formats.len() == 1 {
    return emit_formatted_resource_pot(
      formats[0].clone(),
      resource_pot,
      runtime_module_helper_ast,
      all_used_helper_idents,
      options,
      context,
      hook_context,
    );
  }

  let result = Mutex::new(vec![]);

  formats.into_par_iter().try_for_each(|format| {
    let mut cloned_resource_pot = resource_pot.clone();

    result.lock().append(&mut emit_formatted_resource_pot(
      format,
      &mut cloned_resource_pot,
      runtime_module_helper_ast,
      all_used_helper_idents,
      options,
      context,
      hook_context,
    )?);
    Ok(())
  })?;

  Ok(result.into_inner())
}

fn emit_formatted_resource_pot(
  format: ModuleFormat,
  resource_pot: &mut ResourcePot,
  runtime_module_helper_ast: &Module,
  all_used_helper_idents: &HashSet<String>,
  options: &GenerateLibraryFormatResourcesOptions,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<Vec<GeneratedResource>> {
  match format {
    farmfe_core::config::ModuleFormat::EsModule => {
      emit_esm_resources(resource_pot, options, context, hook_context)
    }
    farmfe_core::config::ModuleFormat::CommonJs => emit_cjs_resources(
      resource_pot,
      runtime_module_helper_ast,
      all_used_helper_idents,
      options,
      context,
      hook_context,
    ),
    farmfe_core::config::ModuleFormat::UMD => emit_umd_resources(
      resource_pot,
      runtime_module_helper_ast,
      all_used_helper_idents,
      options,
      context,
      hook_context,
    ),
    farmfe_core::config::ModuleFormat::System
    | farmfe_core::config::ModuleFormat::IIFE
    | farmfe_core::config::ModuleFormat::AMD => {
      unimplemented!("AMD/System/IIFE format is not supported yet")
    }
  }
}
