use std::sync::Arc;

use parking_lot::RwLock;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::{
  Plugin, PluginAnalyzeDepsHookParam, PluginLoadHookParam, PluginLoadHookResult,
  PluginParseHookParam, PluginResolveHookParam, PluginResolveHookResult, PluginTransformHookParam,
};
use crate::{
  context::CompilationContext,
  error::Result,
  module::{
    module_graph::ModuleGraph,
    module_group::{ModuleGroup, ModuleGroupMap},
    Module, ModuleType,
  },
  resource::{resource_graph::ResourceGraph, Resource},
  stats::Stats,
};

pub struct PluginDriver {
  plugins: Vec<Arc<dyn Plugin>>,
}

macro_rules! hook_first {
  ($func_name:ident, $ret_ty:ty, $($arg:ident: $ty:ty),*) => {
    pub fn $func_name(&self, $($arg: $ty),*) -> $ret_ty {
      for plugin in &self.plugins {
        let ret = plugin.$func_name($($arg),*)?;

        if ret.is_some() {
          return Ok(ret)
        }
      }

      Ok(None)
    }
  };
}

macro_rules! hook_serial {
  ($func_name:ident, $param_ty:ty) => {
    pub fn $func_name(&self, param: $param_ty, context: &Arc<CompilationContext>) -> Result<()> {
      for plugin in &self.plugins {
        plugin.$func_name(param, context)?;
      }

      Ok(())
    }
  };
}

macro_rules! hook_parallel {
  ($func_name:ident) => {
    pub fn $func_name(&self, context: &Arc<CompilationContext>) -> Result<()> {
      self
        .plugins
        .par_iter()
        .try_for_each(|plugin| plugin.$func_name(context).map(|_| ()))
    }
  };

  ($func_name:ident, $($arg:ident: $ty:ty),+) => {
    pub fn $func_name(&self, $($arg: $ty),+, context: &Arc<CompilationContext>) -> Result<()> {
      self
        .plugins
        .par_iter()
        .try_for_each(|plugin| plugin.$func_name($($arg),+, context).map(|_| ()))
    }
  };
}

impl PluginDriver {
  pub fn new(plugins: Vec<Arc<dyn Plugin>>) -> Self {
    Self { plugins }
  }

  hook_parallel!(build_start);

  hook_first!(
    resolve,
    Result<Option<PluginResolveHookResult>>,
    param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>
  );

  hook_first!(
    load,
    Result<Option<PluginLoadHookResult>>,
    param: &PluginLoadHookParam,
    context: &Arc<CompilationContext>
  );

  pub fn transform(
    &self,
    mut param: PluginTransformHookParam<'_>,
    context: &Arc<CompilationContext>,
  ) -> Result<PluginDriverTransformHookResult> {
    let mut result = PluginDriverTransformHookResult {
      source: String::new(),
      source_map_chain: vec![],
      module_type: None,
    };

    for plugin in &self.plugins {
      // if the transform hook returns None, treat it as empty hook and ignore it
      if let Some(plugin_result) = plugin.transform(&param, context)? {
        param.source = plugin_result.source;
        param.module_type = plugin_result.module_type.unwrap_or(param.module_type);

        if let Some(source_map) = plugin_result.source_map {
          result.source_map_chain.push(source_map);
        }
      }
    }

    result.source = param.source;
    result.module_type = Some(param.module_type);

    Ok(result)
  }

  hook_first!(
    parse,
    Result<Option<Module>>,
    param: &PluginParseHookParam,
    context: &Arc<CompilationContext>
  );

  hook_serial!(module_parsed, &mut Module);

  hook_serial!(analyze_deps, &mut PluginAnalyzeDepsHookParam);

  hook_parallel!(build_end);

  hook_parallel!(generate_start);

  hook_serial!(optimize_module_graph, &RwLock<ModuleGraph>);

  hook_first!(
    analyze_module_graph,
    Result<Option<ModuleGroupMap>>,
    param: &RwLock<ModuleGraph>,
    context: &Arc<CompilationContext>
  );

  hook_first!(
    merge_modules,
    Result<Option<ResourceGraph>>,
    module_group: &ModuleGroupMap,
    context: &Arc<CompilationContext>
  );

  hook_parallel!(
    process_resource_graph,
    resource_graph: &RwLock<ResourceGraph>
  );

  hook_serial!(render_resource, &mut Resource);

  hook_serial!(optimize_resource, &mut Resource);

  hook_serial!(generate_resource, &mut Resource);

  hook_serial!(write_resource, &mut Resource);

  hook_parallel!(generate_end);

  hook_parallel!(finish, stat: &Stats);
}

#[derive(Debug)]
pub struct PluginDriverTransformHookResult {
  source: String,
  source_map_chain: Vec<String>,
  module_type: Option<ModuleType>,
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
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
        fn name(&self) -> &str {
          stringify!($plugin_name)
        }

        fn resolve(
          &self,
          _param: &PluginResolveHookParam,
          _context: &Arc<CompilationContext>,
        ) -> Result<Option<PluginResolveHookResult>> {
          if $should_return {
            Ok(Some(PluginResolveHookResult {
              id: stringify!($plugin_name).to_string(),
              external: false,
              side_effects: false,
              package_json_info: None,
              query: HashMap::new(),
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
      caller: None,
    };
    let context = Arc::new(CompilationContext::new(Config::default(), vec![]));

    let resolved = plugin_driver.resolve(&param, &context).unwrap();

    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().id, String::from("ResolvePlugin1"));

    // should ignore first Ok(None) of ResolvePlugin1 and return next Ok(Some(..)) of ResolvePlugin2
    define_hook_first_plugin!(ResolvePlugin3, false);
    define_hook_first_plugin!(ResolvePlugin4, true);

    let plugin_driver = PluginDriver::new(vec![
      Arc::new(ResolvePlugin3 {}),
      Arc::new(ResolvePlugin4 {}),
    ]);

    let resolved = plugin_driver.resolve(&param, &context).unwrap();

    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().id, String::from("ResolvePlugin4"));

    // should return Ok(None)
    define_hook_first_plugin!(ResolvePlugin5, false);

    let plugin_driver = PluginDriver::new(vec![Arc::new(ResolvePlugin5 {})]);

    let resolved = plugin_driver.resolve(&param, &context).unwrap();

    assert!(resolved.is_none());
  }

  #[test]
  fn hook_serial() {}

  #[test]
  fn hook_parallel() {}
}
