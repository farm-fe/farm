use farmfe_core::{
  config::Config,
  module::meta_data::script::statement::Statement,
  plugin::{hooks::freeze_module::PluginFreezeModuleHookParam, Plugin},
  swc_common::Mark,
};
use farmfe_toolkit::{
  script::{
    analyze_statement::{analyze_statement_info, AnalyzedStatementInfo},
    concatenate_modules::expand_exports_of_module_graph,
    idents_collector::UnresolvedIdentCollector,
    swc_try_with::try_with,
  },
  swc_ecma_visit::VisitWith,
};
use features_analyzer::FeaturesAnalyzer;

mod features_analyzer;

pub struct FarmPluginScriptMeta {}

impl FarmPluginScriptMeta {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginScriptMeta {
  fn name(&self) -> &str {
    "FarmPluginScriptMeta"
  }

  /// This plugin should executed at last
  fn priority(&self) -> i32 {
    -99
  }

  fn freeze_module(
    &self,
    param: &mut PluginFreezeModuleHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if !param.module.module_type.is_script() {
      return Ok(None);
    }

    // 1. analyze features used
    let features_analyzer =
      FeaturesAnalyzer::new(&param.resolved_deps, &param.module.meta.as_script().ast);
    param.module.meta.as_script_mut().feature_flags = features_analyzer.analyze();

    let module = &mut param.module;

    // collect statements information, top level idents, unresolved_idents from the ast
    let unresolved_mark = Mark::from_u32(module.meta.as_script().unresolved_mark);

    // 2. fill statements
    let ast = &mut module.meta.as_script_mut().ast;
    let mut statements = vec![];

    for (i, item) in ast.body.iter_mut().enumerate() {
      let AnalyzedStatementInfo {
        import_info,
        export_info,
        defined_idents,
      } = analyze_statement_info(&i, item);
      let stmt = Statement::new(i, export_info, import_info, defined_idents);
      statements.push(stmt);
    }

    // 3. fill unresolved_idents
    let mut unresolved_ident_collector = UnresolvedIdentCollector::new(unresolved_mark);
    let (cm, _) = context
      .meta
      .create_swc_source_map(&module.id, module.content.clone());
    try_with(cm, &context.meta.script.globals, || {
      ast.visit_with(&mut unresolved_ident_collector);
    })
    .unwrap();

    module.meta.as_script_mut().unresolved_idents = unresolved_ident_collector.unresolved_idents;

    // 4. for top level idents, merge defined_idents in each statement
    let top_level_idents = statements
      .iter()
      .filter(|s| s.import_info.is_none())
      .flat_map(|s| s.defined_idents.clone());

    module.meta.as_script_mut().top_level_idents = top_level_idents.collect();
    module.meta.as_script_mut().statements = statements;

    Ok(Some(()))
  }

  /// Must be executed after tree shake
  fn optimize_module_graph(
    &self,
    module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    expand_exports_of_module_graph(module_graph, context);

    Ok(Some(()))
  }
}
