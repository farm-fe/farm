use std::sync::Arc;

use farmfe_utils::stringify_query;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::{
  Plugin, PluginAnalyzeDepsHookParam, PluginDriverRenderResourcePotHookResult,
  PluginFinalizeModuleHookParam, PluginFinalizeResourcesHookParams,
  PluginGenerateResourcesHookResult, PluginHandleEntryResourceHookParams, PluginHookContext,
  PluginLoadHookParam, PluginLoadHookResult, PluginModuleGraphUpdatedHookParams,
  PluginParseHookParam, PluginProcessModuleHookParam, PluginRenderResourcePotHookParam,
  PluginResolveHookParam, PluginResolveHookResult, PluginTransformHookParam,
  PluginUpdateModulesHookParams,
};
use crate::{
  config::Config,
  context::CompilationContext,
  error::Result,
  module::{
    module_graph::ModuleGraph, module_group::ModuleGroupGraph, Module, ModuleId, ModuleMetaData,
    ModuleType,
  },
  record::{
    AnalyzeDepsRecord, ModuleRecord, ResolveRecord, ResourcePotRecord, TransformRecord, Trigger,
  },
  resource::resource_pot::{ResourcePot, ResourcePotInfo, ResourcePotMetaData},
  stats::Stats,
};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub struct PluginDriver {
  plugins: Vec<Arc<dyn Plugin>>,
  record: bool,
}

