use farmfe_core::swc_ecma_ast::Ident;
use farmfe_toolkit::swc_ecma_visit::Visit;

pub struct UsedIdentsCollector {
  pub used_idents: Vec<Ident>,
}

impl UsedIdentsCollector {
  pub fn new() -> Self {
    Self {
      used_idents: vec![],
    }
  }
}

impl Visit for UsedIdentsCollector {
  fn visit_ident(&mut self, ident: &Ident) {
    self.used_idents.push(ident.clone());
  }
}
