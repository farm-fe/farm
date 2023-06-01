use std::collections::HashMap;

use farmfe_core::{
  config::{Config, FARM_MODULE_SYSTEM},
  module::ModuleId,
  plugin::{Plugin, PluginHookContext, PluginResolveHookParam, ResolveKind},
};
use farmfe_utils::stringify_query;

const DYNAMIC_VIRTUAL_PREFIX: &str = "virtual:FARMFE_DYNAMIC_IMPORT:";

pub struct FarmPluginLazyCompilation {}

impl FarmPluginLazyCompilation {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginLazyCompilation {
  fn name(&self) -> &str {
    "FarmPluginLazyCompilation"
  }

  /// The lazy compilation plugin should take priority of all other plugins
  fn priority(&self) -> i32 {
    i32::MAX
  }

  fn resolve(
    &self,
    param: &farmfe_core::plugin::PluginResolveHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginResolveHookResult>> {
    if let Some(caller) = &hook_context.caller {
      if caller == "FarmPluginLazyCompilation" {
        return Ok(None);
      }
    }

    // If importer is dynamic virtual, we should resolve the dependency using the original importer
    if let Some(importer) = &param.importer {
      if importer.to_string().starts_with(DYNAMIC_VIRTUAL_PREFIX) {
        let original_importer = importer.to_string().replace(DYNAMIC_VIRTUAL_PREFIX, "");
        let resolve_result = context.plugin_driver.resolve(
          &PluginResolveHookParam {
            importer: Some(original_importer.into()),
            ..param.clone()
          },
          context,
          &PluginHookContext {
            caller: Some("FarmPluginLazyCompilation".to_string()),
            ..hook_context.clone()
          },
        )?;

        if let Some(resolve_result) = resolve_result {
          return Ok(Some(resolve_result));
        }
      }
    }

    if param.source.starts_with(DYNAMIC_VIRTUAL_PREFIX) {
      let original_path = param.source.replace(DYNAMIC_VIRTUAL_PREFIX, "");
      let resolve_result = context.plugin_driver.resolve(
        &PluginResolveHookParam {
          source: original_path.clone(),
          ..param.clone()
        },
        context,
        &PluginHookContext {
          caller: Some("FarmPluginLazyCompilation".to_string()),
          ..hook_context.clone()
        },
      )?;

      if let Some(mut resolve_result) = resolve_result {
        resolve_result.resolved_path =
          format!("{}{}", DYNAMIC_VIRTUAL_PREFIX, resolve_result.resolved_path);
        resolve_result
          .meta
          .insert("original".to_string(), original_path);

        return Ok(Some(resolve_result));
      } else {
        return Ok(Some(farmfe_core::plugin::PluginResolveHookResult {
          resolved_path: param.source.to_string(),
          external: false,
          side_effects: false,
          query: vec![],
          meta: HashMap::from([(
            "original".to_string(),
            param.source.replace(DYNAMIC_VIRTUAL_PREFIX, ""),
          )]),
        }));
      }
    }

    if matches!(param.kind, ResolveKind::DynamicImport) {
      let resolve_result = context.plugin_driver.resolve(
        param,
        context,
        &PluginHookContext {
          caller: Some("FarmPluginLazyCompilation".to_string()),
          ..hook_context.clone()
        },
      )?;

      if let Some(resolve_result) = resolve_result {
        Ok(Some(farmfe_core::plugin::PluginResolveHookResult {
          resolved_path: format!("{}{}", DYNAMIC_VIRTUAL_PREFIX, resolve_result.resolved_path),
          external: resolve_result.external,
          side_effects: resolve_result.side_effects,
          query: resolve_result.query,
          meta: resolve_result.meta,
        }))
      } else {
        Ok(None)
      }
    } else {
      Ok(None)
    }
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    if let Some(caller) = &_hook_context.caller {
      if caller == "FarmPluginLazyCompilation" {
        return Ok(None);
      }
    }

    if param.resolved_path.starts_with(DYNAMIC_VIRTUAL_PREFIX) {
      if param.meta.get("original").is_none() {
        let resolved_path = param.resolved_path;
        let dynamic_code = include_str!("dynamic_module.ts")
          .replace("MODULE_PATH", &resolved_path.replace(r"\", r"\\"))
          .replace(
            "MODULE_ID",
            &ModuleId::new(
              resolved_path,
              &stringify_query(&param.query),
              &context.config.root,
            )
            .id(context.config.mode.clone())
            .replace(r"\", r"\\"),
          )
          .replace(
            "'FARM_MODULE_SYSTEM'",
            &format!("window.{}", FARM_MODULE_SYSTEM),
          );

        Ok(Some(farmfe_core::plugin::PluginLoadHookResult {
          content: dynamic_code,
          module_type: farmfe_core::module::ModuleType::Ts,
        }))
      } else {
        let resolved_path = param.meta.get("original").unwrap();
        context.plugin_driver.load(
          &farmfe_core::plugin::PluginLoadHookParam {
            resolved_path,
            query: param.query.clone(),
            meta: param.meta.clone(),
          },
          context,
          &farmfe_core::plugin::PluginHookContext {
            caller: Some("FarmPluginLazyCompilation".to_string()),
            .._hook_context.clone()
          },
        )
      }
    } else {
      Ok(None)
    }
  }
}
