use farmfe_core::{hashbrown::HashSet, swc_ecma_ast::Ident};
use farmfe_toolkit::swc_ecma_visit::Visit;

pub struct UsedIdentsCollector {
  pub used_idents: HashSet<String>,
}

impl UsedIdentsCollector {
  pub fn new() -> Self {
    Self {
      used_idents: HashSet::new(),
    }
  }
}

impl Visit for UsedIdentsCollector {
  fn visit_ident(&mut self, ident: &Ident) {
    self.used_idents.insert(ident.to_string());
  }
}
