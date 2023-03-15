#![deny(clippy::all)]

use farmfe_core::{
  config::Config,
  plugin::{Plugin, PluginAnalyzeDepsHookResultEntry, ResolveKind},
};

use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit_plugin_types::{
  libloading::Library,
  load_core_lib,
  swc_transforms::{swc_transform_react, FarmSwcTransformReactOptions},
};

mod react_refresh;
use react_refresh::inject_react_refresh;

const GLOBAL_INJECT_MODULE_ID: &str = "farmfe_plugin_react_global_inject";

#[farm_plugin]
pub struct FarmPluginReact {
  core_lib: Library,
}

impl FarmPluginReact {
  fn new(config: &Config, _options: String) -> Self {
    Self {
      core_lib: load_core_lib(config.core_lib_path.as_ref().unwrap()),
    }
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

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    let react_relative_paths = vec![
      "node_modules/react/index.js",
      "node_modules/react-dom/index.js",
      "node_modules/react-refresh/runtime.js",
    ];

    if react_relative_paths
      .into_iter()
      .any(|p| param.resolved_path.ends_with(p))
    {
      let if_str = "if (process.env.NODE_ENV === 'production') {";

      if param.content.contains(if_str) {
        let index = param.content.find(if_str).unwrap();
        let rest = param.content[index..].to_string();

        if matches!(context.config.mode, farmfe_core::config::Mode::Development) {
          let else_str = "} else {";
          let else_index = rest.find(else_str).unwrap();
          let else_rest = rest[else_index + else_str.len()..].to_string();
          let end_index = else_rest.find('}').unwrap();
          let dev_content = else_rest[..end_index].to_string();

          return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
            content: param.content.replace(rest.as_str(), dev_content.as_str()),
            module_type: Some(farmfe_core::module::ModuleType::Js),
            source_map: None,
          }));
        } else {
          let end_index = rest.find('}').unwrap();
          let prod_content = rest[if_str.len()..end_index].to_string();

          return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
            content: param.content.replace(rest.as_str(), prod_content.as_str()),
            module_type: Some(farmfe_core::module::ModuleType::Js),
            source_map: None,
          }));
        };
      }
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
      let is_dev = matches!(context.config.mode, farmfe_core::config::Mode::Development);
      swc_transform_react(
        &self.core_lib,
        context,
        ast,
        FarmSwcTransformReactOptions {
          top_level_mark,
          unresolved_mark,
          inject_helpers: true,
        },
      )?;

      if is_dev {
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
