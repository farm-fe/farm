use std::collections::HashMap;

use farmfe_core::module::ModuleId;

use crate::resource_pot_to_bundle::{
  modules_analyzer::module_analyzer::{ExportSpecifierInfo, ExportType},
  uniq_name::BundleVariable,
};

#[derive(Debug, Clone, Default)]
pub struct ReferenceExport {
  pub named: HashMap<usize, usize>,
  pub default: Option<usize>,
  // TODO: `export * from "cjs"`; in cjs need transform to _export_star(cjs, module.exports)
  pub all: (bool, Option<usize>),
  pub namespace: Option<usize>,
}

impl ReferenceExport {
  pub fn new() -> Self {
    Self {
      named: HashMap::new(),
      default: None,
      all: (false, None),
      namespace: None,
    }
  }

  fn contains(&self, export: &ExportSpecifierInfo) -> bool {
    match export {
      ExportSpecifierInfo::Named(named) => self.named.contains_key(&named.local()),
      ExportSpecifierInfo::Default(_) => self.default.is_some(),
      ExportSpecifierInfo::All(_) => self.all.0,
      ExportSpecifierInfo::Namespace(_) => self.namespace.is_some(),
    }
  }

  fn insert(&mut self, export: &ExportSpecifierInfo) {
    match export {
      ExportSpecifierInfo::Named(named) => {
        self.named.insert(named.export_as(), named.local());
      }
      ExportSpecifierInfo::Default(local) => {
        self.default = Some(*local);
      }
      ExportSpecifierInfo::All(_) => {
        self.all = (true, None);
      }
      ExportSpecifierInfo::Namespace(name) => {
        self.namespace = Some(*name);
      }
    }
  }

  // reexport should ignore default
  // sub export should ignore all, because it's already to be named
  fn merge_by_export_all(&mut self, from: &ReferenceExport) {
    self.named.extend(from.named.iter());

    if let Some(ref ns) = from.namespace {
      self.named.insert(*ns, *ns);
    }
  }

  // external reexport should save all
  pub fn merge_by_external(&mut self, from: &ReferenceExport) {
    self.named.extend(from.named.iter());

    if let Some(ref ns) = from.namespace {
      self.named.insert(*ns, *ns);
    }

    if let Some(ref default) = from.default {
      self.default = Some(*default);
    }

    if from.all.0 {
      self.all = (true, None);
    }
  }

  pub fn is_empty(&self) -> bool {
    self.named.is_empty() && self.default.is_none() && !self.all.0 && self.namespace.is_none()
  }

  pub fn query(&self, export_from: &String, bundle_variable: &BundleVariable) -> Option<usize> {
    for export_as in self.named.keys() {
      if &bundle_variable.name(*export_as) == export_from {
        return Some(self.named[export_as]);
      }
    }

    if let Some(index) = self.namespace {
      if &bundle_variable.name(index) == export_from {
        return Some(index);
      }
    }

    return None;
  }
}

#[derive(Debug, Clone, Default)]
pub struct ReferenceMap {
  pub reference_map: HashMap<ModuleId, ReferenceExport>,
  pub export: ReferenceExport,
  pub export_type: ExportType,
}

impl ReferenceMap {
  pub fn new() -> Self {
    Self {
      export: ReferenceExport::default(),
      reference_map: Default::default(),
      export_type: ExportType::Static,
    }
  }

  pub fn add_reference(&mut self, module_id: &ModuleId, export: &ExportSpecifierInfo) {
    if !self.reference_map.contains_key(module_id) {
      self
        .reference_map
        .insert(module_id.clone(), ReferenceExport::new());
    }

    let reference = self.reference_map.get_mut(module_id).unwrap();
    reference.insert(export);
  }

  pub fn add_commonjs(&mut self, module_id: &ModuleId, export: &ExportSpecifierInfo) {
    self.add_reference(module_id, export);
    self.export_type.merge(ExportType::HybridDynamic);
  }

  pub fn add_local(&mut self, export: &ExportSpecifierInfo) {
    self.export.insert(export);
  }

  pub fn query_by_var_str(
    &self,
    export_from: &String,
    bundle_variable: &BundleVariable,
  ) -> Option<usize> {
    if let Some(r) = self.export.query(&export_from, bundle_variable) {
      Some(r)
    } else {
      self
        .reference_map
        .values()
        .find_map(|item| item.query(&export_from, bundle_variable))
    }
  }

  pub fn query(&self, index: usize, bundle_variable: &BundleVariable) -> Option<usize> {
    let export_from = bundle_variable.name(index);

    if let Some(r) = self.export.query(&export_from, bundle_variable) {
      Some(r)
    } else {
      self
        .reference_map
        .values()
        .find_map(|item| item.query(&export_from, bundle_variable))
    }
  }

  pub fn extends(&mut self, other: &ReferenceMap) {
    for item in &other.reference_map {
      if !self.reference_map.contains_key(item.0) {
        self
          .reference_map
          .insert(item.0.clone(), ReferenceExport::new());
      }

      if let Some(map) = self.reference_map.get_mut(item.0) {
        map.merge_by_export_all(item.1);
      }
    }

    self.export_type.merge(other.export_type);

    self.export.merge_by_export_all(&other.export);
  }
}
