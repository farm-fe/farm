use std::sync::Arc;

use super::{Plugin, PluginResolveHookParam, PluginResolveHookResult};
use crate::{context::CompilationContext, error::Result};

pub struct PluginDriver {
  plugins: Vec<Arc<dyn Plugin + Send + Sync>>,
}

macro_rules! hook_first {
  ($func_name:ident, $ret_ty:ty, $($arg:ident: $ty:ty),*) => {
    pub fn $func_name(&self, $($arg: $ty),*) -> $ret_ty {
      for plugin in &self.plugins {
        let ret = plugin.resolve($($arg),*)?;

        if ret.is_some() {
          return Ok(ret)
        }
      }

      Ok(None)
    }
  };
}

impl PluginDriver {
  pub fn new(plugins: Vec<Arc<dyn Plugin + Send + Sync>>) -> Self {
    Self { plugins }
  }

  hook_first!(
    call_resolve_hook,
    Result<Option<PluginResolveHookResult>>,
    param: &PluginResolveHookParam,
    context: &CompilationContext
  );
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use crate::{
    config::Config,
    context::CompilationContext,
    error::Result,
    plugin::{Plugin, PluginResolveHookParam, PluginResolveHookResult, ResolveKind},
  };

  use super::PluginDriver;

  macro_rules! define_hook_first_plugin {
    ($plugin_name:ident, $should_return:expr) => {
      struct $plugin_name {}

      impl Plugin for $plugin_name {
        fn name(&self) -> String {
          stringify!($plugin_name).to_string()
        }

        fn resolve(
          &self,
          _param: &PluginResolveHookParam,
          _context: &CompilationContext,
        ) -> Result<Option<PluginResolveHookResult>> {
          if $should_return {
            Ok(Some(PluginResolveHookResult {
              id: stringify!($plugin_name).to_string(),
              external: false,
              side_effects: false,
            }))
          } else {
            Ok(None)
          }
        }
      }
    };
  }

  #[test]
  fn hook_first() {
    define_hook_first_plugin!(ResolvePlugin1, true);
    define_hook_first_plugin!(ResolvePlugin2, true);

    // should execute in serial order and return as soon as the plugin does not return Ok(None)
    let plugin_driver = PluginDriver::new(vec![
      Arc::new(ResolvePlugin1 {}),
      Arc::new(ResolvePlugin2 {}),
    ]);

    let param = PluginResolveHookParam {
      importer: None,
      specifier: "./any".to_string(),
      kind: ResolveKind::Import,
    };
    let context = CompilationContext::new(Config::default(), vec![]);

    let resolved = plugin_driver.call_resolve_hook(&param, &context).unwrap();

    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().id, String::from("ResolvePlugin1"));

    // should ignore first Ok(None) of ResolvePlugin1 and return next Ok(Some(..)) of ResolvePlugin2
    define_hook_first_plugin!(ResolvePlugin3, false);
    define_hook_first_plugin!(ResolvePlugin4, true);

    let plugin_driver = PluginDriver::new(vec![
      Arc::new(ResolvePlugin3 {}),
      Arc::new(ResolvePlugin4 {}),
    ]);

    let resolved = plugin_driver.call_resolve_hook(&param, &context).unwrap();

    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().id, String::from("ResolvePlugin4"));

    // should return Ok(None)
    define_hook_first_plugin!(ResolvePlugin5, false);

    let plugin_driver = PluginDriver::new(vec![Arc::new(ResolvePlugin5 {})]);

    let resolved = plugin_driver.call_resolve_hook(&param, &context).unwrap();

    assert!(resolved.is_none());
  }
}
