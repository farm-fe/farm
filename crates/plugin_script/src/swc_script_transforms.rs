use std::sync::Arc;

use farmfe_core::{
  config::script::{DecoratorVersion, ScriptConfig},
  context::CompilationContext,
  module::ModuleId,
  plugin::PluginProcessModuleHookParam,
  swc_common::{comments::SingleThreadedComments, Mark, SourceMap},
  swc_ecma_ast::Program,
};
use farmfe_toolkit::{
  script::swc_try_with::try_with,
  swc_ecma_transforms::{
    proposals::{decorator_2022_03::decorator_2022_03, decorators},
    typescript::{tsx, typescript, Config as TsConfig, ImportsNotUsedAsValues, TsxConfig},
  },
  swc_ecma_transforms_base::helpers::inject_helpers,
  swc_ecma_visit::{FoldWith, VisitMutWith},
};

fn default_config(script: &ScriptConfig, module_id: &ModuleId) -> TsConfig {
  let import_not_used_as_values = if script.import_not_used_as_values.is_preserved(module_id) {
    ImportsNotUsedAsValues::Preserve
  } else {
    ImportsNotUsedAsValues::Remove
  };
  TsConfig {
    // verbatim_module_syntax: script.verbatim_module_syntax,
    import_not_used_as_values,
    ..Default::default()
  }
}

pub fn strip_typescript(
  param: &mut PluginProcessModuleHookParam,
  cm: &Arc<SourceMap>,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  try_with(cm.clone(), &context.meta.script.globals, || {
    let top_level_mark = Mark::from_u32(param.meta.as_script().top_level_mark);
    let ast = param.meta.as_script_mut().take_ast();
    let mut program = Program::Module(ast);

    match param.module_type {
      farmfe_core::module::ModuleType::Js => {}
      farmfe_core::module::ModuleType::Jsx => {
        // Do nothing, jsx should be handled by other plugins
      }
      farmfe_core::module::ModuleType::Ts => {
        program.visit_mut_with(&mut typescript(
          default_config(&context.config.script, param.module_id),
          top_level_mark,
        ));
      }
      farmfe_core::module::ModuleType::Tsx => {
        let comments: SingleThreadedComments = param.meta.as_script().comments.clone().into();
        // TODO make it configurable
        program.visit_mut_with(&mut tsx(
          cm.clone(),
          default_config(&context.config.script, param.module_id),
          TsxConfig::default(),
          comments,
          top_level_mark,
        ));
        program.visit_mut_with(&mut typescript(
          default_config(&context.config.script, param.module_id),
          top_level_mark,
        ));
      }
      _ => {}
    }

    param.meta.as_script_mut().set_ast(program.expect_module());
  })
}

pub fn transform_decorators(
  param: &mut PluginProcessModuleHookParam,
  cm: &Arc<SourceMap>,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  let config = &context.config.script.decorators;
  let is_included = config
    .includes
    .iter()
    .any(|r| r.is_match(&param.module_id.to_string()));
  let is_excluded = config
    .excludes
    .iter()
    .any(|r| r.is_match(&param.module_id.to_string()));

  if is_included || !is_excluded {
    try_with(cm.clone(), &context.meta.script.globals, || {
      let mut ast = param.meta.as_script_mut().take_ast();

      match config.decorator_version.clone().unwrap_or_default() {
        DecoratorVersion::V202112 => {
          ast = ast.fold_with(&mut decorators(decorators::Config {
            legacy: config.legacy_decorator,
            emit_metadata: config.decorator_metadata,
            ..Default::default()
          }));
        }
        DecoratorVersion::V202203 => ast = ast.fold_with(&mut decorator_2022_03()),
      }

      let unresolved_mark = Mark::from_u32(param.meta.as_script().unresolved_mark);
      ast.visit_mut_with(&mut inject_helpers(unresolved_mark));

      param.meta.as_script_mut().set_ast(ast);
    })?;
  }

  Ok(())
}
