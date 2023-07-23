use farmfe_core::{
  config::{
    config_regex::ConfigRegex,
    preset_env::{PresetEnvConfig, PresetEnvConfigObj},
    Config,
  },
  plugin::Plugin,
  serde_json,
  swc_common::{comments::NoopComments, Mark},
};
use farmfe_toolkit::{
  preset_env_base::query::Query,
  script::swc_try_with::try_with,
  swc_ecma_preset_env::{self, preset_env, Mode, Targets, Version},
  swc_ecma_transforms::Assumptions,
  swc_ecma_transforms_base::{feature::FeatureFlag, helpers::inject_helpers},
  swc_ecma_visit::{FoldWith, VisitMutWith},
};

pub struct FarmPluginPolyfill {
  config: swc_ecma_preset_env::Config,
  include: Vec<ConfigRegex>,
  exclude: Vec<ConfigRegex>,
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
            core_js: Some(Version {
              major: 3,
              minor: 30,
              patch: 1,
            }),
            targets: Some(Targets::Query(Query::Single(
              "> 0.25%, not dead".to_string(),
            ))),
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
        user_config.core_js = user_config.core_js.or(Some(Version {
          major: 3,
          minor: 30,
          patch: 1,
        }));
        user_config.targets = user_config.targets.or(Some(Targets::Query(Query::Single(
          "> 0.25%, not dead".to_string(),
        ))));
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
    }
  }
}

impl Plugin for FarmPluginPolyfill {
  fn name(&self) -> &str {
    "FarmPluginPolyfill"
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

    try_with(
      context.meta.script.cm.clone(),
      &context.meta.script.globals,
      || {
        let unresolved_mark = Mark::from_u32(param.meta.as_script().unresolved_mark);
        let mut ast = param.meta.as_script_mut().take_ast();
        // TODO: store feature flags in module meta and use them when transform the module system
        let mut feature_flag = FeatureFlag::empty();

        ast = ast.fold_with(&mut preset_env(
          unresolved_mark,
          // TODO: support comments
          None as Option<NoopComments>,
          self.config.clone(),
          self.assumptions,
          &mut feature_flag,
        ));
        ast.visit_mut_with(&mut inject_helpers(unresolved_mark));

        param.meta.as_script_mut().set_ast(ast);
      },
    )?;

    Ok(Some(()))
  }
}
