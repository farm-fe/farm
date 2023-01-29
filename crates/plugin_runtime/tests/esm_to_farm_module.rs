use farmfe_testing_helpers::fixture;

#[test]
fn test_esm_to_farm_module() {
  fixture(
    "tests/fixtures/esm_to_farm_module/**/input.js",
    |input, base| {
      println!("input: {:?} {:?}", input, base);
    },
  );
}
