use std::{collections::HashMap, sync::Arc};

use farmfe_core::{
  module::ModuleType,
  plugin::Plugin,
  swc_common::{comments::NoopComments, Mark, GLOBALS},
};
use farmfe_testing_helpers::fixture;
use farmfe_toolkit::{
  script::swc_try_with::try_with,
  swc_ecma_transforms::{
    helpers::inject_helpers,
    react::{react, Options},
    typescript::strip_with_jsx,
  },
  swc_ecma_visit::VisitMutWith,
};

use crate::common::{assert_compiler_result, create_compiler, create_compiler_with_plugins};

mod common;

#[test]
fn tree_shake_test() {
  fixture!(
    "tests/fixtures/tree_shake/**/index.ts",
    |file, crate_path| {
      let cwd = file.parent().unwrap();
      println!("testing tree shake: {:?}", cwd);

      let compiler = create_compiler(
        HashMap::from([("index".to_string(), "./index.ts".to_string())]),
        cwd.to_path_buf(),
        crate_path,
      );
      compiler.compile().unwrap();

      assert_compiler_result(&compiler);
    }
  );
}

#[test]
fn tree_shake_html_entry() {
  fixture!(
    "tests/fixtures/tree_shake/html_entry/**/index.html",
    |file, crate_path| {
      let cwd = file.parent().unwrap();
      println!("testing tree shake: {:?}", cwd);

      let compiler = create_compiler(
        HashMap::from([("index".to_string(), "./index.html".to_string())]),
        cwd.to_path_buf(),
        crate_path,
      );
      compiler.compile().unwrap();

      assert_compiler_result(&compiler);
    }
  );
}

#[test]
fn tree_shake_changed_ast() {
  struct ProcessAstPlugin;

  impl Plugin for ProcessAstPlugin {
    fn name(&self) -> &str {
      "process-ast"
    }

    fn process_module(
      &self,
      param: &mut farmfe_core::plugin::PluginProcessModuleHookParam,
      context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    ) -> farmfe_core::error::Result<Option<()>> {
      if !matches!(param.module_type, ModuleType::Tsx | ModuleType::Jsx) {
        println!(
          "skip non-jsx module: {:?} for {:?}",
          param.module_type, param.module_id
        );
        return Ok(None);
      }

      try_with(
        context.meta.script.cm.clone(),
        &context.meta.script.globals,
        || {
          let top_level_mark = Mark::from_u32(param.meta.as_script_mut().top_level_mark);
          let unresolved_mark = Mark::from_u32(param.meta.as_script_mut().unresolved_mark);

          let ast = &mut param.meta.as_script_mut().ast;
          ast.visit_mut_with(&mut strip_with_jsx(
            context.meta.script.cm.clone(),
            Default::default(),
            None as Option<NoopComments>,
            top_level_mark,
          ));
          ast.visit_mut_with(&mut react::<NoopComments>(
            context.meta.script.cm.clone(),
            None,
            Options {
              refresh: Some(Default::default()),
              development: Some(true),
              ..Default::default()
            },
            top_level_mark,
          ));
          ast.visit_mut_with(&mut inject_helpers(unresolved_mark));
        },
      )
      .unwrap();

      Ok(Some(()))
    }
  }

  // test tree shake that the ast is changed after the mark is resolved.
  // if we don't get the latest mark, the result should be wrong.
  // else the result should be correct.
  fixture!(
    "tests/fixtures/tree_shake/changed_ast/entry.ts",
    |file, crate_path| {
      let cwd = file.parent().unwrap();
      println!("testing tree shake: {:?}", cwd);

      let compiler = create_compiler_with_plugins(
        HashMap::from([("index".to_string(), "./entry.ts".to_string())]),
        cwd.to_path_buf(),
        crate_path,
        vec![Arc::new(ProcessAstPlugin) as Arc<dyn Plugin>],
      );
      compiler.compile().unwrap();

      assert_compiler_result(&compiler);
    }
  );
}
