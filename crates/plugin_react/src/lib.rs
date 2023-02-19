#![deny(clippy::all)]

use farmfe_core::{
  config::Config,
  plugin::{Plugin, PluginAnalyzeDepsHookResultEntry, ResolveKind},
  swc_common::{comments::NoopComments, Mark, GLOBALS},
};

use farmfe_toolkit::{
  swc_ecma_transforms::react::{react, Options, RefreshOptions},
  swc_ecma_visit::VisitMutWith,
};
use react_refresh::inject_react_refresh;

mod react_refresh;

const GLOBAL_INJECT_MODULE_ID: &str = "farmfe_plugin_react_global_inject";

pub struct FarmPluginReact {}

impl FarmPluginReact {
  pub fn new(_config: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginReact {
  fn name(&self) -> &str {
    "FarmPluginReact"
  }

  fn resolve(
    &self,
    param: &farmfe_core::plugin::PluginResolveHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginResolveHookResult>> {
    if param.source == GLOBAL_INJECT_MODULE_ID {
      return Ok(Some(farmfe_core::plugin::PluginResolveHookResult {
        resolved_path: GLOBAL_INJECT_MODULE_ID.to_string(),
        ..Default::default()
      }));
    }

    Ok(None)
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    if param.resolved_path == GLOBAL_INJECT_MODULE_ID {
      return Ok(Some(farmfe_core::plugin::PluginLoadHookResult {
        content: r#"
            import RefreshRuntime from 'react-refresh';

            if (!window.__farm_refresh_runtime_injected_into_global_hook__) {
              RefreshRuntime.injectIntoGlobalHook(window)
              window.$RefreshReg$ = () => {}
              window.$RefreshSig$ = () => (type) => type
              window.__farm_refresh_runtime_injected_into_global_hook__ = true
            }"#
          .to_string(),
        module_type: farmfe_core::module::ModuleType::Js,
      }));
    }

    Ok(None)
  }

  fn process_module(
    &self,
    param: &mut farmfe_core::plugin::PluginProcessModuleHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if matches!(
      param.module_type,
      farmfe_core::module::ModuleType::Jsx | farmfe_core::module::ModuleType::Tsx
    ) {
      GLOBALS.set(&context.meta.script.globals, || {
        let top_level_mark = Mark::from_u32(param.meta.as_script().top_level_mark);
        let ast = &mut param.meta.as_script_mut().ast;
        let is_dev = matches!(context.config.mode, farmfe_core::config::Mode::Development);

        ast.visit_mut_with(&mut react(
          context.meta.script.cm.clone(),
          Some(NoopComments), // TODO parse comments
          Options {
            refresh: if is_dev {
              Some(RefreshOptions::default())
            } else {
              None
            },
            development: Some(is_dev),
            // runtime: Some(Runtime::Automatic),
            ..Default::default()
          },
          top_level_mark,
        ));

        if is_dev {
          inject_react_refresh(ast);
        }
      });

      return Ok(Some(()));
    }

    Ok(None)
  }

  fn analyze_deps(
    &self,
    param: &mut farmfe_core::plugin::PluginAnalyzeDepsHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    // insert a global entry into the html module and make sure the inserted module executes first
    if param.module.module_type == farmfe_core::module::ModuleType::Html {
      param.deps.insert(
        0,
        PluginAnalyzeDepsHookResultEntry {
          source: GLOBAL_INJECT_MODULE_ID.to_string(),
          kind: ResolveKind::ScriptSrc,
        },
      );
    }
    Ok(Some(()))
  }
}
