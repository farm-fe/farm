// use common::create_compiler;
// use farmfe_compiler::update::UpdateType;
use farmfe_testing_helpers::fixture;

mod common;

#[test]
fn update_without_dependencies_change() {
  fixture!("tests/fixtures/update/index.html", |file, crate_path| {
    // let cwd = file.parent().unwrap().to_path_buf();
    // let compiler = create_compiler(cwd.clone(), crate_path);

    // compiler.compile().unwrap();

    // let update_file = file
    //   .parent()
    //   .unwrap()
    //   .join("index.ts")
    //   .to_string_lossy()
    //   .to_string();
    // let result = compiler
    //   .update(vec![(update_file.clone(), UpdateType::Updated)])
    //   .unwrap();

    // assert_eq!(result.added_module_ids.len(), 0);
    // assert_eq!(result.updated_module_ids.len(), 1);
    // assert_eq!(result.removed_module_ids.len(), 0);

    // assert_eq!(
    //   result.updated_module_ids[0].resolved_path(&compiler.context().config.root),
    //   update_file
    // )
  });
}
