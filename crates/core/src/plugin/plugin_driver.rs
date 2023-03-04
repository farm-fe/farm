use std::sync::Arc;

use hashbrown::HashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::{
  Plugin, PluginAnalyzeDepsHookParam, PluginFinalizeModuleHookParam, PluginHookContext,
  PluginLoadHookParam, PluginLoadHookResult, PluginParseHookParam, PluginProcessModuleHookParam,
  PluginResolveHookParam, PluginResolveHookResult, PluginTransformHookParam,
};
use crate::{
  config::Config,
  context::CompilationContext,
  error::Result,
  module::{
    module_graph::ModuleGraph,
    module_group::{ModuleGroup, ModuleGroupGraph},
    ModuleMetaData, ModuleType,
  },
  resource::{resource_pot::ResourcePot, resource_pot_map::ResourcePotMap, Resource},
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
  pub fn new(mut plugins: Vec<Arc<dyn Plugin>>) -> Self {
    plugins.sort_by_key(|b| std::cmp::Reverse(b.priority()));

    Self { plugins }
  }

  pub fn config(&self, config: &mut Config) -> Result<()> {
    for plugin in &self.plugins {
      plugin.config(config)?;
    }
    Ok(())
  }

  hook_parallel!(build_start);

  hook_first!(
    resolve,
    Result<Option<PluginResolveHookResult>>,
    param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  hook_first!(
    load,
    Result<Option<PluginLoadHookResult>>,
    param: &PluginLoadHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  pub fn transform(
    &self,
    mut param: PluginTransformHookParam<'_>,
    context: &Arc<CompilationContext>,
  ) -> Result<PluginDriverTransformHookResult> {
    let mut result = PluginDriverTransformHookResult {
      content: String::new(),
      source_map_chain: vec![],
      module_type: None,
    };

    for plugin in &self.plugins {
      // if the transform hook returns None, treat it as empty hook and ignore it
      if let Some(plugin_result) = plugin.transform(&param, context)? {
        param.content = plugin_result.content;
        param.module_type = plugin_result.module_type.unwrap_or(param.module_type);

        if let Some(source_map) = plugin_result.source_map {
          result.source_map_chain.push(source_map);
        }
      }
    }

    result.content = param.content;
    result.module_type = Some(param.module_type);

    Ok(result)
  }

  hook_first!(
    parse,
    Result<Option<ModuleMetaData>>,
    param: &PluginParseHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  hook_serial!(process_module, &mut PluginProcessModuleHookParam);

  hook_serial!(analyze_deps, &mut PluginAnalyzeDepsHookParam);

  hook_serial!(finalize_module, &mut PluginFinalizeModuleHookParam);

  hook_parallel!(build_end);

  hook_parallel!(generate_start);

  hook_serial!(optimize_module_graph, &mut ModuleGraph);

  hook_first!(
    analyze_module_graph,
    Result<Option<ModuleGroupGraph>>,
    param: &mut ModuleGraph,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  hook_first!(
    partial_bundling,
    Result<Option<Vec<ResourcePot>>>,
    module_group: &mut ModuleGroup,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  hook_serial!(process_resource_pot_map, &mut ResourcePotMap);

  hook_serial!(render_resource_pot, &mut ResourcePot);

  hook_serial!(optimize_resource_pot, &mut ResourcePot);

  hook_first!(
    generate_resources,
    Result<Option<Vec<Resource>>>,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  hook_serial!(finalize_resources, &mut HashMap<String, Resource>);

  hook_parallel!(generate_end);

  hook_parallel!(finish, stat: &Stats);
}

#[derive(Debug)]
pub struct PluginDriverTransformHookResult {
  pub content: String,
  pub source_map_chain: Vec<String>,
  pub module_type: Option<ModuleType>,
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::sync::Arc;

  use crate::{
    config::Config,
    context::CompilationContext,
    error::Result,
    plugin::{
      Plugin, PluginHookContext, PluginResolveHookParam, PluginResolveHookResult, ResolveKind,
    },
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
          _hook_context: &PluginHookContext,
        ) -> Result<Option<PluginResolveHookResult>> {
          if $should_return {
            Ok(Some(PluginResolveHookResult {
              resolved_path: stringify!($plugin_name).to_string(),
              external: false,
              side_effects: false,
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
      source: "./any".to_string(),
      kind: ResolveKind::Import,
    };
    let context = Arc::new(CompilationContext::new(Config::default(), vec![]).unwrap());
    let hook_context = PluginHookContext {
      caller: None,
      meta: HashMap::new(),
    };

    let resolved = plugin_driver
      .resolve(&param, &context, &hook_context)
      .unwrap();

    assert!(resolved.is_some());
    assert_eq!(
      resolved.unwrap().resolved_path,
      String::from("ResolvePlugin1")
    );

    // should ignore first Ok(None) of ResolvePlugin1 and return next Ok(Some(..)) of ResolvePlugin2
    define_hook_first_plugin!(ResolvePlugin3, false);
    define_hook_first_plugin!(ResolvePlugin4, true);

    let plugin_driver = PluginDriver::new(vec![
      Arc::new(ResolvePlugin3 {}),
      Arc::new(ResolvePlugin4 {}),
    ]);

    let resolved = plugin_driver
      .resolve(&param, &context, &hook_context)
      .unwrap();

    assert!(resolved.is_some());
    assert_eq!(
      resolved.unwrap().resolved_path,
      String::from("ResolvePlugin4")
    );

    // should return Ok(None)
    define_hook_first_plugin!(ResolvePlugin5, false);

    let plugin_driver = PluginDriver::new(vec![Arc::new(ResolvePlugin5 {})]);

    let resolved = plugin_driver
      .resolve(&param, &context, &hook_context)
      .unwrap();

    assert!(resolved.is_none());
  }

  #[test]
  fn hook_serial() {}

  #[test]
  fn hook_parallel() {}
}
