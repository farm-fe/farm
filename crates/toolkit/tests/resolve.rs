use farmfe_core::common::SideEffects;
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
      SideEffects::Bool(false)
    ));

    let sub = dir.join("sub");

    let result = resolve::load_package_json(sub.clone(), Default::default());
    assert!(result.is_ok());
    let result = result.unwrap();

    assert_eq!(result.name, Some("sub-fixture".to_string()));
    assert_eq!(result.version, Some("1.0.0".to_string()));

    assert!(matches!(
      result.side_effects().unwrap(),
      SideEffects::Array(_)
    ));

    if let SideEffects::Array(arr) = result.side_effects().unwrap() {
      assert!(arr.len() == 1);
      assert!(arr[0].is_match("main.css"));
    }

    // make sure cache works
    let cache = PACKAGE_JSON_LOADER.cache();
    assert!(cache.contains_key(&PACKAGE_JSON_LOADER.get_cache_key(&main, &Default::default())));
    assert!(
      cache.contains_key(&PACKAGE_JSON_LOADER.get_cache_key(&dir.join("src"), &Default::default()))
    );
    assert!(cache
      .contains_key(&PACKAGE_JSON_LOADER.get_cache_key(&dir.to_path_buf(), &Default::default())));

    let cached_result = cache
      .get(&PACKAGE_JSON_LOADER.get_cache_key(&dir.join("src"), &Default::default()))
      .unwrap();
    assert_eq!(
      cached_result.value().name,
      Some("fixture-package-json".to_string()),
    );
  })
}
