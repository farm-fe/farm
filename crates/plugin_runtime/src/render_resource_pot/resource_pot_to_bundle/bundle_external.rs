use std::collections::HashMap;

use farmfe_core::{
  error::{CompilationError, Result},
  module::ModuleId,
};

use super::{
  modules_analyzer::module_analyzer::{ExportSpecifierInfo, ImportSpecifierInfo},
  uniq_name::BundleVariable,
};

#[derive(Debug)]
pub struct ExternalReferenceImport {
  pub named: HashMap<String, usize>,
  pub namespace: Option<usize>,
  pub default: Option<usize>,
}

impl ExternalReferenceImport {
  fn new() -> Self {
    Self {
      named: HashMap::new(),
      namespace: None,
      default: None,
    }
  }

  fn fetch(
    &self,
    import_type: &ImportSpecifierInfo,
    bundle_variable: &BundleVariable,
  ) -> Option<usize> {
    match import_type {
      ImportSpecifierInfo::Named { local, imported } => self
        .named
        .get(&bundle_variable.name(imported.unwrap_or(*local)))
        .cloned(),
      ImportSpecifierInfo::Namespace(_) => self.namespace.clone(),
      ImportSpecifierInfo::Default(_) => self.default.clone(),
    }
  }

  fn insert(&mut self, import_type: ImportSpecifierInfo, bundle_variable: &BundleVariable) {
    match import_type {
      ImportSpecifierInfo::Named { local, imported } => {
        let imported = imported.unwrap_or_else(|| local.clone());
        let name = bundle_variable.name(imported);

        if !self.named.contains_key(&name) {
          self.named.insert(bundle_variable.name(imported), local);
        }
      }
      ImportSpecifierInfo::Namespace(name) => {
        self.namespace = Some(name);
      }
      ImportSpecifierInfo::Default(name) => {
        self.default = Some(name);
      }
    }
  }
}

#[derive(Debug)]
pub struct ExternalReferenceExport {
  pub named: HashMap<usize, usize>,
  pub default: Option<usize>,
  pub all: bool,
  pub namespace: Option<usize>,
}

impl ExternalReferenceExport {
  fn new() -> Self {
    Self {
      named: HashMap::new(),
      default: None,
      all: false,
      namespace: None,
    }
  }

  fn contains(&self, export: &ExportSpecifierInfo) -> bool {
    match export {
      ExportSpecifierInfo::Named(named) => self.named.contains_key(&named.local()),
      ExportSpecifierInfo::Default(_) => self.default.is_some(),
      ExportSpecifierInfo::All(_) => self.all,
      ExportSpecifierInfo::Namespace(_) => self.namespace.is_some(),
    }
  }

  fn insert(&mut self, export: ExportSpecifierInfo) {
    match export {
      ExportSpecifierInfo::Named(named) => {
        self.named.insert(named.export_as(), named.local());
      }
      ExportSpecifierInfo::Default(local) => {
        self.default = Some(local);
      }
      ExportSpecifierInfo::All(_) => {
        self.all = true;
      }
      ExportSpecifierInfo::Namespace(name) => {
        self.namespace = Some(name);
      }
    }
  }
}

#[derive(Debug)]
pub struct BundleReference {
  /// import { xxx } from './external_bundle_module' | './other_bundle_module'
  pub import_map: HashMap<String, ExternalReferenceImport>,
  /// export xxx from './external_bundle_module'
  pub external_export_map: HashMap<ModuleId, ExternalReferenceExport>,
  /// export local
  pub export: Option<ExternalReferenceExport>,
}

impl BundleReference {
  pub fn new() -> Self {
    Self {
      import_map: HashMap::new(),
      external_export_map: HashMap::new(),
      export: None,
    }
  }

  pub fn sync_export(&mut self, export: &ExportSpecifierInfo, source: &Option<ModuleId>) {
    if let Some(module_id) = source {
      if !self.external_export_map.contains_key(&module_id) {
        self
          .external_export_map
          .insert(module_id.clone(), ExternalReferenceExport::new());
      }

      let module_export_map = self.external_export_map.get_mut(&module_id).unwrap();

      if !module_export_map.contains(export) {
        module_export_map.insert(export.clone());
      }
    } else {
      if self.export.is_none() {
        self.export = Some(ExternalReferenceExport::new());
      }

      if let Some(self_export) = self.export.as_mut() {
        if !self_export.contains(export) {
          self_export.insert(export.clone());
        }
      }
    }
  }

  pub fn sync_import<M: ToString>(
    &mut self,
    module_id: &M,
    import: &ImportSpecifierInfo,
    bundle_variable: &BundleVariable,
  ) -> Result<usize> {
    let module_id = module_id.to_string();
    if !self.import_map.contains_key(&module_id) {
      self
        .import_map
        .insert(module_id.clone(), ExternalReferenceImport::new());
    }

    let module_import_map = self.import_map.get_mut(&module_id).unwrap();

    if let Some(options) = module_import_map.fetch(import, bundle_variable) {
      Ok(options)
    } else {
      module_import_map.insert(import.clone(), bundle_variable);
      module_import_map
        .fetch(import, bundle_variable)
        .map(Ok)
        .unwrap_or(Err(CompilationError::GenericError(
          "failed fetch import".to_string(),
        )))
    }
  }

  pub fn import<M: ToString>(&self, module_id: &M) -> Option<&ExternalReferenceImport> {
    self.import_map.get(&module_id.to_string())
  }
}
