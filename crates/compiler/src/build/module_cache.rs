use std::sync::Arc;

use farmfe_core::{
  cache::module_cache::CachedModule,
  context::CompilationContext,
  module::ModuleId,
  swc_common::{Mark, Span, SyntaxContext},
};

use farmfe_toolkit::{
  script::swc_try_with::try_with,
  swc_ecma_transforms_base::resolver,
  swc_ecma_visit::{VisitMut, VisitMutWith},
};

pub fn get_cache_key(module_id: &ModuleId, content: &str) -> String {
  let code = format!("{}-{}", module_id.to_string(), content);
  let module_content_hash = farmfe_toolkit::hash::sha256(code.as_bytes(), 32);
  module_content_hash
}

pub fn try_read_module_cache(
  cache_key: &str,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<Option<CachedModule>> {
  if !context.config.persistent_cache.enabled() {
    return Ok(None);
  }

  if context
    .cache_manager
    .module_cache
    .has_module_cache(&cache_key)
  {
    let mut cached_module = context
      .cache_manager
      .module_cache
      .get_module_cache(&cache_key);

    // using swc resolver
    match &mut cached_module.module.meta {
      farmfe_core::module::ModuleMetaData::Script(script) => {
        try_with(
          context.meta.script.cm.clone(),
          &context.meta.script.globals,
          || {
            let ast = &mut script.ast;
            // clear ctxt
            ast.visit_mut_with(&mut ResetSpanVisitMut);

            let unresolved_mark = Mark::new();
            let top_level_mark = Mark::new();

            ast.visit_mut_with(&mut resolver(
              unresolved_mark,
              top_level_mark,
              cached_module.module.module_type.is_typescript(),
            ));

            script.top_level_mark = top_level_mark.as_u32();
            script.unresolved_mark = unresolved_mark.as_u32();
          },
        )?;
      }
      farmfe_core::module::ModuleMetaData::Css(_)
      | farmfe_core::module::ModuleMetaData::Html(_) => { /* do nothing */ }
      farmfe_core::module::ModuleMetaData::Custom(_) => { /* TODO: add a hook for custom module */ }
    }

    return Ok(Some(cached_module));
  }

  Ok(None)
}

struct ResetSpanVisitMut;

impl VisitMut for ResetSpanVisitMut {
  fn visit_mut_span(&mut self, span: &mut Span) {
    span.ctxt = SyntaxContext::empty();
  }
}

pub fn write_module_cache(
  cache_key: &str,
  cached_module: &mut CachedModule,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  if !context.config.persistent_cache.enabled() {
    return Ok(());
  }

  context
    .cache_manager
    .module_cache
    .set_module_cache(&cache_key, cached_module);

  Ok(())
}
