use std::{path::PathBuf, sync::Arc};

use farmfe_core::{
  config::{bool_or_obj::BoolOrObj, Config},
  context::CompilationContext,
  error::Result,
  plugin::Plugin,
  resource::resource_pot::{ResourcePot, ResourcePotType},
  serde_json,
  swc_common::Mark,
  swc_ecma_ast::{EsVersion, Program},
  swc_ecma_parser::Syntax,
};
use farmfe_toolkit::{
  common::{build_source_map, create_swc_source_map, Source},
  css::{codegen_css_stylesheet, parse_css_stylesheet, ParseCssModuleResult},
  html::parse_html_document,
  script::{
    codegen_module, parse_module, swc_try_with::try_with, CodeGenCommentsConfig,
    ParseScriptModuleResult,
  },
  swc_css_minifier::minify,
  swc_ecma_minifier::{
    optimize,
    option::{ExtraOptions, MinifyOptions},
  },
  swc_ecma_transforms::fixer,
  swc_ecma_transforms_base::resolver,
  swc_ecma_visit::FoldWith,
  swc_html_minifier::minify_document,
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
    let (cm, _) = create_swc_source_map(Source {
      path: PathBuf::from(&resource_pot.name),
      content: resource_pot.meta.rendered_content.clone(),
    });

    try_with(cm.clone(), &context.meta.script.globals, || {
      let unresolved_mark = Mark::new();
      let top_level_mark = Mark::new();

      let ParseScriptModuleResult { ast, comments } = match parse_module(
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

      let mut program = Program::Module(ast);
      program = program.fold_with(&mut resolver(unresolved_mark, top_level_mark, false));

      let minify_options = match &*context.config.minify {
        BoolOrObj::Bool(_) => MinifyOptions {
          compress: Some(Default::default()),
          mangle: Some(Default::default()),
          ..Default::default()
        },
        BoolOrObj::Obj(obj) => serde_json::from_value(obj.clone()).unwrap(),
      };
      let mut program = optimize(
        program,
        cm.clone(),
        Some(&comments),
        None,
        &minify_options,
        &ExtraOptions {
          unresolved_mark,
          top_level_mark,
        },
      );

      program = program.fold_with(&mut fixer(Some(&comments)));

      let ast = match program {
        Program::Module(ast) => ast,
        _ => unreachable!(),
      };
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

  pub fn minify_html(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> Result<()> {
    let (cm, _) = create_swc_source_map(Source {
      path: PathBuf::from(&resource_pot.name),
      content: resource_pot.meta.rendered_content.clone(),
    });

    try_with(cm.clone(), &context.meta.html.globals, || {
      let mut ast = parse_html_document(
        &resource_pot.name,
        resource_pot.meta.rendered_content.clone(),
      )
      .expect("failed to parse html document");
      // TODO support html minify options
      minify_document(&mut ast, &Default::default());

      let minified_content =
        farmfe_toolkit::html::codegen_html_document(&ast, context.config.minify.enabled());

      resource_pot.meta.rendered_content = Arc::new(minified_content);
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
    if matches!(
      resource_pot.resource_pot_type,
      ResourcePotType::Js | ResourcePotType::Runtime
    ) {
      self.minify_js(resource_pot, context)?;
    } else if matches!(resource_pot.resource_pot_type, ResourcePotType::Css) {
      self.minify_css(resource_pot, context)?;
    } else if matches!(resource_pot.resource_pot_type, ResourcePotType::Html) {
      self.minify_html(resource_pot, context)?;
    }

    Ok(None)
  }
}
