use common::build_module;
use farmfe_core::module::ModuleSystem;
use farmfe_testing_helpers::fixture;

mod common;

#[test]
pub fn module_system() {
  fixture("tests/fixtures/module_system/**/*.js", |path, base| {
    let module = build_module(path.clone(), base);
    let (expected, contains_esm, contains_commonjs_exports) =
      match path.file_name().and_then(|name| name.to_str()) {
        Some("esm.js" | "hybrid.js" | "local-exports.js") => {
          // Require dependencies and locally bound `exports` do not change
          // the export system of a module with static ESM declarations.
          (ModuleSystem::EsModule, true, false)
        }
        Some("commonjs.js") => (ModuleSystem::CommonJs, false, false),
        Some("hybrid-exports.js") => (ModuleSystem::Hybrid, true, true),
        _ => unreachable!("Unexpected file: {}", path.display()),
      };
    let meta = module.meta.as_script();
    assert_eq!(meta.module_system, expected, "{}", path.display());
    assert_eq!(meta.contains_esm_decl, contains_esm, "{}", path.display());
    assert_eq!(
      meta.contains_module_exports,
      contains_commonjs_exports,
      "{}",
      path.display()
    );
  })
}

#[test]
pub fn module_system_with_ts() {
  fixture("tests/fixtures/module_system/hybrid*.ts", |path, base| {
    let module = build_module(path.clone(), base);
    let real_commonjs_exports = path.ends_with("hybrid-exports.ts");
    let meta = module.meta.as_script();
    assert_eq!(
      meta.module_system,
      if real_commonjs_exports {
        ModuleSystem::Hybrid
      } else {
        ModuleSystem::EsModule
      },
      "{}",
      path.display()
    );
    assert!(meta.contains_esm_decl);
    assert_eq!(meta.contains_module_exports, real_commonjs_exports);
  });
}
