use std::collections::HashMap;

use farmfe_core::{
  config::{external::ExternalConfig, Config, FARM_MODULE_SYSTEM},
  module::{ModuleId, ModuleType},
  plugin::{Plugin, PluginHookContext, PluginLoadHookResult, PluginResolveHookParam, ResolveKind},
};
use farmfe_toolkit::{html::get_farm_global_this, script::constant::RUNTIME_SUFFIX};
use farmfe_utils::{relative, stringify_query};

pub const DYNAMIC_VIRTUAL_SUFFIX: &str = ".farm_dynamic_import_virtual_module";
const ORIGINAL_RESOLVED_PATH: &str = "FARMFE_VIRTUAL_DYNAMIC_MODULE_ORIGINAL_RESOLVED_PATH";

pub struct FarmPluginLazyCompilation {}

impl FarmPluginLazyCompilation {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

const PLUGIN_NAME: &str = "FarmPluginLazyCompilation";

impl Plugin for FarmPluginLazyCompilation {
  fn name(&self) -> &str {
    PLUGIN_NAME
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
    if hook_context.contain_caller(PLUGIN_NAME)
    // All runtime files will be merged into one resourcePot, even files introduced through `import()`
    // Therefore, the asynchronous polyfill here is unnecessary
      || param.source.ends_with(RUNTIME_SUFFIX)
      || param
        .importer
        .as_ref()
        .is_some_and(|i| i.to_string().ends_with(RUNTIME_SUFFIX))
    {
      return Ok(None);
    }

    // If importer is a dynamic virtual module, we should resolve the dependency using the original importer
    if let Some(importer) = &param.importer {
      if importer.to_string().ends_with(DYNAMIC_VIRTUAL_SUFFIX) {
        let original_importer = importer.to_string().replace(DYNAMIC_VIRTUAL_SUFFIX, "");
        if let Some(res) = context.plugin_driver.resolve(
          &PluginResolveHookParam {
            importer: Some(original_importer.as_str().into()),
            ..param.clone()
          },
          context,
          &PluginHookContext {
            caller: hook_context.add_caller(PLUGIN_NAME),
            ..hook_context.clone()
          },
        )? {
          return Ok(Some(res));
        } else if original_importer == param.source {
          // if the original importer is the same as the source, it is a virtual module that farm generated, we should return the source as the resolved path
          return Ok(Some(farmfe_core::plugin::PluginResolveHookResult {
            resolved_path: param.source.clone(),
            external: false,
            side_effects: false,
            query: vec![],
            meta: HashMap::new(),
          }));
        }
      }
    }

    if param.source.ends_with(DYNAMIC_VIRTUAL_SUFFIX) {
      let original_path = param.source.replace(DYNAMIC_VIRTUAL_SUFFIX, "");
      let resolved_path =
        ModuleId::from(original_path.as_str()).resolved_path(&context.config.root);
      let resolve_result = context.plugin_driver.resolve(
        &PluginResolveHookParam {
          source: resolved_path.clone(),
          ..param.clone()
        },
        context,
        &PluginHookContext {
          caller: hook_context.add_caller(PLUGIN_NAME),
          ..hook_context.clone()
        },
      )?;

      if let Some(mut resolve_result) = resolve_result {
        resolve_result.meta.insert(
          ORIGINAL_RESOLVED_PATH.to_string(),
          resolve_result.resolved_path.clone(),
        );
        resolve_result.resolved_path =
          format!("{}{}", resolve_result.resolved_path, DYNAMIC_VIRTUAL_SUFFIX);

        return Ok(Some(resolve_result));
      } else {
        return Ok(Some(farmfe_core::plugin::PluginResolveHookResult {
          resolved_path: param.source.to_string(),
          external: false,
          side_effects: false,
          query: vec![],
          meta: HashMap::from([(ORIGINAL_RESOLVED_PATH.to_string(), resolved_path)]),
        }));
      }
    }

    let is_external = || {
      let external_config = ExternalConfig::from(&*context.config);

      external_config.is_external(&param.source)
    };

    // if the source is imported by dynamic import and it's not external source
    if matches!(param.kind, ResolveKind::DynamicImport) && !is_external() {
      let resolve_result = context.plugin_driver.resolve(
        param,
        context,
        &PluginHookContext {
          caller: hook_context.add_caller(PLUGIN_NAME),
          ..hook_context.clone()
        },
      )?;

      if let Some(resolve_result) = resolve_result {
        Ok(Some(farmfe_core::plugin::PluginResolveHookResult {
          resolved_path: format!("{}{}", resolve_result.resolved_path, DYNAMIC_VIRTUAL_SUFFIX),
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
    hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    if hook_context.contain_caller(PLUGIN_NAME) {
      return Ok(None);
    }

    if param.resolved_path.ends_with(DYNAMIC_VIRTUAL_SUFFIX) {
      if param.meta.get(ORIGINAL_RESOLVED_PATH).is_none() {
        let farm_global_this = get_farm_global_this(
          &context.config.runtime.namespace,
          &context.config.output.target_env,
        );
        let resolved_path = param.resolved_path;
        let dynamic_code = include_str!("dynamic_module.ts")
          .replace("MODULE_PATH", &resolved_path.replace('\\', r"\\"))
          .replace(
            "MODULE_ID",
            &ModuleId::new(
              resolved_path,
              &stringify_query(&param.query),
              &context.config.root,
            )
            .id(context.config.mode.clone())
            .replace('\\', r"\\"),
          )
          .replace(
            "'FARM_MODULE_SYSTEM'",
            &format!("{farm_global_this}.{FARM_MODULE_SYSTEM}"),
          );

        Ok(Some(farmfe_core::plugin::PluginLoadHookResult {
          content: dynamic_code,
          module_type: farmfe_core::module::ModuleType::Ts,
          source_map: None,
        }))
      } else {
        let resolved_path = param.meta.get(ORIGINAL_RESOLVED_PATH).unwrap();
        let dir = std::path::Path::new(&param.resolved_path)
          .parent()
          .unwrap()
          .to_string_lossy()
          .to_string();
        let relative_source = relative(&dir, &resolved_path);
        let content = format!(
          r#"
          import * as ns from "./{0}"
          module.exports = ns;
        "#,
          relative_source
        );
        Ok(Some(PluginLoadHookResult {
          content,
          module_type: ModuleType::Js,
          source_map: None,
        }))
      }
    } else {
      Ok(None)
    }
  }
}
