use std::collections::HashMap;

use farmfe_core::{
  error::{CompilationError, Result},
  module::{ModuleId, ModuleSystem},
};

use super::{
  modules_analyzer::module_analyzer::{ExportSpecifierInfo, ExportType, ImportSpecifierInfo},
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
          self.named.insert(name, local);
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

#[derive(Debug, Default)]
pub struct ExternalReferenceExport {
  pub named: HashMap<usize, usize>,
  pub default: Option<usize>,
  pub all: bool,
  pub namespace: Option<usize>,
  pub export_type: ExportType,
}

impl ExternalReferenceExport {
  pub fn new() -> Self {
    Self {
      named: HashMap::new(),
      default: None,
      all: false,
      namespace: None,
      export_type: Default::default(),
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub enum ReferenceKind {
  Bundle(String),
  Module(ModuleId),
}

impl ReferenceKind {
  pub fn to_module_id(&self) -> ModuleId {
    match self {
      ReferenceKind::Bundle(name) => ModuleId::from(name.as_str()),
      ReferenceKind::Module(id) => id.clone(),
    }
  }
}

impl ToString for ReferenceKind {
  fn to_string(&self) -> String {
    match self {
      ReferenceKind::Bundle(name) => name.clone(),
      ReferenceKind::Module(id) => id.to_string(),
    }
  }
}

impl From<ModuleId> for ReferenceKind {
  fn from(id: ModuleId) -> Self {
    ReferenceKind::Module(id)
  }
}

impl From<String> for ReferenceKind {
  fn from(name: String) -> Self {
    ReferenceKind::Bundle(name)
  }
}

#[derive(Debug)]
pub struct BundleReference {
  /// import { xxx } from './external_bundle_module' | './other_bundle_module'
  pub import_map: HashMap<ReferenceKind, ExternalReferenceImport>,

  pub commonjs_import_map: HashMap<ReferenceKind, ExternalReferenceImport>,

  pub commonjs_export_map: HashMap<ReferenceKind, ExternalReferenceExport>,

  /// export xxx from './external_bundle_module'
  pub external_export_map: HashMap<ReferenceKind, ExternalReferenceExport>,

  /// export local
  pub export: Option<ExternalReferenceExport>,
}

impl BundleReference {
  pub fn new() -> Self {
    Self {
      commonjs_import_map: HashMap::new(),
      commonjs_export_map: HashMap::new(),
      import_map: HashMap::new(),
      external_export_map: HashMap::new(),
      export: None,
    }
  }

  pub fn sync_export(
    &mut self,
    export: &ExportSpecifierInfo,
    source: Option<ReferenceKind>,
    to_export_map: Option<&mut HashMap<ReferenceKind, ExternalReferenceExport>>,
  ) {
    if let Some(module_id) = source {
      let map = to_export_map.unwrap_or(&mut self.external_export_map);
      if !map.contains_key(&module_id) {
        map.insert(module_id.clone(), ExternalReferenceExport::new());
      }

      let module_export_map = map.get_mut(&module_id).unwrap();

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

  pub fn sync_import(
    &mut self,
    import_kind: ReferenceKind,
    import: &ImportSpecifierInfo,
    bundle_variable: &BundleVariable,
    is_cjs: bool,
  ) -> Result<usize> {
    let import_map = if is_cjs {
      &mut self.commonjs_import_map
    } else {
      &mut self.import_map
    };
    if !import_map.contains_key(&import_kind) {
      import_map.insert(import_kind.clone(), ExternalReferenceImport::new());
    }

    let module_import_map = import_map.get_mut(&import_kind).unwrap();

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

  pub fn import(&self, import_kind: &ReferenceKind) -> Option<&ExternalReferenceImport> {
    self.import_map.get(import_kind)
  }
}
