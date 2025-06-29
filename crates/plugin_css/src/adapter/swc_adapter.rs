use std::{borrow::Cow, sync::Arc};

use farmfe_core::{
  config::{AliasItem, CssPrefixerConfig},
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{module_graph::ModuleGraph, Module, ModuleId},
  parking_lot::Mutex,
  plugin::PluginAnalyzeDepsHookResultEntry,
  rayon::iter::{IntoParallelIterator, ParallelIterator},
  resource::{
    meta_data::{css::CssResourcePotMetaData, ResourcePotMetaData},
    Resource,
  },
  swc_common::DUMMY_SP,
  swc_css_ast::Stylesheet,
  HashMap,
};
use farmfe_toolkit::{
  css::{codegen_css_stylesheet, merge_css_sourcemap, parse_css_stylesheet, ParseCssModuleResult},
  script::swc_try_with::try_with,
  swc_atoms::JsWord,
  swc_css_modules::TransformConfig,
  swc_css_prefixer,
  swc_css_visit::{VisitMut, VisitMutWith, VisitWith},
};

use crate::{
  adapter::adapter_trait::{
    CodegenContext, CreateResourcePotMetadataContext, ParseOption, SourceReplaceContext,
  },
  dep_analyzer::DepAnalyzer,
  source_replacer::SourceReplacer,
};

pub fn parse<'a>(
  content: Arc<String>,
  options: &Cow<'a, ParseOption>,
) -> Result<ParseCssModuleResult> {
  parse_css_stylesheet(&options.module_id, content)
}

pub struct CssModuleRename {
  pub indent_name: String,
  pub hash: String,
}

impl TransformConfig for CssModuleRename {
  fn new_name_for(&self, local: &JsWord) -> JsWord {
    let name = local.to_string();
    let r: HashMap<String, &String> = [("name".into(), &name), ("hash".into(), &self.hash)]
      .into_iter()
      .collect();
    transform_css_module_indent_name(self.indent_name.clone(), r).into()
  }
}

fn transform_css_module_indent_name(
  indent_name: String,
  context: HashMap<String, &String>,
) -> String {
  context.iter().fold(indent_name, |acc, (key, value)| {
    acc.replace(&format!("[{key}]"), value)
  })
}
pub fn css_modules() {
  // let ParseCssModuleResult {
  //   ast: mut css_stylesheet,
  //   comments,
  //   source_map,
  // } = parse_css_stylesheet(
  //   &css_modules_module_id.to_string(),
  //   Arc::new(param.content.clone()),
  // )?;
  // set source map and globals for css modules ast
  // context
  //   .meta
  //   .set_module_source_map(&css_modules_module_id, source_map);
  // context
  //   .meta
  //   .set_globals(&css_modules_module_id, Globals::default());

  // we can not use css_modules_resolved_path here because of the compatibility of windows. eg: \\ vs \\\\
  // let cache_id = css_modules_module_id.to_string();
  // self.ast_map.lock().insert(
  //   cache_id.clone(),
  //   (css_stylesheet, CommentsMetaData::from(comments)),
  // );
  // self
  //   .content_map
  //   .lock()
  //   .insert(cache_id, param.content.clone());
}

pub fn prefixer(stylesheet: &mut Stylesheet, css_prefixer_config: &CssPrefixerConfig) {
  let mut prefixer = swc_css_prefixer::prefixer(swc_css_prefixer::options::Options {
    env: css_prefixer_config.targets.clone(),
  });
  prefixer.visit_mut_stylesheet(stylesheet);
}

pub fn analyze(
  stylesheet: &Stylesheet,
  context: &Arc<CompilationContext>,
) -> Vec<PluginAnalyzeDepsHookResultEntry> {
  // Implement SwcCss analyze_deps logic here
  let mut dep_analyzer = DepAnalyzer::new(context.config.resolve.alias.clone());
  stylesheet.visit_with(&mut dep_analyzer);

  dep_analyzer.deps
}

pub fn source_replace_helper(
  stylesheet: &mut Stylesheet,
  module_id: &ModuleId,
  module_graph: &ModuleGraph,
  resources_map: &HashMap<String, Resource>,
  public_path: String,
  alias: Vec<AliasItem>,
) {
  let mut source_replacer = SourceReplacer::new(
    module_id.clone(),
    module_graph,
    resources_map,
    public_path,
    alias,
  );

  stylesheet.visit_mut_with(&mut source_replacer);
}

pub fn source_replace(
  SourceReplaceContext {
    module,
    module_graph,
    resources_map,
    context,
  }: SourceReplaceContext<'_>,
) -> Result<Stylesheet> {
  let cm = context.meta.get_module_source_map(&module.id);
  let mut css_stylesheet = module.meta.as_css().ast.clone();
  let globals = context.meta.get_globals(&module.id);

  try_with(cm, globals.value(), || {
    source_replace_helper(
      &mut css_stylesheet,
      &module.id,
      &module_graph,
      &resources_map,
      context.config.output.public_path.clone(),
      context.config.resolve.alias.clone(),
    );
  })?;

  Ok(css_stylesheet)
}

pub fn create_resource_pot_metadata(
  CreateResourcePotMetadataContext {
    context,
    modules,
    module_execution_order,
    module_graph,
    resource_pot,
    ..
  }: CreateResourcePotMetadataContext<'_>,
) -> Result<ResourcePotMetaData> {
  let resources_map = context.resources_map.lock();

  let rendered_modules = Mutex::new(Vec::with_capacity(modules.len()));
  modules.into_par_iter().try_for_each(|module| {
    let css_stylesheet = source_replace(SourceReplaceContext {
      module,
      module_graph,
      resources_map: &resources_map,
      context,
    })?;

    rendered_modules
      .lock()
      .push((module.id.clone(), css_stylesheet));

    Ok::<(), CompilationError>(())
  })?;

  let mut rendered_modules = rendered_modules.into_inner();

  rendered_modules.sort_by_key(|module| module_execution_order[&module.0]);

  let mut stylesheet = Stylesheet {
    span: DUMMY_SP,
    rules: vec![],
  };

  let source_map = merge_css_sourcemap(&mut rendered_modules, context);
  context
    .meta
    .set_resource_pot_source_map(&resource_pot.id, source_map);

  for (_, rendered_module_ast) in rendered_modules {
    stylesheet.rules.extend(rendered_module_ast.rules);
  }

  Ok(ResourcePotMetaData::Css(CssResourcePotMetaData {
    ast: stylesheet,
    custom: Default::default(),
  }))
}

// pub fn prefixer() {

// }

pub fn codegen(
  CodegenContext {
    context,
    resource_pot,
  }: CodegenContext,
) -> Result<(String, Option<String>)> {
  let css_stylesheet = &resource_pot.meta.as_css().ast;
  let source_map_enabled = context.config.sourcemap.enabled(resource_pot.immutable);

  Ok(codegen_css_stylesheet(
    css_stylesheet,
    context.config.minify.enabled(),
    if source_map_enabled {
      Some(context.meta.get_resource_pot_source_map(&resource_pot.id))
    } else {
      None
    },
  ))
}
