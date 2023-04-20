use std::sync::Arc;

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::Result,
  plugin::Plugin,
  resource::resource_pot::{
    JsResourcePotMetaData, ResourcePot, ResourcePotMetaData, ResourcePotType,
  },
  swc_common::Mark,
  swc_ecma_ast::Program,
};
use farmfe_toolkit::{
  script::swc_try_with::try_with,
  swc_ecma_minifier::{
    optimize,
    option::{ExtraOptions, MinifyOptions},
  },
  swc_ecma_transforms_base::resolver,
  swc_ecma_visit::VisitMutWith,
};

pub struct FarmPluginMinify {}

impl FarmPluginMinify {
  pub fn new(_config: &Config) -> Self {
    Self {}
  }

  pub fn minify_js(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> Result<()> {
    try_with(
      context.meta.script.cm.clone(),
      &context.meta.script.globals,
      || {
        println!("before minify_js: {:?}", resource_pot.meta.as_js().ast);

        let meta = resource_pot.take_meta();
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        let mut program = Program::Module(meta.take_js().ast);
        program.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));

        let program = optimize(
          program,
          context.meta.script.cm.clone(),
          None,
          None,
          &MinifyOptions {
            // TODO: make it configurable
            compress: Some(Default::default()),
            mangle: Some(Default::default()),
            ..Default::default()
          },
          &ExtraOptions {
            unresolved_mark,
            top_level_mark,
          },
        );

        let ast = match program {
          Program::Module(ast) => ast,
          _ => unreachable!(),
        };

        println!("after minify_js: {:?}", ast);

        resource_pot.meta = ResourcePotMetaData::Js(JsResourcePotMetaData { ast });
      },
    )
  }
}

impl Plugin for FarmPluginMinify {
  fn name(&self) -> &'static str {
    "FarmPluginMinify"
  }

  fn optimize_resource_pot(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Js) {
      self.minify_js(resource_pot, context)?;
    } else if matches!(resource_pot.resource_pot_type, ResourcePotType::Css) {
    } else if matches!(resource_pot.resource_pot_type, ResourcePotType::Html) {
    }
    Ok(None)
  }
}
