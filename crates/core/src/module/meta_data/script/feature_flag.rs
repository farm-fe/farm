use farmfe_macro_cache_item::cache_item;

pub const FARM_ENABLE_TOP_LEVEL_AWAIT: &str = "__FARM_ENABLE_TOP_LEVEL_AWAIT__";
pub const FARM_ENABLE_EXPORT_HELPER: &str = "__FARM_ENABLE_EXPORT_HELPER__";
pub const FARM_ENABLE_EXPORT_ALL_HELPER: &str = "__FARM_ENABLE_EXPORT_ALL_HELPER__";
pub const FARM_ENABLE_IMPORT_ALL_HELPER: &str = "__FARM_ENABLE_IMPORT_ALL_HELPER__";
pub const FARM_IMPORT_EXPORT_FROM_HELPER: &str = "__FARM_IMPORT_EXPORT_FROM_HELPER__";
pub const FARM_ENABLE_IMPORT_DEFAULT_HELPER: &str = "__FARM_ENABLE_IMPORT_DEFAULT_HELPER__";

/// Features that used in a script module
#[cache_item]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[archive_attr(derive(Hash, Eq, PartialEq))]
pub enum FeatureFlag {
  /// import('xxx')
  DynamicImport,
  /// use `await xxx` in top level scope
  TopLevelAwait,
  /// import xxx from 'xxx'
  ImportDefault,
  /// import * as xxx from 'xxx'
  ImportNamespace,
  /// import { xxx } from 'xxx'
  ImportNamed,
  /// export * from 'xxx'
  ExportAll,
  ExportFrom,
  ExportStatement,
  ImportStatement,
}

impl FeatureFlag {
  pub fn as_str(&self) -> &str {
    match self {
      FeatureFlag::DynamicImport => "DynamicImport",
      FeatureFlag::TopLevelAwait => FARM_ENABLE_TOP_LEVEL_AWAIT,
      FeatureFlag::ImportDefault => FARM_ENABLE_IMPORT_DEFAULT_HELPER,
      FeatureFlag::ImportNamespace => FARM_ENABLE_IMPORT_ALL_HELPER,
      FeatureFlag::ExportAll => FARM_ENABLE_EXPORT_ALL_HELPER,
      FeatureFlag::ExportFrom => FARM_IMPORT_EXPORT_FROM_HELPER,
      FeatureFlag::ExportStatement => FARM_ENABLE_EXPORT_HELPER,
      FeatureFlag::ImportStatement => "ImportStatement",
      FeatureFlag::ImportNamed => "ImportStatement",
    }
  }
}
