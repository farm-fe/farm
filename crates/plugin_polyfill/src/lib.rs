use farmfe_core::{
  config::{
    config_regex::ConfigRegex,
    preset_env::{PresetEnvConfig, PresetEnvConfigObj},
    AliasItem, Config, Mode as FarmMode, StringOrRegex,
  },
  plugin::Plugin,
  serde_json,
  swc_common::{comments::SingleThreadedComments, Mark},
  swc_ecma_ast::{EsVersion, Module, ModuleItem, Program, Script},
};
use farmfe_toolkit::{
  constant::RUNTIME_SUFFIX,
  script::swc_try_with::{resolve_module_mark, try_with},
  swc_ecma_preset_env::{self, transform_from_env, transform_from_es_version, EnvConfig, Mode},
  swc_ecma_transforms::{
    fixer,
    hygiene::{hygiene_with_config, Config as HygieneConfig},
    Assumptions,
  },
  swc_ecma_transforms_base::helpers::inject_helpers,
  swc_ecma_visit::VisitMutWith,
};

pub const SWC_HELPERS_PACKAGE: &str = "@swc/helpers";

pub struct FarmPluginPolyfill {
  config: swc_ecma_preset_env::Config,
  include: Vec<ConfigRegex>,
  exclude: Vec<ConfigRegex>,
  enforce_exclude: Vec<ConfigRegex>,
  assumptions: Assumptions,
  farm_runtime_regex: ConfigRegex,
}

impl FarmPluginPolyfill {
  pub fn new(config: &Config) -> Self {
    let (config, include, exclude, assumptions) = match &*config.preset_env {
      PresetEnvConfig::Bool(_) => {
        let PresetEnvConfigObj {
          include,
          exclude,
          options: _,
          assumptions: _,
        } = PresetEnvConfigObj::default();

        (
          swc_ecma_preset_env::Config {
            mode: Some(Mode::Usage),
            ..Default::default()
          },
          include,
          exclude,
          Default::default(),
        )
      }
      PresetEnvConfig::Obj(obj) => {
        let options = &obj.options;
        let mut user_config: swc_ecma_preset_env::Config =
          serde_json::from_value(*options.clone()).unwrap();
        user_config.mode = user_config.mode.or(Some(Mode::Usage));
        let user_assumption: Assumptions =
          serde_json::from_value(*obj.assumptions.clone()).unwrap();
        (
          user_config,
          obj.include.clone(),
          obj.exclude.clone(),
          user_assumption,
        )
      }
    };

    Self {
      config,
      include,
      exclude,
      assumptions,
      enforce_exclude: vec![ConfigRegex::new("node_modules/core-js")],
      farm_runtime_regex: ConfigRegex::new(format!("\\{RUNTIME_SUFFIX}$").as_str()),
    }
  }
}

impl Plugin for FarmPluginPolyfill {
  fn name(&self) -> &str {
    "FarmPluginPolyfill"
  }
  /// The polyfill plugin should run after all other plugins
  fn priority(&self) -> i32 {
    i32::MIN
  }

  /// Add alias for swc helpers
  fn config(&self, config: &mut Config) -> farmfe_core::error::Result<Option<()>> {
    let swc_helpers_find = StringOrRegex::String(SWC_HELPERS_PACKAGE.to_string());

    if !config.runtime.swc_helpers_path.is_empty()
      && !config.resolve.alias.iter().any(|a| {
        if let StringOrRegex::String(str) = &a.find {
          str == SWC_HELPERS_PACKAGE
        } else {
          false
        }
      })
    {
      config.resolve.alias.push(AliasItem {
        find: swc_helpers_find,
        replacement: config.runtime.swc_helpers_path.clone(),
      });
    }

    Ok(Some(()))
  }

  fn process_module(
    &self,
    param: &mut farmfe_core::plugin::PluginProcessModuleHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if !param.module_type.is_script() {
      return Ok(None);
    }

    // ignore node_modules by default
    let relative_path = param.module_id.relative_path();

    if !self.include.iter().any(|r| r.is_match(relative_path))
      && self.exclude.iter().any(|r| r.is_match(relative_path))
    {
      return Ok(None);
    }

    if self
      .enforce_exclude
      .iter()
      .any(|r| r.is_match(relative_path))
    {
      return Ok(None);
    }

    let cm = context.meta.get_module_source_map(param.module_id);
    let globals = context.meta.get_globals(&param.module_id);
    try_with(cm, globals.value(), || {
      let unresolved_mark = Mark::from_u32(param.meta.as_script().unresolved_mark);
      let top_level_mark = Mark::from_u32(param.meta.as_script().top_level_mark);

      let ast = param.meta.as_script_mut().take_ast();
      let is_runtime_module = self.farm_runtime_regex.is_match(relative_path);

      // fix #2103, transform the ast from Module to Script if the module does not have module declaration
      // to make swc polyfill prepend require('core-js/xxx') instead of import 'core-js/xxx'
      let mut final_ast = if ast
        .body
        .iter()
        .all(|item| !matches!(item, farmfe_core::swc_ecma_ast::ModuleItem::ModuleDecl(_)))
        && !is_runtime_module
      {
        Program::Script(Script {
          span: ast.span,
          body: ast
            .body
            .into_iter()
            .map(|item| item.expect_stmt())
            .collect(),
          shebang: ast.shebang,
        })
      } else {
        Program::Module(ast)
      };

      let comments: SingleThreadedComments = param.meta.as_script().comments.clone().into();

      if is_runtime_module && matches!(context.config.mode, FarmMode::Production) {
        // downgrade syntax for runtime module but do not inject polyfill
        final_ast.mutate(&mut transform_from_es_version(
          unresolved_mark,
          Some(&comments),
          EsVersion::Es5,
          self.assumptions,
          false,
        ));
      } else if !is_runtime_module {
        final_ast.mutate(&mut transform_from_env(
          unresolved_mark,
          Some(&comments),
          EnvConfig::from(self.config.clone()),
          self.assumptions,
        ));
      }

      final_ast.visit_mut_with(&mut inject_helpers(unresolved_mark));
      final_ast.visit_mut_with(&mut hygiene_with_config(HygieneConfig {
        top_level_mark,
        ..Default::default()
      }));
      final_ast.visit_mut_with(&mut fixer(Some(&comments)));

      let module_ast = match final_ast {
        Program::Script(script_ast) => Module {
          span: script_ast.span,
          body: script_ast
            .body
            .into_iter()
            .map(|item| ModuleItem::Stmt(item))
            .collect(),
          shebang: script_ast.shebang,
        },
        Program::Module(module_ast) => module_ast,
      };

      param.meta.as_script_mut().set_ast(module_ast);
    })?;

    // we have to update unresolved_mark and top_level_mark after hygiene, cause hygiene will change the mark of some nodes,
    // which may cause some unexpected error later handling global variables and top level variables
    let (unresolved_mark, top_level_mark) = resolve_module_mark(
      &mut param.meta.as_script_mut().ast,
      param.module_type.is_typescript(),
      globals.value(),
    );

    param.meta.as_script_mut().unresolved_mark = unresolved_mark.as_u32();
    param.meta.as_script_mut().top_level_mark = top_level_mark.as_u32();

    Ok(Some(()))
  }
}
