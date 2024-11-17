use std::sync::Arc;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::{
  Plugin, PluginAnalyzeDepsHookParam, PluginDriverRenderResourcePotHookResult,
  PluginFinalizeModuleHookParam, PluginFinalizeResourcesHookParams,
  PluginGenerateResourcesHookResult, PluginHandleEntryResourceHookParams, PluginHookContext,
  PluginLoadHookParam, PluginLoadHookResult, PluginModuleGraphUpdatedHookParams,
  PluginParseHookParam, PluginProcessModuleHookParam, PluginRenderResourcePotHookParam,
  PluginRenderResourcePotHookResult, PluginResolveHookParam, PluginResolveHookResult,
  PluginTransformHookParam, PluginUpdateModulesHookParams,
};
use crate::{
  config::Config,
  context::CompilationContext,
  error::Result,
  module::{
    module_graph::ModuleGraph, module_group::ModuleGroupGraph, Module, ModuleId, ModuleMetaData,
    ModuleType,
  },
  resource::resource_pot::{ResourcePot, ResourcePotInfo, ResourcePotMetaData},
  stats::{CompilationModuleGraphStats, CompilationPluginHookStats, Stats},
};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub struct PluginDriver {
  pub plugins: Vec<Arc<dyn Plugin>>,
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
              .as_millis();
              let ret = plugin.$func_name($($arg),*)?;
              let end_time = SystemTime::now()
              .duration_since(UNIX_EPOCH)
              .expect("hook_first get end_time failed")
              .as_millis();
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

  ($func_name:ident, $param_ty:ty, $before_transformer:expr, $after_transformer:expr, $callback:expr) => {
    pub fn $func_name(&self, param: $param_ty, context: &Arc<CompilationContext>) -> Result<()> {
      for plugin in &self.plugins {
        if self.record {
          let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
          let ret = plugin.$func_name(param, context)?;
          let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("hook_first get end_time failed")
            .as_millis();
          if ret.is_some() {
            let plugin_name = plugin.name().to_string();
            let before = $before_transformer(param);
            let after = $after_transformer(param);
            $callback(
              plugin_name,
              start_time,
              end_time,
              before,
              after,
              param,
              context,
            );
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
    let start_time = if context.config.record {
      SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
    } else {
      0
    };

    for plugin in &self.plugins {
      if let Some(plugin_cache) = context.cache_manager.plugin_cache.read_cache(plugin.name()) {
        plugin.plugin_cache_loaded(plugin_cache.value(), context)?;

        if context.config.record {
          let end_time = if context.config.record {
            SystemTime::now()
              .duration_since(UNIX_EPOCH)
              .expect("hook_first get end_time failed")
              .as_millis()
          } else {
            0
          };
          context
            .record_manager
            .add_plugin_hook_stats(CompilationPluginHookStats {
              plugin_name: plugin.name().to_string(),
              hook_name: "plugin_cache_loaded".to_string(),
              hook_context: None,
              module_id: "root".into(),
              input: "".to_string(),
              output: "".to_string(),
              duration: end_time - start_time,
              start_time,
              end_time,
            });
        }
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
     start_time: u128,
     end_time: u128,
     param: &PluginResolveHookParam,
     context: &Arc<CompilationContext>,
     hook_context: &PluginHookContext| {
      if let Some(resolve_result) = result {
        context.record_manager.add_plugin_hook_stats(CompilationPluginHookStats {
          plugin_name: plugin_name.to_string(),
          hook_name: "resolve".to_string(),
          hook_context: Some(hook_context.clone()),
          module_id: param.importer.clone().unwrap_or("root".into()),
          input: serde_json::to_string(param).unwrap(),
          output: serde_json::to_string(resolve_result).unwrap(),
          duration: end_time - start_time,
          start_time,
          end_time
        });
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
     start_time: u128,
     end_time: u128,
     param: &PluginLoadHookParam,
     context: &Arc<CompilationContext>,
     hook_context: &PluginHookContext| {
      if let Some(load_result) = result {
        context.record_manager.add_plugin_hook_stats(CompilationPluginHookStats {
          plugin_name: plugin_name.to_string(),
          hook_name: "load".to_string(),
          hook_context: Some(hook_context.clone()),
          module_id: param.module_id.clone().into(),
          input: serde_json::to_string(param).unwrap(),
          output: serde_json::to_string(load_result).unwrap(),
          duration: end_time - start_time,
          start_time,
          end_time
        });
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

    fn transform_fn_with_hook() {}

    let transform_fn = transform_fn_with_hook;
    transform_fn();

    for plugin in &self.plugins {
      let start_time = if self.record {
        Some(
          SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis(),
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
              .as_millis(),
          )
        } else {
          None
        };

        if self.record {
          let plugin_name = plugin.name().to_string();
          let start_time = start_time.unwrap();
          let end_time = end_time.unwrap();
          context
            .record_manager
            .add_plugin_hook_stats(CompilationPluginHookStats {
              plugin_name: plugin_name.to_string(),
              hook_name: "transform".to_string(),
              hook_context: None,
              module_id: param.module_id.clone().into(),
              input: serde_json::to_string(&param).unwrap(),
              output: serde_json::to_string(&plugin_result).unwrap(),
              duration: end_time - start_time,
              start_time,
              end_time,
            });
        }

        param.content = plugin_result.content;
        param.module_type = plugin_result.module_type.unwrap_or(param.module_type);

        if plugin_result.ignore_previous_source_map {
          result.source_map_chain.clear();
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
     start_time: u128,
     end_time: u128,
     param: &PluginParseHookParam,
     context: &Arc<CompilationContext>,
     hook_context: &PluginHookContext| {
      context.record_manager.add_plugin_hook_stats(
        CompilationPluginHookStats {
          plugin_name: plugin_name.to_string(),
          hook_name: "parse".to_string(),
          hook_context: Some(hook_context.clone()),
          module_id: param.module_id.clone(),
          input: serde_json::to_string(param).unwrap(),
          output: "".to_string(),
          duration: end_time - start_time,
          start_time,
          end_time,
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
    |_: &mut PluginProcessModuleHookParam| { "".to_string() },
    |_: &mut PluginProcessModuleHookParam| { "".to_string() },
    |plugin_name: String,
     start_time: u128,
     end_time: u128,
     input: String,
     output: String,
     param: &PluginProcessModuleHookParam,
     context: &Arc<CompilationContext>| {
      context
        .record_manager
        .add_plugin_hook_stats(CompilationPluginHookStats {
          plugin_name: plugin_name.to_string(),
          hook_name: "process_module".to_string(),
          hook_context: None,
          module_id: param.module_id.clone(),
          input,
          output,
          duration: end_time - start_time,
          start_time,
          end_time,
        })
    }
  );

  hook_serial!(
    analyze_deps,
    &mut PluginAnalyzeDepsHookParam,
    |before_param: &mut PluginAnalyzeDepsHookParam| {
      serde_json::to_string(&before_param.deps).unwrap()
    },
    |after_param: &mut PluginAnalyzeDepsHookParam| {
      serde_json::to_string(&after_param.deps).unwrap()
    },
    |plugin_name: String,
     start_time: u128,
     end_time: u128,
     input: String,
     output: String,
     param: &PluginAnalyzeDepsHookParam,
     context: &Arc<CompilationContext>| {
      context
        .record_manager
        .add_plugin_hook_stats(CompilationPluginHookStats {
          plugin_name: plugin_name.to_string(),
          hook_name: "analyze_deps".to_string(),
          hook_context: None,
          module_id: param.module.id.clone(),
          input,
          output,
          duration: end_time - start_time,
          start_time,
          end_time,
        })
    }
  );

  hook_serial!(finalize_module, &mut PluginFinalizeModuleHookParam);

  hook_parallel!(build_end);

  hook_parallel!(generate_start);

  hook_serial!(
    optimize_module_graph,
    &mut ModuleGraph,
    |before_param: &mut ModuleGraph| {
      serde_json::to_string(&CompilationModuleGraphStats::from(&*before_param)).unwrap()
    },
    |after_param: &mut ModuleGraph| {
      serde_json::to_string(&CompilationModuleGraphStats::from(&*after_param)).unwrap()
    },
    |plugin_name: String,
     start_time: u128,
     end_time: u128,
     input: String,
     output: String,
     _: &mut ModuleGraph,
     context: &Arc<CompilationContext>| {
      context
        .record_manager
        .add_plugin_hook_stats(CompilationPluginHookStats {
          plugin_name: plugin_name.to_string(),
          hook_name: "optimize_module_graph".to_string(),
          hook_context: None,
          module_id: "".into(),
          input,
          output,
          duration: end_time - start_time,
          start_time,
          end_time,
        })
    }
  );

  hook_first!(
    analyze_module_graph,
    Result<Option<ModuleGroupGraph>>,
    |result: & Option<ModuleGroupGraph>,
     plugin_name: String,
     start_time: u128,
     end_time: u128,
     _param: &mut ModuleGraph,
     context: &Arc<CompilationContext>,
     hook_context: &PluginHookContext| {
      if result.is_none() {
        return;
      }
      context.record_manager.add_plugin_hook_stats(
        CompilationPluginHookStats {
          plugin_name: plugin_name.to_string(),
          hook_name: "analyze_module_graph".to_string(),
          hook_context: Some(hook_context.clone()),
          module_id: "".into(),
          input: "".to_string(),
          output: serde_json::to_string(&result.as_ref().unwrap().print_graph()).unwrap(),
          duration: end_time - start_time,
          start_time,
          end_time,
        },
      )
    },
    param: &mut ModuleGraph,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  hook_first!(
    partial_bundling,
    Result<Option<Vec<ResourcePot>>>,
    |result: &Option<Vec<ResourcePot>>,
     plugin_name: String,
     start_time: u128,
     end_time: u128,
     modules: &Vec<ModuleId>,
     context: &Arc<CompilationContext>,
     hook_context: &PluginHookContext| {
      if result.is_none() {
        return;
      }
      context.record_manager.add_plugin_hook_stats(
        CompilationPluginHookStats {
          plugin_name: plugin_name.to_string(),
          hook_name: "partial_bundling".to_string(),
          hook_context: Some(hook_context.clone()),
          module_id: "".into(),
          input: serde_json::to_string(&modules).unwrap(),
          output: serde_json::to_string(&result).unwrap(),
          duration: end_time - start_time,
          start_time,
          end_time,
        },
      )
    },
    modules: &Vec<ModuleId>,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext
  );

  hook_serial!(
    process_resource_pots,
    &mut Vec<&mut ResourcePot>,
    |before_resource_pots: &mut Vec<&mut ResourcePot>| {
      serde_json::to_string(&before_resource_pots).unwrap()
    },
    |after_resource_pots: &mut Vec<&mut ResourcePot>| {
      serde_json::to_string(&after_resource_pots).unwrap()
    },
    |plugin_name: String,
     start_time: u128,
     end_time: u128,
     input: String,
     output: String,
     _resource_pots: &mut Vec<&mut ResourcePot>,
     context: &Arc<CompilationContext>| {
      context
        .record_manager
        .add_plugin_hook_stats(CompilationPluginHookStats {
          plugin_name: plugin_name.to_string(),
          hook_name: "process_resource_pots".to_string(),
          hook_context: None,
          module_id: "".into(),
          input,
          output,
          duration: end_time - start_time,
          start_time,
          end_time,
        })
    }
  );

  hook_serial!(render_start, &Config);

  hook_first!(
    render_resource_pot_modules,
    Result<Option<ResourcePotMetaData>>,
    |result: &Option<ResourcePotMetaData>,
     plugin_name: String,
     start_time: u128,
     end_time: u128,
     resource_pot: &ResourcePot,
     context: &Arc<CompilationContext>,
     hook_context: &PluginHookContext| {
      if result.is_none() {
        return;
      }
      context.record_manager.add_plugin_hook_stats(
        CompilationPluginHookStats {
          plugin_name: plugin_name.to_string(),
          hook_name: "render_resource_pot_modules".to_string(),
          hook_context: Some(hook_context.clone()),
          module_id: "".into(),
          input: serde_json::to_string(resource_pot).unwrap(),
          output: serde_json::to_string(&result).unwrap(),
          duration: end_time - start_time,
          start_time,
          end_time,
        },
      )
    },
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
      let start_time = if context.config.record {
        std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .unwrap()
          .as_millis()
      } else {
        0
      };
      // if the transform hook returns None, treat it as empty hook and ignore it
      if let Some(plugin_result) = plugin.render_resource_pot(param, context)? {
        if context.config.record {
          let end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
          context
            .record_manager
            .add_plugin_hook_stats(CompilationPluginHookStats {
              plugin_name: plugin.name().to_string(),
              hook_name: "render_resource_pot".to_string(),
              hook_context: None,
              module_id: "".into(),
              input: serde_json::to_string(param).unwrap(),
              output: serde_json::to_string(&plugin_result).unwrap(),
              duration: end_time - start_time,
              start_time,
              end_time,
            })
        }

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
    |before_resource_pot: &mut ResourcePot| {
      serde_json::to_string(&before_resource_pot).unwrap()
    },
    |after_resource_pot: &mut ResourcePot| { serde_json::to_string(&after_resource_pot).unwrap() },
    |plugin_name: String,
     start_time: u128,
     end_time: u128,
     input: String,
     output: String,
     _resource_pot: &mut ResourcePot,
     context: &Arc<CompilationContext>| {
      context
        .record_manager
        .add_plugin_hook_stats(CompilationPluginHookStats {
          plugin_name: plugin_name.to_string(),
          hook_name: "optimize_resource_pot".to_string(),
          hook_context: None,
          module_id: "".into(),
          input,
          output,
          duration: end_time - start_time,
          start_time,
          end_time,
        })
    }
  );

  hook_first!(
    generate_resources,
    Result<Option<PluginGenerateResourcesHookResult>>,
    |result: &Option<PluginGenerateResourcesHookResult>,
     plugin_name: String,
     start_time: u128,
     end_time: u128,
     resource_pot: &mut ResourcePot,
     context: &Arc<CompilationContext>,
     hook_context: &PluginHookContext| {
      if let Some(resources) = result {
        context.record_manager.add_plugin_hook_stats(
          CompilationPluginHookStats {
            plugin_name: plugin_name.to_string(),
            hook_name: "generate_resources".to_string(),
            hook_context: Some(hook_context.clone()),
            module_id: "".into(),
            input: serde_json::to_string(resource_pot).unwrap(),
            output: serde_json::to_string(resources).unwrap(),
            duration: end_time - start_time,
            start_time,
            end_time,
          },
        );
      }
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
    |_plugin_name: String, _context: &Arc<CompilationContext>| {}
  );

  hook_serial!(
    update_modules,
    &mut PluginUpdateModulesHookParams,
    |before_params: &mut PluginUpdateModulesHookParams| {
      serde_json::to_string(&before_params).unwrap()
    },
    |after_params: &mut PluginUpdateModulesHookParams| {
      serde_json::to_string(&after_params).unwrap()
    },
    |plugin_name: String,
     start_time: u128,
     end_time: u128,
     input: String,
     output: String,
     _: &mut PluginUpdateModulesHookParams,
     context: &Arc<CompilationContext>| {
      context
        .record_manager
        .add_plugin_hook_stats(CompilationPluginHookStats {
          plugin_name,
          hook_name: "update_modules".to_string(),
          hook_context: None,
          module_id: "".into(),
          input,
          output,
          duration: end_time - start_time,
          start_time,
          end_time,
        })
    }
  );

  hook_parallel!(
    module_graph_updated,
    param: &PluginModuleGraphUpdatedHookParams
  );

  hook_first!(
    render_update_resource_pot,
    Result<Option<PluginRenderResourcePotHookResult>>,
    resource_pot: &ResourcePot,
    context: &Arc<CompilationContext>
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
      let start_time = if context.config.record {
        SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .expect("Time went backwards")
          .as_millis()
      } else {
        0
      };

      let plugin_cache = plugin.write_plugin_cache(context)?;

      if let Some(plugin_cache) = plugin_cache {
        context
          .cache_manager
          .plugin_cache
          .set_cache(plugin.name(), plugin_cache);

        if context.config.record {
          let end_time = if context.config.record {
            SystemTime::now()
              .duration_since(UNIX_EPOCH)
              .expect("Time went backwards")
              .as_millis()
          } else {
            0
          };

          context
            .record_manager
            .add_plugin_hook_stats(CompilationPluginHookStats {
              plugin_name: plugin.name().to_string(),
              hook_name: "write_plugin_cache".to_string(),
              hook_context: None,
              module_id: "".into(),
              input: "".to_string(),
              output: "".to_string(),
              duration: end_time - start_time,
              start_time,
              end_time,
            });
        }
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
