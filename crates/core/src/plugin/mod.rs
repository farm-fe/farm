use crate::{context::CompilationContext, error::Result, module::ModuleType};

pub mod plugin_driver;

pub struct PluginResolveHookResult {
  pub id: String,
  pub external: bool,
  pub side_effects: bool,
}

pub enum ResolveKind {
  /// entry input in the config
  Entry,
  /// static import, e.g. `import a from './a'`
  Import,
  /// dynamic import, e.g. `import('./a').then(module => console.log(module))`
  DynamicImport,
  /// cjs require, e.g. `require('./a')`
  Require,
  /// @import of css, e.g. @import './a.css'
  AtImport,
  /// url() of css, e.g. url('./a.png')
  Url,
  /// `<script src="./index.html" />` of html
  ScriptSrc,
  /// `<link href="index.css" />` of html
  LinkHref,
}

pub struct PluginResolveHookParam {
  pub specifier: String,
  pub importer: Option<String>,
  pub kind: ResolveKind,
}

pub trait Plugin {
  fn name(&self) -> String;

  fn should_execute_hook(&self, _param: &FilteringHookParam<'_>) -> bool {
    true
  }

  fn priority(&self) -> Result<Option<usize>> {
    Ok(None)
  }

  fn resolve(
    &self,
    _param: &PluginResolveHookParam,
    _context: &CompilationContext,
  ) -> Result<Option<PluginResolveHookResult>> {
    Ok(None)
  }
}

pub enum FilteringHookName {
  Resolve,
  Load,
  Transform,
  Parse,
  Unknown,
}

impl Default for FilteringHookName {
  fn default() -> Self {
    Self::Unknown
  }
}

#[derive(Default)]
pub struct FilteringHookParam<'a> {
  pub hook_name: FilteringHookName,
  pub specifier: Option<&'a str>,
  pub importer: Option<&'a str>,
  pub id: Option<&'a str>,
  pub module_type: Option<ModuleType>,
}
