use farmfe_core::{
  config::Config,
  plugin::Plugin,
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

pub struct FarmPluginPolyfill {}

impl FarmPluginPolyfill {
  pub fn new(_config: &Config) -> Self {
    Self {}
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
    // ignore node_modules by default
    // TODO: make this configurable
    if param.module_id.relative_path().contains("node_modules/") {
      return Ok(None);
    }

    if !param.module_type.is_script() {
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
          // TODO: make this configurable
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
          // TODO: make this configurable
          Assumptions::default(),
          &mut feature_flag,
        ));
        ast.visit_mut_with(&mut inject_helpers(unresolved_mark));

        param.meta.as_script_mut().set_ast(ast);
      },
    )?;

    Ok(Some(()))
  }
}
