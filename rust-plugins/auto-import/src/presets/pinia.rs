use super::ImportPreset;

pub fn get_pinia_preset() -> ImportPreset {
  ImportPreset {
    from: "pinia".to_string(),
    imports: vec![
      "acceptHMRUpdate".to_string(),
      "createPinia".to_string(),
      "defineStore".to_string(),
      "getActivePinia".to_string(),
      "mapActions".to_string(),
      "mapGetters".to_string(),
      "mapState".to_string(),
      "mapStores".to_string(),
      "mapWritableState".to_string(),
      "setActivePinia".to_string(),
      "setMapStoreSuffix".to_string(),
      "storeToRefs".to_string(),
    ]
  }
}
