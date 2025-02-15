#![deny(clippy::all)]

use farmfe_core::{
  config::Config,
  plugin::{Plugin, PluginAnalyzeDepsHookResultEntry, ResolveKind},
  serde_json,
};

use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit_plugin_types::{
  libloading::Library,
  load_core_lib,
  swc_ast::create_swc_source_map,
  swc_transforms::{swc_transform_react, FarmSwcTransformReactOptions},
};

mod react_refresh;
use react_refresh::{inject_react_refresh, IS_REACT_REFRESH_BOUNDARY};

const GLOBAL_INJECT_MODULE_ID: &str = "farmfe_plugin_react_global_inject";

#[derive(Debug, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct SwcTransformReactOptions {
  pub refresh: Option<bool>,
  pub use_absolute_path: Option<bool>,
}

#[farm_plugin]
pub struct FarmPluginReact {
  core_lib: Library,
  options: String,
  enable_react_refresh: bool,
  use_absolute_path: bool,
}

impl FarmPluginReact {
  pub fn new(config: &Config, options: String) -> Self {
    let react_options = serde_json::from_str::<SwcTransformReactOptions>(&options).unwrap();
    let is_dev = matches!(config.mode, farmfe_core::config::Mode::Development);

    // remove useAbsolutePath in options
    let mut options_obj =
      serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&options).unwrap();
    options_obj.remove("useAbsolutePath");
    let options = serde_json::to_string(&options_obj).unwrap();

    Self {
      core_lib: load_core_lib(config.core_lib_path.as_ref().unwrap()),
      options,
      enable_react_refresh: is_dev && react_options.refresh.unwrap_or(true),
      use_absolute_path: react_options.use_absolute_path.unwrap_or(false),
    }
  }
}

impl Plugin for FarmPluginReact {
  fn name(&self) -> &str {
    "FarmPluginReact"
  }
  fn priority(&self) -> i32 {
    99
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
    } else if param.source == IS_REACT_REFRESH_BOUNDARY {
      return Ok(Some(farmfe_core::plugin::PluginResolveHookResult {
        resolved_path: IS_REACT_REFRESH_BOUNDARY.to_string(),
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
        source_map: None,
      }));
    } else if param.resolved_path == IS_REACT_REFRESH_BOUNDARY {
      let code = include_str!("is_react_refresh_boundary.ts");
      return Ok(Some(farmfe_core::plugin::PluginLoadHookResult {
        content: code.to_string(),
        module_type: farmfe_core::module::ModuleType::Ts,
        source_map: None,
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
      let top_level_mark = param.meta.as_script().top_level_mark;
      let unresolved_mark = param.meta.as_script().unresolved_mark;
      let ast = &mut param.meta.as_script_mut().ast;

      let file_name = if self.use_absolute_path {
        param.module_id.resolved_path(&context.config.root)
      } else {
        param.module_id.to_string()
      };

      let (cm, _) = create_swc_source_map(&self.core_lib, &file_name, param.content.clone())?;

      swc_transform_react(
        &self.core_lib,
        ast,
        FarmSwcTransformReactOptions {
          top_level_mark,
          unresolved_mark,
          inject_helpers: true,
          cm,
          globals: &context.meta.script.globals,
          mode: context.config.mode,
          options: self.options.clone(),
        },
      )?;

      if self.enable_react_refresh {
        inject_react_refresh(&self.core_lib, ast);
      }

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
    if self.enable_react_refresh
      && param.module.module_type == farmfe_core::module::ModuleType::Html
    {
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
