use std::sync::Arc;

use farmfe_core::{
  config::minify::MinifyOptions, context::CompilationContext, error::Result,
  resource::resource_pot::ResourcePot, swc_common::Globals,
};
use farmfe_toolkit::{
  script::{minify::minify_js_resource_pot, swc_try_with::try_with},
  swc_css_minifier::minify,
};

pub fn minify_js(
  resource_pot: &mut ResourcePot,
  minify_options: &MinifyOptions,
  context: &Arc<CompilationContext>,
) -> Result<()> {
  minify_js_resource_pot(resource_pot, minify_options, context)
}

pub fn minify_css(resource_pot: &mut ResourcePot, context: &Arc<CompilationContext>) -> Result<()> {
  let cm = context.meta.get_resource_pot_source_map(&resource_pot.id);

  try_with(cm.clone(), &Globals::new(), || {
    let ast = &mut resource_pot.meta.as_css_mut().ast;
    // TODO support css minify options
    minify(ast, Default::default());
  })
}
