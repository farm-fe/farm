use farmfe_core::HashSet;
use farmfe_core::module::meta_data::script::feature_flag::FeatureFlag;
use farmfe_core::module::meta_data::script::statement::{
  ExportSpecifierInfo, ImportSpecifierInfo, Statement,
};
use farmfe_core::plugin::{PluginAnalyzeDepsHookResultEntry, ResolveKind};

pub struct FeaturesAnalyzer<'a> {
  deps: &'a Vec<PluginAnalyzeDepsHookResultEntry>,
  statements: &'a Vec<Statement>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use farmfe_core::module::meta_data::script::statement::{ExportInfo, SwcId};

  #[test]
  fn default_reexports_enable_the_helpers_used_by_lowering() {
    for alias in [None, Some(SwcId::new("Alias".into(), 0))] {
      let statements = vec![Statement::new(
        0,
        Some(ExportInfo {
          source: Some("./value.js".into()),
          specifiers: vec![ExportSpecifierInfo::Named {
            local: SwcId::new("default".into(), 0),
            exported: alias,
          }],
          stmt_id: 0,
        }),
        None,
        HashSet::default(),
        false,
      )];
      let flags = FeaturesAnalyzer::new(&vec![], &statements).analyze();
      assert!(flags.contains(&FeatureFlag::ImportDefault));
    }
    let statements = vec![Statement::new(
      0,
      Some(ExportInfo {
        source: Some("./value.js".into()),
        specifiers: vec![
          ExportSpecifierInfo::Named {
            local: SwcId::new("default".into(), 0),
            exported: None,
          },
          ExportSpecifierInfo::Named {
            local: SwcId::new("named".into(), 0),
            exported: None,
          },
        ],
        stmt_id: 0,
      }),
      None,
      HashSet::default(),
      false,
    )];
    let flags = FeaturesAnalyzer::new(&vec![], &statements).analyze();
    assert!(flags.contains(&FeatureFlag::ImportNamespace));
  }
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

          if matches!(sp, ImportSpecifierInfo::Named { .. }) {
            feature_flags.insert(FeatureFlag::ImportNamed);
          }
        }
      }

      if let Some(export_info) = &stmt.export_info {
        feature_flags.insert(FeatureFlag::ExportStatement);

        if export_info.source.is_some() {
          feature_flags.insert(FeatureFlag::ExportFrom);

          let reexports_default = export_info.specifiers.iter().any(|specifier| {
            matches!(specifier, ExportSpecifierInfo::Named { local, .. } if local.sym == "default")
          });
          if reexports_default {
            // module2cjs wraps default-only reexports with the default helper,
            // and mixed default/named reexports with the namespace helper.
            let reexports_named = export_info.specifiers.iter().any(|specifier| {
              matches!(specifier, ExportSpecifierInfo::Named { local, .. } if local.sym != "default")
            });
            feature_flags.insert(if reexports_named {
              FeatureFlag::ImportNamespace
            } else {
              FeatureFlag::ImportDefault
            });
          }
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
