pub mod module_graph;

pub enum ModuleType {
  // native supported module type by the core plugins
  Js,
  Jsx,
  Ts,
  Tsx,
  Css,
  Scss,
  Less,
  // custom module type from using by custom plugins
  Custom(String),
}

pub struct Module {
  pub id: String,
  pub module_type: ModuleType,
}
