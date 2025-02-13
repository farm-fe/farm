use std::sync::Arc;

use farmfe_core::{config::ModuleFormat, context::CompilationContext, module::ModuleId};
use farmfe_toolkit::script::module2cjs::RuntimeCalleeAllocator;

use super::targets::util::ShareBundleRuntimeCalleeAllocator;

///
///
/// ```js
/// // farm.config.js
/// {
///   alias: {
///     "react": "node_modules/react/index.js"
///   }
/// }
/// ```
///
/// ```js
/// // index.js
/// import React from "react";
/// ```
///
/// after ShareBundle import generate
/// ```js
/// import React from "node_modules/react/index.js";
/// ```
///
/// but in non-full ShareBundle render, cannot find it
///

pub struct ShareBundleOptions {
  /// whether to use reference slot
  ///
  /// `true`:
  /// ```js
  /// require("__FARM_BUNDLE_REFERENCE_SLOT__(({bundle_group_id}))")
  /// ```
  ///
  /// `false`:
  /// ```js
  /// require("{bundle_group_id}")
  /// ```
  pub reference_slot: bool,

  /// require("external")
  pub ignore_external_polyfill: bool,

  /// in non-full ShareBundle render, maybe not that transform by config.output.format
  pub format: ModuleFormat,
  /// hash paths other than external
  pub hash_path: bool,

  pub concatenation_module: bool,
  pub allocator: Option<Box<dyn RuntimeCalleeAllocator>>,
}

impl Default for ShareBundleOptions {
  fn default() -> Self {
    Self {
      reference_slot: true,
      ignore_external_polyfill: false,
      hash_path: false,
      format: ModuleFormat::EsModule,
      concatenation_module: false,
      allocator: None,
    }
  }
}

pub struct ShareBundleContext {
  pub options: ShareBundleOptions,
  pub context: Arc<CompilationContext>,
  pub allocator: Box<dyn RuntimeCalleeAllocator>,
}

impl ShareBundleContext {
  pub fn format(&self, module_id: &ModuleId) -> String {
    if self.options.hash_path {
      module_id.id(self.context.config.mode.clone())
    } else {
      module_id.to_string()
    }
  }

  pub fn new(mut options: ShareBundleOptions, context: &Arc<CompilationContext>) -> Self {
    let allocator = options
      .allocator
      .take()
      .unwrap_or_else(|| Box::new(ShareBundleRuntimeCalleeAllocator::new()));

    Self {
      options,
      context: Arc::clone(context),
      allocator,
    }
  }
}
