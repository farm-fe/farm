use super::ImportPreset;

pub fn get_vue_router_preset() -> ImportPreset {
  ImportPreset {
    from: "vue-router".to_string(),
    imports: vec![
      "useRouter".to_string(),
      "useRoute".to_string(),
      "useLink".to_string(),
      "onBeforeRouteLeave".to_string(),
      "onBeforeRouteUpdate".to_string(),
    ],
  }
}
