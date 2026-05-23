use super::ImportPreset;

pub fn get_react_router_preset() -> ImportPreset {
  ImportPreset {
    from: "react-router".to_string(),
    imports: vec![
      "useOutletContext".to_string(),
      "useHref".to_string(),
      "useInRouterContext".to_string(),
      "useLocation".to_string(),
      "useNavigationType".to_string(),
      "useNavigate".to_string(),
      "useOutlet".to_string(),
      "useParams".to_string(),
      "useResolvedPath".to_string(),
      "useRoutes".to_string(),
    ],
  }
}
