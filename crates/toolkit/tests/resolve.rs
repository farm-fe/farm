use farmfe_core::common::ParsedSideEffects;
use farmfe_testing_helpers::fixture;
use farmfe_toolkit::resolve::{self, PACKAGE_JSON_LOADER};

#[test]
fn load_package_json() {
  fixture("tests/fixtures/resolve/**/index.*", |file, _| {
    let dir = file.parent().unwrap();
    let main = dir.join("src").join("main.ts");

    let result = resolve::load_package_json(main.clone(), Default::default());
    assert!(result.is_ok());
    let result = result.unwrap();

    assert_eq!(result.name, Some("fixture-package-json".to_string()));
    assert_eq!(result.version, Some("1.0.0".to_string()));
    assert!(matches!(
      result.side_effects().unwrap(),
      ParsedSideEffects::Bool(false)
    ));

    let sub = dir.join("sub");

    let result = resolve::load_package_json(sub.clone(), Default::default());
    assert!(result.is_ok());
    let result = result.unwrap();

    assert_eq!(result.name, Some("sub-fixture".to_string()));
    assert_eq!(result.version, Some("1.0.0".to_string()));

    assert!(matches!(
      result.side_effects().unwrap(),
      ParsedSideEffects::Array(_)
    ));

    if let ParsedSideEffects::Array(arr) = result.side_effects().unwrap() {
      assert_eq!(
        *arr,
        vec![sub.join("main.css").to_string_lossy().to_string()]
      );
    }

    // make sure cache works
    let cache = PACKAGE_JSON_LOADER.cache();
    assert!(cache.contains_key(&main.to_string_lossy().to_string()));
    assert!(cache.contains_key(&dir.join("src").to_string_lossy().to_string()));
    assert!(cache.contains_key(&dir.to_string_lossy().to_string()));

    let cached_result = cache
      .get(&dir.join("src").to_string_lossy().to_string())
      .unwrap();
    assert_eq!(
      cached_result.value().name,
      Some("fixture-package-json".to_string()),
    );
  })
}
