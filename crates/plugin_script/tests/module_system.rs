use common::build_module;
use farmfe_core::module::ModuleSystem;
use farmfe_toolkit::testing_helpers::fixture;

mod common;

#[test]
pub fn module_system() {
  fixture("tests/fixtures/module_system/**/*.js", |path, base| {
    let module = build_module(path.clone(), base);

    if path.ends_with("esm.js") {
      assert_eq!(
        module.meta.as_script().module_system,
        ModuleSystem::EsModule
      );
    } else if path.ends_with("commonjs.js") {
      assert_eq!(
        module.meta.as_script().module_system,
        ModuleSystem::CommonJs
      );
    } else if path.ends_with("hybrid.js") {
      assert_eq!(module.meta.as_script().module_system, ModuleSystem::Hybrid);
    } else {
      unreachable!("Unexpected file: {}", path.display());
    }
  })
}
