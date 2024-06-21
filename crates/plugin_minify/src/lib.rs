use std::{path::PathBuf, sync::Arc};

use farmfe_core::{
  config::{
    minify::{MinifyMode, MinifyOptions},
    Config,
  },
  context::CompilationContext,
  error::Result,
  plugin::Plugin,
  resource::resource_pot::{ResourcePot, ResourcePotType},
  swc_common::{util::take::Take, Mark},
  swc_ecma_ast::EsVersion,
  swc_ecma_parser::Syntax,
};
use farmfe_toolkit::{
  common::{build_source_map, create_swc_source_map, Source},
  css::{codegen_css_stylesheet, parse_css_stylesheet, ParseCssModuleResult},
  minify::config::NormalizedMinifyOptions,
  script::{
    codegen_module, parse_module, swc_try_with::try_with, CodeGenCommentsConfig,
    ParseScriptModuleResult,
  },
  swc_css_minifier::minify,
  swc_ecma_minifier::{optimize, option::ExtraOptions},
  swc_ecma_transforms::fixer,
  swc_ecma_transforms_base::{fixer::paren_remover, resolver},
  swc_ecma_visit::VisitMutWith,
};

pub struct FarmPluginMinify {
  minify_options: MinifyOptions,
}

impl FarmPluginMinify {
  pub fn new(config: &Config) -> Self {
    Self {
      minify_options: config
        .minify
        .clone()
        .map(|val| MinifyOptions::from(val))
        .unwrap_or_default(),
    }
  }

  pub fn minify_js(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> Result<()> {
    let (cm, _) = create_swc_source_map(Source {
      path: PathBuf::from(&resource_pot.name),
      content: resource_pot.meta.rendered_content.clone(),
    });

    try_with(cm.clone(), &context.meta.script.globals, || {
      let ParseScriptModuleResult { mut ast, comments } = match parse_module(
        &resource_pot.name,
        &resource_pot.meta.rendered_content,
        Syntax::Es(Default::default()),
        EsVersion::EsNext,
      ) {
        Ok(res) => res,
        Err(err) => {
          println!("{}", err.to_string());
          panic!("Parse {} failed. See error details above.", resource_pot.id);
        }
      };

      let options = NormalizedMinifyOptions::minify_options_for_resource_pot(&self.minify_options)
        .into_js_minify_options(cm.clone());

      let unresolved_mark = Mark::new();
      let top_level_mark = Mark::new();

      ast.visit_mut_with(&mut paren_remover(Some(&comments)));

      ast.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));

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
          },
        )
        .expect_module()
      });

      ast.visit_mut_with(&mut fixer(Some(&comments)));

      let sourcemap_enabled = context.config.sourcemap.enabled(resource_pot.immutable);

      let mut src_map = vec![];

      let minified_content = codegen_module(
        &ast,
        context.config.script.target.clone(),
        cm.clone(),
        if sourcemap_enabled {
          Some(&mut src_map)
        } else {
          None
        },
        context.config.minify.enabled(),
        Some(CodeGenCommentsConfig {
          comments: &comments,
          config: &context.config.comments,
        }),
      )
      .unwrap();

      resource_pot.meta.rendered_content = Arc::new(String::from_utf8(minified_content).unwrap());

      if sourcemap_enabled {
        let map = build_source_map(cm, &src_map);
        let mut buf = vec![];
        map.to_writer(&mut buf).expect("failed to write sourcemap");

        resource_pot
          .meta
          .rendered_map_chain
          .push(Arc::new(String::from_utf8(buf).unwrap()));
      }
    })
  }

  pub fn minify_css(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> Result<()> {
    let (cm, _) = create_swc_source_map(Source {
      path: PathBuf::from(&resource_pot.name),
      content: resource_pot.meta.rendered_content.clone(),
    });

    try_with(cm.clone(), &context.meta.css.globals, || {
      let ParseCssModuleResult {
        mut ast,
        comments: _,
      } = parse_css_stylesheet(
        &resource_pot.name,
        resource_pot.meta.rendered_content.clone(),
      )
      .unwrap();

      // TODO support css minify options
      minify(&mut ast, Default::default());

      let sourcemap_enabled = context.config.sourcemap.enabled(resource_pot.immutable);

      let (minified_content, map) = codegen_css_stylesheet(
        &ast,
        if sourcemap_enabled {
          Some(Source {
            path: PathBuf::from(&resource_pot.name),
            content: resource_pot.meta.rendered_content.clone(),
          })
        } else {
          None
        },
        context.config.minify.enabled(),
      );

      resource_pot.meta.rendered_content = Arc::new(minified_content);

      if let Some(map) = map {
        resource_pot.meta.rendered_map_chain.push(Arc::new(map));
      }
    })
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
    let enable_minify = context.config.minify.enabled();

    if !enable_minify {
      return Ok(None);
    }

    // if defer minify, ignore match mode
    if !resource_pot.is_defer_minify_as_resource_pot() {
      if !matches!(self.minify_options.mode, MinifyMode::ResourcePot) {
        return Ok(None);
      }
    }

    if matches!(
      resource_pot.resource_pot_type,
      ResourcePotType::Js | ResourcePotType::Runtime
    ) {
      self.minify_js(resource_pot, context)?;
    } else if matches!(resource_pot.resource_pot_type, ResourcePotType::Css) {
      self.minify_css(resource_pot, context)?;
    } else if matches!(resource_pot.resource_pot_type, ResourcePotType::Html) {
      // html minify is handled in plugin html after all resources are injected in finalize_resources hook
    }

    Ok(None)
  }
}
