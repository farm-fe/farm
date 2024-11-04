use farmfe_core::{
  config::Mode,
  error::Result,
  serde_json,
  swc_common::{comments::NoopComments, Mark},
  swc_ecma_ast::{self, Program},
};
use farmfe_toolkit::{
  script::swc_try_with::try_with,
  swc_ecma_transforms::{
    helpers::inject_helpers,
    react::{react, Options, RefreshOptions, Runtime},
  },
};
use farmfe_toolkit_plugin_types::swc_transforms::FarmSwcTransformReactOptions;

#[no_mangle]
pub fn farm_swc_transform_react(
  ast: &mut swc_ecma_ast::Module,
  options: FarmSwcTransformReactOptions,
) -> Result<()> {
  let is_dev = matches!(options.mode, Mode::Development);
  let top_level_mark = Mark::from_u32(options.top_level_mark);
  let unresolved_mark = Mark::from_u32(options.unresolved_mark);
  let swc_transforms_react_options = serde_json::from_str::<Options>(&options.options).unwrap();
  let development = if is_dev {
    if let Some(development) = &swc_transforms_react_options.development {
      Some(*development)
    } else {
      Some(true)
    }
  } else {
    None
  };
  let react_options = Options {
    refresh: if matches!(development, Some(true)) {
      if let Some(refresh) = swc_transforms_react_options.refresh {
        Some(refresh)
      } else {
        Some(RefreshOptions::default())
      }
    } else {
      None
    },
    development,
    runtime: if let Some(runtime) = &swc_transforms_react_options.runtime {
      Some(*runtime)
    } else {
      Some(Runtime::Automatic)
    },
    ..swc_transforms_react_options
  };

  try_with(options.cm.clone(), options.globals, || {
    let take_ast = std::mem::take(ast);
    let mut program = Program::Module(take_ast);

    program.mutate(&mut react(
      options.cm,
      Some(NoopComments),
      react_options,
      top_level_mark,
      unresolved_mark,
    ));

    if options.inject_helpers {
      program.mutate(&mut inject_helpers(unresolved_mark));
    }

    *ast = program.expect_module();
  })
}
