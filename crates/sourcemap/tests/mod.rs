use farmfe_testing_helpers::fixture;

#[test]
fn test_combine_string() {
  fixture!(
    "tests/fixtures/combine-string/**/input.ts",
    |file, crate_path| {}
  );
}
