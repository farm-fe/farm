pub use farmfe_core::module::meta_data::script::{EXPORT_DEFAULT, EXPORT_NAMESPACE};
use farmfe_core::{
  module::{meta_data::script::statement::SwcId, module_graph::ModuleGraph, ModuleId},
  swc_ecma_ast::{ExportSpecifier, Ident},
  HashMap,
};
use swc_ecma_visit::{VisitMut, VisitMutWith};

struct TopLevelIdents {
  idents: HashMap<String, usize>,
}

impl TopLevelIdents {
  pub fn new() -> Self {
    Self {
      idents: HashMap::default(),
    }
  }

  pub fn extend(&mut self, iter: impl Iterator<Item = String>) {
    for ident in iter {
      self.add_ident(ident);
    }
  }

  pub fn add_ident(&mut self, ident: String) {
    let count = self.idents.entry(ident).or_insert(0);
    *count += 1;
  }

  pub fn get_unique_ident(&self, ident: &str) -> String {
    if let Some(count) = self.idents.get(ident) {
      if *count > 1 {
        return format!("{ident}${}", *count - 1);
      } else {
        return ident.to_string();
      }
    }

    unreachable!("add_ident({ident}) should be called before get_unique_ident")
  }
}

pub struct TopLevelIdentsRenameHandler {
  module_rename_map: HashMap<ModuleId, HashMap<SwcId, SwcId>>,
  top_level_idents: TopLevelIdents,
}

impl TopLevelIdentsRenameHandler {
  fn new(top_level_idents: TopLevelIdents) -> Self {
    Self {
      module_rename_map: HashMap::default(),
      top_level_idents,
    }
  }

  pub fn rename_ident(&mut self, module_id: ModuleId, from: SwcId, to: SwcId) {
    self
      .module_rename_map
      .entry(module_id)
      .or_default()
      .insert(from, to);
  }

  pub fn get_renamed_ident(&self, module_id: &ModuleId, ident: &SwcId) -> Option<SwcId> {
    self
      .module_rename_map
      .get(module_id)
      .and_then(|map| map.get(ident))
      .cloned()
  }

  /// rename the imported ident if there are conflicts
  pub fn rename_ident_if_conflict(&mut self, module_id: &ModuleId, ident: &SwcId) {
    self.top_level_idents.add_ident(ident.sym.to_string());
    let unique_ident = self.top_level_idents.get_unique_ident(ident.sym.as_str());

    if unique_ident != *ident.sym {
      let mut cloned = ident.clone();
      cloned.sym = unique_ident.into();
      self.rename_ident(module_id.clone(), ident.clone(), cloned);
    }
  }
}

pub struct RenameVisitor<'a> {
  module_id: &'a ModuleId,
  rename_handler: &'a TopLevelIdentsRenameHandler,
}

impl<'a> RenameVisitor<'a> {
  pub fn new(module_id: &'a ModuleId, rename_handler: &'a TopLevelIdentsRenameHandler) -> Self {
    Self {
      module_id,
      rename_handler,
    }
  }
}

impl VisitMut for RenameVisitor<'_> {
  fn visit_mut_export_specifier(&mut self, sp: &mut ExportSpecifier) {
    if let ExportSpecifier::Named(named) = sp {
      // do not rename exported ident
      named.orig.visit_mut_children_with(self);
    } else {
      sp.visit_mut_children_with(self);
    }
  }

  fn visit_mut_ident(&mut self, n: &mut Ident) {
    if let Some(renamed_ident) = self
      .rename_handler
      .get_renamed_ident(self.module_id, &n.to_id().into())
    {
      n.ctxt = renamed_ident.ctxt();
      n.sym = renamed_ident.sym;
    }
  }
}

pub fn init_rename_handler(
  sorted_modules: &[ModuleId],
  module_graph: &ModuleGraph,
) -> TopLevelIdentsRenameHandler {
  let mut top_level_idents = TopLevelIdents::new();

  // init top level idents
  sorted_modules.iter().for_each(|module_id| {
    let module = module_graph.module(module_id).unwrap();
    let script_meta = module.meta.as_script();

    top_level_idents.extend(
      script_meta
        .unresolved_idents
        .iter()
        .map(|id| id.sym.to_string()),
    );
  });

  let mut rename_handler = TopLevelIdentsRenameHandler::new(top_level_idents);

  // rename top level idents that are not defined in module decl statement
  sorted_modules.iter().for_each(|module_id| {
    let module = module_graph.module(module_id).unwrap();
    let script_meta = module.meta.as_script();

    for statement in &script_meta.statements {
      if statement.import_info.is_some()
        || statement.export_info.is_some()
        || statement.defined_idents.is_empty()
      {
        continue;
      }

      statement.defined_idents.iter().for_each(|ident| {
        rename_handler.rename_ident_if_conflict(module_id, ident);
      });
    }
  });

  rename_handler
}
