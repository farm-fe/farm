use farmfe_core::module::meta_data::script::feature_flag::FeatureFlag;
use farmfe_core::module::module_graph::ModuleGraphEdge;
use farmfe_core::module::ModuleId;
use farmfe_core::swc_ecma_ast::Module as SwcModule;
use farmfe_core::HashSet;

pub struct FeaturesAnalyzer<'a> {
  resolved_deps: &'a Vec<(ModuleId, ModuleGraphEdge)>,
  ast: &'a SwcModule,
}

impl<'a> FeaturesAnalyzer<'a> {
  pub fn new(deps: &'a Vec<(ModuleId, ModuleGraphEdge)>, ast: &'a SwcModule) -> Self {
    Self {
      resolved_deps: deps,
      ast,
    }
  }

  pub fn analyze(&self) -> HashSet<FeatureFlag> {
    let mut feature_flags = HashSet::default();

    // dynamic import
    if self
      .resolved_deps
      .iter()
      .any(|(_, edge)| edge.contains_dynamic_import())
    {
      feature_flags.insert(FeatureFlag::DefaultImport);
    }

    // TODO top level await and other flags

    feature_flags
  }
}
