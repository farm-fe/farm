use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  plugin::{GeneratedResource, PluginHookContext},
  resource::resource_pot::ResourcePot,
  swc_common::Mark,
  swc_ecma_ast::{Expr, Module},
  HashSet,
};
use farmfe_toolkit::{
  script::{
    create_top_level_ident,
    module2cjs::{self, TransformModuleDeclsOptions},
    swc_try_with::{resolve_module_mark, try_with},
    transform_to_esm::transform_hybrid::Hybrid2CjsCalleeAllocator,
  },
  swc_ecma_transforms::hygiene::{hygiene_with_config, Config},
  swc_ecma_utils::StmtLikeInjector,
  swc_ecma_visit::VisitMutWith,
};

use crate::{
  formats::{esm::create_farm_node_require_item, GenerateLibraryFormatResourcesOptions},
  import_meta_visitor::replace_import_meta_url,
  utils::{add_format_to_generated_resources, emit_resource_pot, inject_farm_runtime_helpers},
};

pub fn emit_cjs_resources(
  resource_pot: &mut ResourcePot,
  runtime_module_helper_ast: &Module,
  all_used_helper_idents: &HashSet<String>,
  options: &GenerateLibraryFormatResourcesOptions,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<Vec<GeneratedResource>> {
  let meta = resource_pot.meta.as_js_mut();
  let cm = context.meta.get_resource_pot_source_map(&resource_pot.id);
  let globals = context.meta.get_resource_pot_globals(&resource_pot.id);

  try_with(cm, globals.value(), || {
    // the syntax mark may be wrong after minify, so we need to re-resolve the marks if minify is enabled
    let (unresolved_mark, top_level_mark) = if context.config.minify.enabled() {
      resolve_module_mark(&mut meta.ast, false, globals.value())
    } else {
      (
        Mark::from_u32(meta.unresolved_mark),
        Mark::from_u32(meta.top_level_mark),
      )
    };

    let callee_allocator = Hybrid2CjsCalleeAllocator::new(top_level_mark);

    module2cjs::transform_module_decls(
      &mut meta.ast,
      unresolved_mark,
      &callee_allocator,
      TransformModuleDeclsOptions {
        is_target_legacy: false,
        ..Default::default()
      },
    );
    meta.ast.visit_mut_with(&mut hygiene_with_config(Config {
      top_level_mark,
      ..Default::default()
    }));

    // TODO add test case
    replace_import_meta_url(&mut meta.ast, Mark::from_u32(meta.unresolved_mark));

    // TODO add exportStar for cjs entries and entries that reexport from cjs or hybrid modules

    let used_helper_idents: HashSet<&str> =
      std::mem::take(&mut callee_allocator.used_helper_idents.borrow_mut());
    let used_helper_idents = used_helper_idents
      .into_iter()
      .filter(|i| !all_used_helper_idents.contains(*i))
      .map(|s| s.to_string())
      .collect::<HashSet<_>>();

    if !used_helper_idents.is_empty() {
      // TODO support multiple bundle
      let items = inject_farm_runtime_helpers(runtime_module_helper_ast, &used_helper_idents);
      // prepend helper ast
      meta.ast.body.prepend_stmts(items);
    }

    // prepend `const __farmNodeRequire = require`
    if options.should_add_farm_node_require {
      let unresolved_mark = Mark::from_u32(meta.unresolved_mark);
      let require_item = create_farm_node_require_item(Expr::Ident(create_top_level_ident(
        "require",
        unresolved_mark,
      )));
      meta.ast.body.prepend_stmt(require_item);
    }
  })
  .unwrap();

  let mut resources = emit_resource_pot(resource_pot, context, hook_context)?;
  add_format_to_generated_resources(&mut resources, "cjs");

  if options.should_add_farm_node_require {
    // revert ast
    let ast = &mut resource_pot.meta.as_js_mut().ast;
    ast.body.remove(0);
  }
  Ok(resources)
}
