use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  plugin::{GeneratedResource, PluginHookContext},
  resource::resource_pot::ResourcePot,
  swc_ecma_ast::Module,
  HashSet,
};
use farmfe_toolkit::swc_ecma_utils::StmtLikeInjector;

use crate::{
  formats::{cjs::emit_cjs_resources, esm::emit_esm_resources},
  utils::{inject_farm_runtime_helpers, strip_runtime_module_helper_import},
};

pub mod cjs;
pub mod esm;

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
    let items = inject_farm_runtime_helpers(runtime_module_helper_ast, &helpers_need_inject);
    resource_pot.meta.as_js_mut().ast.body.prepend_stmts(items);
  }

  all_used_helper_idents.extend(extra_used_helper_idents);

  let mut result = vec![];

  if context.config.output.format.contains_esm() {
    result.extend(emit_esm_resources(
      resource_pot,
      options,
      context,
      hook_context,
    )?);
  }

  if context.config.output.format.contains_cjs() {
    result.extend(emit_cjs_resources(
      resource_pot,
      runtime_module_helper_ast,
      all_used_helper_idents,
      options,
      context,
      hook_context,
    )?);
  }

  Ok(result)
}
