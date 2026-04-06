use super::ImportPreset;

pub fn get_react_router_dom_preset() -> ImportPreset {
  ImportPreset {
    imports: vec![
      "useLinkClickHandler".to_string(),
      "useSearchParams".to_string(),
      "Link".to_string(),
      "NavLink".to_string(),
      "Navigate".to_string(),
      "Outlet".to_string(),
      "Route".to_string(),
      "Routes".to_string(),
    ],
    from: "react-router-dom".to_string(),
  }
}
