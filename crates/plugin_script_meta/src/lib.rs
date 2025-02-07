/// A set of Plugins that are used to fill module.meta for the script module.
pub use exports::FarmPluginScriptMetaExports;
pub use features::FarmPluginScriptMetaFeatures;

/// Each module exports a Farm Plugin
mod exports;
mod features;
