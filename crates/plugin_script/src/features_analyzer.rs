use farmfe_core::module::meta_data::script::feature_flag::FeatureFlag;
use farmfe_core::plugin::{PluginAnalyzeDepsHookResultEntry, ResolveKind};
use farmfe_core::swc_ecma_ast::Module as SwcModule;
use farmfe_core::HashSet;

pub struct FeaturesAnalyzer<'a> {
  deps: &'a Vec<PluginAnalyzeDepsHookResultEntry>,
  ast: &'a SwcModule,
}

impl<'a> FeaturesAnalyzer<'a> {
  pub fn new(deps: &'a Vec<PluginAnalyzeDepsHookResultEntry>, ast: &'a SwcModule) -> Self {
    Self { deps, ast }
  }

  pub fn analyze(&self) -> HashSet<FeatureFlag> {
    let mut feature_flags = HashSet::default();

    // dynamic import
    if self
      .deps
      .iter()
      .any(|dep| matches!(dep.kind, ResolveKind::DynamicImport))
    {
      feature_flags.insert(FeatureFlag::DefaultImport);
    }

    // TODO top level await and other flags

    feature_flags
  }
}
