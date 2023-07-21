use farmfe_testing_helpers::fixture;

use crate::common::build_module_deps;

mod common;

#[test]
pub fn import_equal() {
  fixture!(
    "tests/fixtures/analyze_deps/import_equal.ts",
    |path, base| {
      let (_, deps) = build_module_deps(path, base);

      assert_eq!(deps.len(), 1);
      assert_eq!(deps[0].source, "fs-extra".to_string());
    }
  );
}
