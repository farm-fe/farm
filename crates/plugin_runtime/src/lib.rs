use std::{collections::HashMap, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  error::CompilationError,
  parking_lot::RwLock,
  plugin::{
    Plugin, PluginHookContext, PluginLoadHookParam, PluginLoadHookResult, PluginResolveHookParam,
    PluginResolveHookResult,
  },
  resource::resource_pot_graph::ResourcePotGraph,
};
use farmfe_toolkit::{fs::read_file_utf8, script::module_type_from_id};

const RUNTIME_PREFIX: &str = "runtime:";

/// FarmPluginRuntime is charge of:
/// * resolving, parsing and generating a executable runtime code and inject the code into the entries.
/// * merge module's ast and render the script module using farm runtime's specification, for example, wrap the module to something like `function(module, exports, require) { xxx }`, see [Farm Runtime RFC](https://github.com/farm-fe/rfcs/pull/1)
///
/// The runtime supports html entry and script(js/jsx/ts/tsx) entry, when entry is html, the runtime will be injected as a inline <script /> tag in the <head /> tag;
/// when entry is script, the runtime will be injected into the entry module's head, makes sure the runtime execute before all other code.
///
/// All runtime module (including the runtime core and its plugins) will be prefixed as `runtime:` to distinguish with normal script modules.
/// ```
pub struct FarmPluginRuntime {}

impl Plugin for FarmPluginRuntime {
  fn name(&self) -> &str {
    "FarmPluginRuntime"
  }

  fn resolve(
    &self,
    param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginResolveHookResult>> {
    // avoid cyclic resolve
    if matches!(&hook_context.caller, Some(c) if c == "FarmPluginRuntime") {
      Ok(None)
    } else if param.source.starts_with(RUNTIME_PREFIX) {
      let ori_source = param.source.replace(RUNTIME_PREFIX, "");
      let resolve_result = context.plugin_driver.resolve(
        &PluginResolveHookParam {
          source: ori_source,
          ..param.clone()
        },
        &context,
        &PluginHookContext {
          caller: Some(String::from("FarmPluginRuntime")),
          meta: HashMap::new(),
        },
      )?;

      if let Some(mut res) = resolve_result {
        res.id = format!("{}{}", RUNTIME_PREFIX, res.id);
        Ok(Some(res))
      } else {
        Ok(None)
      }
    } else {
      Ok(None)
    }
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    if param.id.starts_with(RUNTIME_PREFIX) {
      let real_file_path = param.id.replace(RUNTIME_PREFIX, "");
      let content = read_file_utf8(&real_file_path)?;

      Ok(Some(PluginLoadHookResult {
        content,
        module_type: module_type_from_id(param.id).ok_or_else(|| {
          CompilationError::GenericError(
            "Unsupported file type of runtime, only support `js/jsx/ts/tsx/mjs/cjs`".to_string(),
          )
        })?,
      }))
    } else {
      Ok(None)
    }
  }
  // fn process_resource_pot_graph(
  //   &self,
  //   resource_pot_graph: &RwLock<ResourcePotGraph>,
  //   context: &Arc<CompilationContext>,
  // ) -> farmfe_core::error::Result<Option<()>> {
  //   if matches!(context.config.runtime.path, None) {
  //     let rpg = resource_pot_graph.read();

  //     if rpg.resources().len() != 1 {
  //       panic!("default runtime only works with single resource");
  //     }

  //     Ok(Some(()))
  //   } else {
  //     Ok(None)
  //   }
  // }

  // fn render_resource_pot(
  //   &self,
  //   _resource_pot: &mut farmfe_core::resource::resource_pot::ResourcePot,
  //   context: &Arc<CompilationContext>,
  // ) -> farmfe_core::error::Result<Option<()>> {
  //   if matches!(context.config.runtime.path, None) {
  //     // Strip out the rendered module and wrap it with the minimal runtime
  //     Ok(Some(()))
  //   } else {
  //     Ok(None)
  //   }
  // }
}
