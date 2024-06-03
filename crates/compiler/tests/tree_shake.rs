use std::{collections::HashMap, path::PathBuf, sync::Arc};

use farmfe_core::{
  module::ModuleType,
  plugin::Plugin,
  swc_common::{comments::NoopComments, Mark},
};
use farmfe_testing_helpers::fixture;
use farmfe_toolkit::{
  common::{create_swc_source_map, Source},
  script::swc_try_with::try_with,
  swc_ecma_transforms::{
    helpers::inject_helpers,
    react::{react, Options},
    typescript::tsx,
  },
  swc_ecma_visit::VisitMutWith,
};

use crate::common::{
  assert_compiler_result, create_compiler, create_compiler_with_plugins, create_config,
  create_with_compiler,
};

mod common;

#[test]
fn tree_shake_test() {
  fixture!(
    "tests/fixtures/tree_shake/self-executed/new-call/**/index.ts",
    |file, crate_path| {
      let cwd = file.parent().unwrap();
      println!("testing tree shake: {:?}", cwd);

      let entry_name = "index".to_string();
      let compiler = create_compiler(
        HashMap::from([(entry_name.clone(), "./index.ts".to_string())]),
        cwd.to_path_buf(),
        crate_path,
        false,
      );
      compiler.compile().unwrap();

      assert_compiler_result(&compiler, Some(&entry_name));
    }
  );
}

#[test]
fn tree_shake_development() {
  fixture!(
    "tests/fixtures/tree_shake_development/**/index.ts",
    |file, crate_path| {
      let cwd = file.parent().unwrap();
      let entry_name = "index".to_string();
      println!("testing tree shake: {:?}", cwd);

      let mut config = create_config(cwd.into(), crate_path);

      config.input = HashMap::from([(entry_name.clone(), "./index.ts".to_string())]);
      config.mode = farmfe_core::config::Mode::Development;

      let compiler = create_with_compiler(config, vec![]);

      compiler.compile().expect("failed compile");

      assert_compiler_result(&compiler, Some(&entry_name))
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

      let entry_name = "index".to_string();
      let compiler = create_compiler(
        HashMap::from([(entry_name, "./index.html".to_string())]),
        cwd.to_path_buf(),
        crate_path,
        false,
      );
      compiler.compile().unwrap();

      assert_compiler_result(&compiler, None);
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

      let (cm, _) = create_swc_source_map(Source {
        path: PathBuf::from(&param.module_id.to_string()),
        content: param.content.clone(),
      });
      try_with(cm.clone(), &context.meta.script.globals, || {
        let top_level_mark = Mark::from_u32(param.meta.as_script_mut().top_level_mark);
        let unresolved_mark = Mark::from_u32(param.meta.as_script_mut().unresolved_mark);

        let ast = &mut param.meta.as_script_mut().ast;
        ast.visit_mut_with(&mut tsx(
          cm.clone(),
          Default::default(),
          Default::default(),
          None as Option<NoopComments>,
          top_level_mark,
        ));
        ast.visit_mut_with(&mut react::<NoopComments>(
          cm.clone(),
          None,
          Options {
            refresh: Some(Default::default()),
            development: Some(true),
            ..Default::default()
          },
          top_level_mark,
          unresolved_mark,
        ));
        ast.visit_mut_with(&mut inject_helpers(unresolved_mark));
      })
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

      let entry_name = "index".to_string();
      let compiler = create_compiler_with_plugins(
        HashMap::from([(entry_name.clone(), "./entry.ts".to_string())]),
        cwd.to_path_buf(),
        crate_path,
        false,
        vec![Arc::new(ProcessAstPlugin) as Arc<dyn Plugin>],
      );
      compiler.compile().unwrap();

      assert_compiler_result(&compiler, Some(&entry_name));
    }
  );
}
