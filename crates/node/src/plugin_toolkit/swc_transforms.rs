use std::sync::Arc;

use farmfe_core::{
  config::Mode,
  context::CompilationContext,
  error::Result,
  swc_common::{comments::NoopComments, Mark},
  swc_ecma_ast,
};
use farmfe_toolkit::{
  script::swc_try_with::try_with,
  swc_ecma_transforms::{
    helpers::inject_helpers,
    react::{react, Options, RefreshOptions},
  },
  swc_ecma_visit::VisitMutWith,
};
use farmfe_toolkit_plugin_types::swc_transforms::FarmSwcTransformReactOptions;

#[no_mangle]
pub fn farm_swc_transform_react(
  context: &Arc<CompilationContext>,
  ast: &mut swc_ecma_ast::Module,
  options: FarmSwcTransformReactOptions,
) -> Result<()> {
  let is_dev = matches!(context.config.mode, Mode::Development);
  let top_level_mark = Mark::from_u32(options.top_level_mark);
  let unresolved_mark = Mark::from_u32(options.unresolved_mark);

  try_with(
    context.meta.script.cm.clone(),
    &context.meta.script.globals,
    || {
      ast.visit_mut_with(&mut react(
        context.meta.script.cm.clone(),
        Some(NoopComments), // TODO parse comments
        Options {
          refresh: if is_dev {
            Some(RefreshOptions::default())
          } else {
            None
          },
          development: Some(is_dev),
          // runtime: Some(Runtime::Automatic),
          ..Default::default()
        },
        top_level_mark,
        unresolved_mark
      ));

      if options.inject_helpers {
        ast.visit_mut_with(&mut inject_helpers(unresolved_mark));
      }
    },
  )
}