macro_rules! hook_first {
  (
    $func_name:ident,
    $ret_ty:ty,
    $($arg:ident: $ty:ty),*
  ) => {
      pub fn $func_name(&self, $($arg: $ty),*) -> $ret_ty {
          for plugin in &self.plugins {
              let ret = plugin.$func_name($($arg),*)?;
              if ret.is_some() {
                return Ok(ret);
              }
          }

          Ok(None)
      }
  };

  (
      $func_name:ident,
      $ret_ty:ty,
      $callback:expr,
      $($arg:ident: $ty:ty),*
  ) => {
      pub fn $func_name(&self, $($arg: $ty),*) -> $ret_ty {
          for plugin in &self.plugins {
            if self.record {
              let start_time = SystemTime::now()
              .duration_since(UNIX_EPOCH)
              .expect("Time went backwards")
              .as_micros() as i64;
              let ret = plugin.$func_name($($arg),*)?;
              let end_time = SystemTime::now()
              .duration_since(UNIX_EPOCH)
              .expect("hook_first get end_time failed")
              .as_micros() as i64;
              if ret.is_some() {
                  let plugin_name = plugin.name().to_string();
                  $callback(&ret, plugin_name, start_time, end_time, $($arg),*);
                  return Ok(ret);
              }
            }else {
              let ret = plugin.$func_name($($arg),*)?;
              if ret.is_some() {
                return Ok(ret);
              }
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

  ($func_name:ident, $param_ty:ty, $callback:expr) => {
    pub fn $func_name(&self, param: $param_ty, context: &Arc<CompilationContext>) -> Result<()> {
      for plugin in &self.plugins {
        if self.record {
          let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_micros() as i64;
          let ret = plugin.$func_name(param, context)?;
          let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("hook_first get end_time failed")
            .as_micros() as i64;
          if ret.is_some() {
            let plugin_name = plugin.name().to_string();
            $callback(plugin_name, start_time, end_time, param, context);
          }
        } else {
          plugin.$func_name(param, context)?;
        }
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
        .try_for_each(|plugin| {
          let ret = plugin.$func_name(context).map(|_| ());
          return ret;
        })
    }
  };

  ($func_name:ident, $callback:expr) => {
    pub fn $func_name(&self, context: &Arc<CompilationContext>) -> Result<()> {
      self
        .plugins
        .par_iter()
        .try_for_each(|plugin| {
          let ret = plugin.$func_name(context).map(|_| ());
          if self.record {
            let plugin_name = plugin.name().to_string();
            $callback(plugin_name, context);
          }
          return ret;
        })
    }
  };

  ($func_name:ident, $($arg:ident: $ty:ty),+) => {
    pub fn $func_name(&self, $($arg: $ty),+, context: &Arc<CompilationContext>) -> Result<()> {
      self
        .plugins
        .par_iter()
        .try_for_each(|plugin| {
          let ret = plugin.$func_name($($arg),+, context).map(|_| ());
          return ret;
        })
    }
  };

  ($func_name:ident, $($arg:ident: $ty:ty),+ ,$callback:expr) => {
    pub fn $func_name(&self, $($arg: $ty),+, context: &Arc<CompilationContext>) -> Result<()> {
      self
        .plugins
        .par_iter()
        .try_for_each(|plugin| {
          let ret = plugin.$func_name($($arg),+, context).map(|_| ());
          if self.record {
            let plugin_name = plugin.name().to_string();
            $callback(plugin_name, context);
          }
          return ret;
        })
    }
  };
}

impl PluginDriver {
  pub fn new(mut plugins: Vec<Arc<dyn Plugin>>, record: bool) -> Self {
    plugins.sort_by_key(|b| std::cmp::Reverse(b.priority()));

    Self { plugins, record }
  }

  pub fn config(&self, config: &mut Config) -> Result<()> {
    for plugin in &self.plugins {
      plugin.config(config)?;
    }
    Ok(())
  }

  pub fn plugin_cache_loaded(&self, context: &Arc<CompilationContext>) -> Result<()> {
    for plugin in &self.plugins {
      if let Some(plugin_cache) = context.cache_manager.plugin_cache.read_cache(plugin.name()) {
        plugin.plugin_cache_loaded(plugin_cache.value(), context)?;
      }
    }

    Ok(())
  }

  hook_parallel!(build_start);

  hook_first!(
    resolve,
    Result<Option<PluginResolveHookResult>>,
    |result: &Option<PluginResolveHookResult>,
     plugin_name: String,
     start_time: i64,
     end_time: i64,
     param: &PluginResolveHookParam,
     context: &Arc<CompilationContext>,
     _hook_context: &PluginHookContext| {
      match result {
        Some(resolve_result) => {
          context.record_manager.add_resolve_record(
            resolve_result.resolved_path.clone() + stringify_query(&resolve_result.query).as_str(),
            ResolveRecord {
              start_time,
              end_time,
              duration: end_time - start_time,
              plugin: plugin_name,
              hook: "resolve".to_string(),
              source: param.source.clone(),
              importer: param
                .importer
                .clone()
                .map(|module_id| module_id.relative_path().to_string()),
              kind: String::from(param.kind.clone()),
              trigger: Trigger::Compiler,
            },
          );
        }
        None => {}
      }
    },
    param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  hook_first!(
    load,
    Result<Option<PluginLoadHookResult>>,
    |result: &Option<PluginLoadHookResult>,
     plugin_name: String,
     start_time: i64,
     end_time: i64,
     param: &PluginLoadHookParam,
     context: &Arc<CompilationContext>,
     _hook_context: &PluginHookContext| {
      match result {
        Some(load_result) => {
          context.record_manager.add_load_record(
            param.resolved_path.to_string() + stringify_query(&param.query).as_str(),
            TransformRecord {
              plugin: plugin_name,
              hook: "load".to_string(),
              content: load_result.content.clone(),
              source_maps: None,
              module_type: load_result.module_type.clone(),
              trigger: Trigger::Compiler,
              start_time,
              end_time,
              duration: end_time - start_time,
            },
          );
        }
        None => {}
      }
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
      source_map_chain: param.source_map_chain.clone(),
      module_type: None,
    };

    for plugin in &self.plugins {
      let start_time = if self.record {
        Some(
          SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_micros() as i64,
        )
      } else {
        None
      };
      // if the transform hook returns None, treat it as empty hook and ignore it
      if let Some(plugin_result) = plugin.transform(&param, context)? {
        let end_time = if self.record {
          Some(
            SystemTime::now()
              .duration_since(UNIX_EPOCH)
              .expect("hook_first get end_time failed")
              .as_micros() as i64,
          )
        } else {
          None
        };
        param.content = plugin_result.content;
        param.module_type = plugin_result.module_type.unwrap_or(param.module_type);

        if plugin_result.ignore_previous_source_map {
          result.source_map_chain.clear();
        }

        if self.record {
          let plugin_name = plugin.name().to_string();

          context.record_manager.add_transform_record(
            param.resolved_path.to_string() + stringify_query(&param.query).as_str(),
            TransformRecord {
              plugin: plugin_name,
              hook: "transform".to_string(),
              content: param.content.clone(),
              source_maps: plugin_result.source_map.clone(),
              module_type: param.module_type.clone(),
              trigger: Trigger::Compiler,
              start_time: start_time.unwrap(),
              end_time: end_time.unwrap(),
              duration: end_time.unwrap() - start_time.unwrap(),
            },
          );
        }

        if let Some(source_map) = plugin_result.source_map {
          let sourcemap = Arc::new(source_map);
          result.source_map_chain.push(sourcemap.clone());
          param.source_map_chain.push(sourcemap);
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
    |_result: &Option<ModuleMetaData>,
     plugin_name: String,
     start_time: i64,
     end_time: i64,
     param: &PluginParseHookParam,
     context: &Arc<CompilationContext>,
     _hook_context: &PluginHookContext| {
      context.record_manager.add_parse_record(
        param.resolved_path.to_string() + stringify_query(&param.query).as_str(),
        ModuleRecord {
          plugin: plugin_name,
          hook: "parse".to_string(),
          module_type: param.module_type.clone(),
          trigger: Trigger::Compiler,
          start_time,
          end_time,
          duration: end_time - start_time,
        },
      );
    },
    param: &PluginParseHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  hook_serial!(
    process_module,
    &mut PluginProcessModuleHookParam,
    |plugin_name: String,
     start_time: i64,
     end_time: i64,
     param: &mut PluginProcessModuleHookParam,
     context: &Arc<CompilationContext>| {
      context.record_manager.add_process_record(
        param.module_id.resolved_path(&context.config.root) + param.module_id.query_string(),
        ModuleRecord {
          plugin: plugin_name,
          hook: "process".to_string(),
          module_type: param.module_type.clone(),
          trigger: Trigger::Compiler,
          start_time,
          end_time,
          duration: end_time - start_time,
        },
      );
    }
  );

  hook_serial!(
    analyze_deps,
    &mut PluginAnalyzeDepsHookParam,
    |plugin_name: String,
     start_time: i64,
     end_time: i64,
     param: &mut PluginAnalyzeDepsHookParam,
     context: &Arc<CompilationContext>| {
      context.record_manager.add_analyze_deps_record(
        param.module.id.resolved_path(&context.config.root) + param.module.id.query_string(),
        AnalyzeDepsRecord {
          plugin: plugin_name,
          hook: "analyze_deps".to_string(),
          module_type: param.module.module_type.clone(),
          trigger: Trigger::Compiler,
          deps: param.deps.clone(),
          start_time,
          end_time,
          duration: end_time - start_time,
        },
      );
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
    |result: &Option<Vec<ResourcePot>>,
     plugin_name: String,
     start_time: i64,
     end_time: i64,
     _modules: &Vec<ModuleId>,
     context: &Arc<CompilationContext>,
     _hook_context: &PluginHookContext| {
      context.record_manager.update_plugin_stats(plugin_name.clone(), "partial_bundling", end_time - start_time);
      match result {
        Some(resource_pots) => {
          for resource_pot in resource_pots.iter() {
            context.record_manager.add_resource_pot_record(
              resource_pot.id.to_string(),
              ResourcePotRecord {
                name: plugin_name.clone(),
                hook: "partial_bundling".to_string(),
                modules: resource_pot.modules().into_iter().cloned().collect(),
                resources: vec![],
              },
            )
          }
        }
        None => {}
      }
    },
    modules: &Vec<ModuleId>,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  hook_serial!(
    process_resource_pots,
    &mut Vec<&mut ResourcePot>,
    |plugin_name: String,
     start_time: i64,
     end_time: i64,
     resource_pots: &mut Vec<&mut ResourcePot>,
     context: &Arc<CompilationContext>| {
      context.record_manager.update_plugin_stats(
        plugin_name.clone(),
        "process_resource_pots",
        end_time - start_time,
      );
      for resource_pot in resource_pots.iter() {
        context.record_manager.add_resource_pot_record(
          resource_pot.id.to_string(),
          ResourcePotRecord {
            name: plugin_name.clone(),
            hook: "process_resource_pots".to_string(),
            modules: resource_pot.modules().into_iter().cloned().collect(),
            resources: vec![],
          },
        )
      }
    }
  );

  hook_serial!(render_start, &Config);

  hook_first!(
    render_resource_pot_modules,
    Result<Option<ResourcePotMetaData>>,
    resource_pot: &ResourcePot,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  pub fn render_resource_pot(
    &self,
    param: &mut PluginRenderResourcePotHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<PluginDriverRenderResourcePotHookResult> {
    for plugin in &self.plugins {
      // if the transform hook returns None, treat it as empty hook and ignore it
      if let Some(plugin_result) = plugin.render_resource_pot(param, context)? {
        param.content = Arc::new(plugin_result.content);

        if let Some(source_map) = plugin_result.source_map {
          param.source_map_chain.push(Arc::new(source_map));
        }
      }
    }

    let result = PluginDriverRenderResourcePotHookResult {
      content: param.content.clone(),
      source_map_chain: param.source_map_chain.clone(),
    };

    Ok(result)
  }

  pub fn augment_resource_hash(
    &self,
    render_pot_info: &ResourcePotInfo,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<String>> {
    let mut result: Option<String> = None;

    for plugin in &self.plugins {
      if let Some(plugin_result) = plugin.augment_resource_hash(render_pot_info, context)? {
        match result {
          Some(ref mut result) => {
            result.push_str(plugin_result.as_str());
          }
          None => {
            result = Some(plugin_result);
          }
        }
      }
    }

    Ok(result)
  }

  hook_serial!(
    optimize_resource_pot,
    &mut ResourcePot,
    |plugin_name: String,
     _start_time: i64,
     _end_time: i64,
     resource_pot: &mut ResourcePot,
     context: &Arc<CompilationContext>| {
      context.record_manager.add_resource_pot_record(
        resource_pot.id.to_string(),
        ResourcePotRecord {
          name: plugin_name,
          hook: "optimize_resource_pot".to_string(),
          modules: resource_pot.modules().into_iter().cloned().collect(),
          resources: vec![],
        },
      );
    }
  );

  hook_first!(
    generate_resources,
    Result<Option<PluginGenerateResourcesHookResult>>,
    |result: &Option<PluginGenerateResourcesHookResult>,
     plugin_name: String,
     _start_time: i64,
     _end_time: i64,
     resource_pot: &mut ResourcePot,
     context: &Arc<CompilationContext>,
     _hook_context: &PluginHookContext| {
      match result {
        Some(resources) => {
          context.record_manager.add_resource_pot_record(
            resource_pot.id.to_string(),
            ResourcePotRecord {
              name: plugin_name,
              hook: "generate_resources".to_string(),
              modules: resource_pot.modules().into_iter().cloned().collect(),
              resources: vec![
                resources.resource.name.clone(),
                resources
                  .source_map
                  .as_ref()
                  .map_or(String::new(), |r| r.name.clone()),
              ]
              .into_iter()
              .filter(|r| !r.is_empty())
              .collect(),
            },
          );
        }
        None => {}
      };
    },
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  hook_serial!(
    process_generated_resources,
    &mut PluginGenerateResourcesHookResult
  );

  hook_serial!(
    handle_entry_resource,
    &mut PluginHandleEntryResourceHookParams
  );

  hook_serial!(finalize_resources, &mut PluginFinalizeResourcesHookParams);

  hook_parallel!(generate_end);

  hook_parallel!(
    finish,
    stat: &Stats,
    |_plugin_name: String, context: &Arc<CompilationContext>| {
      context.record_manager.set_trigger(Trigger::Update);
    }
  );

  hook_serial!(update_modules, &mut PluginUpdateModulesHookParams);

  hook_parallel!(
    module_graph_updated,
    param: &PluginModuleGraphUpdatedHookParams
  );

  hook_parallel!(update_finished);

  hook_first!(
    handle_persistent_cached_module,
    Result<Option<bool>>,
    module: &Module,
    context: &Arc<CompilationContext>
  );

  pub fn write_plugin_cache(&self, context: &Arc<CompilationContext>) -> Result<()> {
    for plugin in &self.plugins {
      let plugin_cache = plugin.write_plugin_cache(context)?;

      if let Some(plugin_cache) = plugin_cache {
        context
          .cache_manager
          .plugin_cache
          .set_cache(plugin.name(), plugin_cache);
      }
    }

    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct PluginDriverTransformHookResult {
  pub content: String,
  pub source_map_chain: Vec<Arc<String>>,
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
    let plugin_driver = PluginDriver::new(
      vec![Arc::new(ResolvePlugin1 {}), Arc::new(ResolvePlugin2 {})],
      false,
    );

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

    let plugin_driver = PluginDriver::new(
      vec![Arc::new(ResolvePlugin3 {}), Arc::new(ResolvePlugin4 {})],
      false,
    );

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

    let plugin_driver = PluginDriver::new(vec![Arc::new(ResolvePlugin5 {})], false);

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
