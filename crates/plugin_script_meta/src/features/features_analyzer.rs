use farmfe_core::module::meta_data::script::feature_flag::FeatureFlag;
use farmfe_core::module::meta_data::script::statement::{
  ExportSpecifierInfo, ImportSpecifierInfo, Statement,
};
use farmfe_core::plugin::{PluginAnalyzeDepsHookResultEntry, ResolveKind};
use farmfe_core::HashSet;

pub struct FeaturesAnalyzer<'a> {
  deps: &'a Vec<PluginAnalyzeDepsHookResultEntry>,
  statements: &'a Vec<Statement>,
}

impl<'a> FeaturesAnalyzer<'a> {
  pub fn new(
    deps: &'a Vec<PluginAnalyzeDepsHookResultEntry>,
    statements: &'a Vec<Statement>,
  ) -> Self {
    Self { deps, statements }
  }

  pub fn analyze(&self) -> HashSet<FeatureFlag> {
    let mut feature_flags = HashSet::default();

    // dynamic import
    if self
      .deps
      .iter()
      .any(|dep| matches!(dep.kind, ResolveKind::DynamicImport))
    {
      feature_flags.insert(FeatureFlag::DynamicImport);
    }

    // top level await
    for stmt in self.statements {
      if stmt.top_level_await {
        feature_flags.insert(FeatureFlag::TopLevelAwait);
      }

      if let Some(import_info) = &stmt.import_info {
        feature_flags.insert(FeatureFlag::ImportStatement);

        for sp in &import_info.specifiers {
          if matches!(sp, ImportSpecifierInfo::Namespace(_)) {
            feature_flags.insert(FeatureFlag::ImportNamespace);
          }

          if matches!(sp, ImportSpecifierInfo::Default(_)) {
            feature_flags.insert(FeatureFlag::ImportDefault);
          }
        }
      }

      if let Some(export_info) = &stmt.export_info {
        feature_flags.insert(FeatureFlag::ExportStatement);

        if export_info.source.is_some() {
          feature_flags.insert(FeatureFlag::ExportFrom);
        }

        for sp in &export_info.specifiers {
          if matches!(sp, ExportSpecifierInfo::Namespace(_)) {
            feature_flags.insert(FeatureFlag::ImportNamespace);
          }

          if matches!(sp, ExportSpecifierInfo::All) {
            feature_flags.insert(FeatureFlag::ExportAll);
          }
        }
      }
    }

    feature_flags
  }
}
