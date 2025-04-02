use farmfe_core::{
  config::{
    config_regex::ConfigRegex,
    preset_env::{PresetEnvConfig, PresetEnvConfigObj},
    Config,
  },
  plugin::Plugin,
  serde_json,
  swc_common::{comments::SingleThreadedComments, Mark},
  swc_ecma_ast::{Module, ModuleItem, Program, Script},
};
use farmfe_toolkit::{
  preset_env_base::query::Query,
  script::swc_try_with::try_with,
  sourcemap::create_swc_source_map,
  swc_ecma_preset_env::{self, preset_env, Mode, Targets},
  swc_ecma_transforms::Assumptions,
  swc_ecma_transforms_base::{feature::FeatureFlag, helpers::inject_helpers},
  swc_ecma_visit::VisitMutWith,
};

pub struct FarmPluginPolyfill {
  config: swc_ecma_preset_env::Config,
  include: Vec<ConfigRegex>,
  exclude: Vec<ConfigRegex>,
  enforce_exclude: Vec<ConfigRegex>,
  assumptions: Assumptions,
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
            targets: Some(Targets::Query(Query::Single("ie >= 9".to_string()))),
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
        user_config.targets = user_config
          .targets
          .or(Some(Targets::Query(Query::Single("ie >= 9".to_string()))));
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
      enforce_exclude: vec![
        ConfigRegex::new("node_modules/core-js"),
        ConfigRegex::new("@farmfe/runtime"),
      ],
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

    let (cm, _) = create_swc_source_map(param.module_id, param.content.clone());
    let globals = context.meta.get_globals(&param.module_id);
    try_with(cm, globals.value(), || {
      let unresolved_mark = Mark::from_u32(param.meta.as_script().unresolved_mark);
      let ast = param.meta.as_script_mut().take_ast();

      // fix #2103, transform the ast from Module to Script if the module does not have module declaration
      // to make swc polyfill prepend require('core-js/xxx') instead of import 'core-js/xxx'
      let mut final_ast = if ast
        .body
        .iter()
        .all(|item| !matches!(item, farmfe_core::swc_ecma_ast::ModuleItem::ModuleDecl(_)))
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

      let mut feature_flag = FeatureFlag::empty();
      let comments: SingleThreadedComments = param.meta.as_script().comments.clone().into();

      final_ast.mutate(&mut preset_env(
        unresolved_mark,
        Some(&comments),
        self.config.clone(),
        self.assumptions,
        &mut feature_flag,
      ));
      final_ast.visit_mut_with(&mut inject_helpers(unresolved_mark));

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

    Ok(Some(()))
  }
}
