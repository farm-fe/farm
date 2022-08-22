use farmfe_toolkit::{
  resolve::{self, PACKAGE_JSON_LOADER},
  testing_helpers::fixture,
};

#[test]
fn load_package_json() {
  fixture("tests/fixtures/resolve/**/index.*", |file| {
    let dir = file.parent().unwrap();
    let main = dir.join("src").join("main.ts");

    let result = resolve::load_package_json(main.clone());
    assert!(result.is_ok());
    let result = result.unwrap();

    assert_eq!(result.name, "fixture-package-json".to_string());
    assert_eq!(result.version, "1.0.0".to_string());

    let sub = dir.join("sub");

    let result = resolve::load_package_json(sub);
    assert!(result.is_ok());
    let result = result.unwrap();

    assert_eq!(result.name, "sub-fixture".to_string());
    assert_eq!(result.version, "1.0.0".to_string());

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
      "fixture-package-json".to_string(),
    );
  })
}
