use std::sync::Arc;

use self::config::NormalizedMinifyOptions;
use farmfe_core::config::minify::MinifyOptions;
use farmfe_core::swc_common::util::take::Take;
use farmfe_core::{
  swc_common::{comments::SingleThreadedComments, Mark, SourceMap},
  swc_css_ast::Stylesheet,
  swc_html_ast::Document,
};
use swc_css_minifier::minify;
use swc_ecma_minifier::{optimize, option::ExtraOptions};
use swc_ecma_transforms::{fixer::paren_remover, resolver};
use swc_ecma_visit::VisitMutWith;
use swc_html_minifier::minify_document;

pub mod config;

pub fn minify_js_module(
  ast: &mut farmfe_core::swc_ecma_ast::Module,
  cm: Arc<SourceMap>,
  comments: &SingleThreadedComments,
  unresolved_mark: Mark,
  top_level_mark: Mark,
  minify_options: &MinifyOptions,
) {
  let options = NormalizedMinifyOptions::minify_options_for_module(minify_options)
    .into_js_minify_options(cm.clone());
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
        mangle_name_cache: None,
      },
    )
    .expect_module()
  });
}

pub fn minify_css_module(ast: &mut Stylesheet) {
  minify(ast, Default::default());
}

pub fn minify_html_module(ast: &mut Document) {
  minify_document(ast, &Default::default());
}
