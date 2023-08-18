use std::sync::Arc;

use hashbrown::HashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::{
  Plugin, PluginAnalyzeDepsHookParam, PluginFinalizeModuleHookParam, PluginHookContext,
  PluginLoadHookParam, PluginLoadHookResult, PluginParseHookParam, PluginProcessModuleHookParam,
  PluginResolveHookParam, PluginResolveHookResult, PluginTransformHookParam,
  PluginUpdateModulesHookParams,
};
use crate::{
  config::Config,
  context::CompilationContext,
  error::Result,
  module::{
    module_graph::ModuleGraph, module_group::ModuleGroupGraph, ModuleId, ModuleMetaData, ModuleType,
  },
  record::{ModuleRecord, ResolveRecord, TransformRecord, AnalyzeDepsRecord},
  resource::{resource_pot::ResourcePot, Resource},
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

macro_rules! hook_first_with_callback {
  (
      $func_name:ident,
      $ret_ty:ty,
      $callback:expr,
      $($arg:ident: $ty:ty),*
  ) => {
      pub fn $func_name(&self, $($arg: $ty),*) -> $ret_ty {
          for plugin in &self.plugins {
              let ret = plugin.$func_name($($arg),*)?;
              if ret.is_some() {
                let plugin_name = plugin.name().to_string();
                $callback(&ret, plugin_name, $($arg),*);
                return Ok(ret);
              }
          }

          Ok(None)
      }
  };
}

macro_rules! hook_serial_with_callback {
  ($func_name:ident, $param_ty:ty, $callback:expr) => {
    pub fn $func_name(&self, param: $param_ty, context: &Arc<CompilationContext>) -> Result<()> {
      for plugin in &self.plugins {
        let ret = plugin.$func_name(param, context)?;
        let plugin_name = plugin.name().to_string();
        if ret.is_some() {
          $callback(plugin_name, param, context);
        }
      }

      Ok(())
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

  hook_first_with_callback!(
    resolve,
    Result<Option<PluginResolveHookResult>>,
    |result: &Option<PluginResolveHookResult>,
     plugin_name: String,
     param: &PluginResolveHookParam,
     context: &Arc<CompilationContext>,
     _hook_context: &PluginHookContext| {
      context.record_manager.add_resolve_record(
        param.source.clone(),
        ResolveRecord {
          name: plugin_name,
          result: result.as_ref().unwrap().resolved_path.clone(),
        },
      );
    },
    param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  hook_first_with_callback!(
    load,
    Result<Option<PluginLoadHookResult>>,
    |_result: &Option<PluginLoadHookResult>,
     plugin_name: String,
     param: &PluginLoadHookParam,
     context: &Arc<CompilationContext>,
     _hook_context: &PluginHookContext| {
      context.record_manager.add_load_record(
        param.resolved_path.to_string(),
        TransformRecord {
          name: plugin_name,
          result: _result.as_ref().unwrap().content.clone(),
          source_maps: None,
        },
      );
    },
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

        let plugin_name = plugin.name().to_string();

        if let Some(source_map) = plugin_result.source_map {
          context.record_manager.add_transform_record(
            param.resolved_path.to_string(),
            TransformRecord {
              name: plugin_name,
              result: param.content.clone(),
              source_maps: Some(source_map.clone()),
            },
          );

          result.source_map_chain.push(source_map);
        } else {
          context.record_manager.add_transform_record(
            param.resolved_path.to_string(),
            TransformRecord {
              name: plugin_name,
              result: param.content.clone(),
              source_maps: None,
            },
          );
        }
      }
    }

    result.content = param.content;
    result.module_type = Some(param.module_type);

    Ok(result)
  }

  hook_first_with_callback!(
    parse,
    Result<Option<ModuleMetaData>>,
    |_result: &Option<ModuleMetaData>,
     plugin_name: String,
     param: &PluginParseHookParam,
     context: &Arc<CompilationContext>,
     _hook_context: &PluginHookContext| {
      context.record_manager.add_parse_record(
        param.resolved_path.to_string(),
        ModuleRecord { name: plugin_name },
      );
    },
    param: &PluginParseHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  hook_serial_with_callback!(
    process_module,
    &mut PluginProcessModuleHookParam,
    |plugin_name: String,
     param: &mut PluginProcessModuleHookParam,
     context: &Arc<CompilationContext>| {
      context.record_manager.add_process_record(
        param.module_id.resolved_path(&context.config.root),
        ModuleRecord { name: plugin_name },
      );
    }
  );

  hook_serial_with_callback!(
    analyze_deps,
    &mut PluginAnalyzeDepsHookParam,
    |plugin_name: String,
     param: &mut PluginAnalyzeDepsHookParam,
     context: &Arc<CompilationContext>| {
      context.record_manager.add_analyze_deps_record(param.module.id.resolved_path(&context.config.root), AnalyzeDepsRecord {
        name: plugin_name,
        deps: param.deps.clone()
      });
    }
  );

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
    modules: &Vec<ModuleId>,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  hook_serial!(process_resource_pots, &mut Vec<&mut ResourcePot>);

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

  hook_serial!(update_modules, &mut PluginUpdateModulesHookParams);
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
              query: vec![],
              meta: HashMap::new(),
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
