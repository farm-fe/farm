use std::sync::Arc;

use farmfe_core::{
  config::minify::MinifyOptions,
  context::CompilationContext,
  resource::resource_pot::ResourcePot,
  swc_common::{comments::SingleThreadedComments, util::take::Take, Mark},
};
use swc_ecma_minifier::{optimize, option::ExtraOptions};
use swc_ecma_transforms::fixer::paren_remover;
use swc_ecma_transforms_base::fixer::fixer;
use swc_ecma_visit::VisitMutWith;

use crate::{minify::config::NormalizedMinifyOptions, script::swc_try_with::try_with};

pub fn minify_js_resource_pot(
  resource_pot: &mut ResourcePot,
  minify_options: &MinifyOptions,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  let cm = context.meta.get_resource_pot_source_map(&resource_pot.id);
  let globals = context.meta.get_resource_pot_globals(&resource_pot.id);

  try_with(cm.clone(), globals.value(), || {
    let comments = resource_pot.meta.as_js_mut().take_comments();
    let unresolved_mark = Mark::from_u32(resource_pot.meta.as_js().unresolved_mark);
    let top_level_mark = Mark::from_u32(resource_pot.meta.as_js().top_level_mark);
    let comments: SingleThreadedComments = comments.into();
    let ast = &mut resource_pot.meta.as_js_mut().ast;

    let options = NormalizedMinifyOptions::minify_options_for_resource_pot(minify_options)
      .into_js_minify_options(cm.clone());

    ast.visit_mut_with(&mut paren_remover(Some(&comments)));

    ast.map_with_mut(|m| {
      optimize(
        m.into(),
        cm.clone(),
        Some(&comments),
        None,
        &options,
        &ExtraOptions {
          unresolved_mark,
          top_level_mark,
          mangle_name_cache: None,
        },
      )
      .expect_module()
    });

    ast.visit_mut_with(&mut fixer(Some(&comments)));
  })
}
