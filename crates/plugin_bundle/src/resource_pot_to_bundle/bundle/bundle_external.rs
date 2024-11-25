use std::collections::HashMap;

use farmfe_core::{
  error::{CompilationError, Result},
  module::{ModuleId, ModuleSystem},
  resource::resource_pot::ResourcePotType,
};

use crate::resource_pot_to_bundle::{
  modules_analyzer::module_analyzer::{ExportSpecifierInfo, ImportSpecifierInfo},
  uniq_name::BundleVariable,
};

use super::ModuleAnalyzerManager;

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
      ImportSpecifierInfo::Namespace(_) => self.namespace,
      ImportSpecifierInfo::Default(_) => self.default,
    }
  }

  fn insert(&mut self, import_type: ImportSpecifierInfo, bundle_variable: &BundleVariable) {
    match import_type {
      ImportSpecifierInfo::Named { local, imported } => {
        let imported = imported.unwrap_or(local);
        let name = bundle_variable.name(imported);

        self.named.entry(name).or_insert(local);
      }
      ImportSpecifierInfo::Namespace(name) => {
        self.namespace = Some(name);
      }
      ImportSpecifierInfo::Default(name) => {
        self.default = Some(name);
      }
    }
  }

  pub fn is_empty(&self) -> bool {
    self.named.is_empty() && self.namespace.is_none() && self.default.is_none()
  }
}

#[derive(Debug)]
pub struct ExternalReferenceExport {
  pub named: HashMap<usize, usize>,
  pub default: Option<usize>,
  // TODO: `export * from "cjs"`; in cjs need transform to _export_star(cjs, module.exports)
  pub all: (bool, Option<usize>),
  pub namespace: Option<usize>,
  pub module_system: ModuleSystem,
}

impl ExternalReferenceExport {
  pub fn new(module_system: ModuleSystem) -> Self {
    Self {
      named: HashMap::new(),
      default: None,
      all: (false, None),
      namespace: None,
      module_system,
    }
  }

  #[allow(dead_code)]
  fn contains(&self, export: &ExportSpecifierInfo) -> bool {
    match export {
      ExportSpecifierInfo::Named(named) => self.named.contains_key(&named.local()),
      ExportSpecifierInfo::Default(_) => self.default.is_some(),
      ExportSpecifierInfo::All(_) => self.all.0,
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
        self.all = (true, None);
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

#[allow(clippy::to_string_trait_impl)]
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

#[derive(Debug, Default)]
pub struct BundleReference {
  /// import { xxx } from './external_bundle_module' | './other_bundle_module'
  pub import_map: HashMap<ReferenceKind, ExternalReferenceImport>,

  ///
  /// ```ts
  /// export { } from "./cjs_module";
  /// export * as ns from "./cjs_module";
  /// export { default } ns from "./cjs_module";
  /// // =>
  /// const cjs_module_cjs = cjs_module()["default"];
  /// ```
  ///
  pub redeclare_commonjs_import: HashMap<ReferenceKind, ExternalReferenceImport>,

  // pub declare_commonjs_export: HashMap<ReferenceKind, ExternalReferenceExport>,
  /// export xxx from './external_bundle_module'
  /// export * as ns from './external_bundle_module'
  pub external_export_map: HashMap<ReferenceKind, ExternalReferenceExport>,

  /// export { local }
  /// export default local
  pub export: Option<ExternalReferenceExport>,
}

impl BundleReference {
  pub fn new() -> Self {
    Self::default()
  }

  /// import "./cjs"
  pub fn execute_module_for_cjs(&mut self, import_kind: ReferenceKind) {
    self
      .redeclare_commonjs_import
      .entry(import_kind)
      .or_insert_with(ExternalReferenceImport::new);
  }

  pub fn add_local_export(&mut self, specify: &ExportSpecifierInfo, module_system: ModuleSystem) {
    if self.export.is_none() {
      self.export = Some(ExternalReferenceExport::new(module_system));
    }
    if let Some(ref mut export) = self.export {
      export.insert(specify.clone())
    };
  }

  pub fn add_reference_export(
    &mut self,
    specify: &ExportSpecifierInfo,
    source: ReferenceKind,
    module_system: ModuleSystem,
  ) {
    if self.external_export_map.contains_key(&source) {
      let map = self.external_export_map.get_mut(&source).unwrap();
      map.insert(specify.clone());
      return;
    }

    let mut map = ExternalReferenceExport::new(module_system);
    map.insert(specify.clone());
    self.external_export_map.insert(source, map);
  }

  pub fn change_to_hybrid_dynamic(&mut self, source: ReferenceKind) {
    if let Some(map) = self.external_export_map.get_mut(&source) {
      map.module_system.merge(ModuleSystem::Hybrid);
    }
  }

  fn add_import_helper(
    map: &mut HashMap<ReferenceKind, ExternalReferenceImport>,
    import: &ImportSpecifierInfo,
    source: ReferenceKind,
    bundle_variable: &BundleVariable,
  ) -> Result<usize> {
    if !map.contains_key(&source) {
      map.insert(source.clone(), ExternalReferenceImport::new());
    }

    let module_import_map = map.get_mut(&source).unwrap();

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

  pub fn add_declare_commonjs_import(
    &mut self,
    import: &ImportSpecifierInfo,
    source: ReferenceKind,
    bundle_variable: &BundleVariable,
  ) -> Result<usize> {
    Self::add_import_helper(
      &mut self.redeclare_commonjs_import,
      import,
      source,
      bundle_variable,
    )
  }

  pub fn add_empty_import(&mut self, source: ReferenceKind) {
    self
      .import_map
      .entry(source)
      .or_insert_with(ExternalReferenceImport::new);
  }

  pub fn add_import(
    &mut self,
    import: &ImportSpecifierInfo,
    source: ReferenceKind,
    bundle_variable: &BundleVariable,
  ) -> Result<usize> {
    Self::add_import_helper(&mut self.import_map, import, source, bundle_variable)
  }

  pub fn import(&self, import_kind: &ReferenceKind) -> Option<&ExternalReferenceImport> {
    self.import_map.get(import_kind)
  }
}

///
/// when export something from entry module, we need to reexport it to the bundle
/// ```ts
/// // entry module
/// export const name = "foo";
/// module.exports.age = 18;
/// ```
///
/// output
///
/// ```js
/// var index_cjs = __commonJs((module, exports)=>{
///   // ...
///   Object.defineProperty(exports, "name", {
///       enumerable: true,
///       get: function() {
///           return name;
///       }
///   });
///   const name = 'foo';
///   module.exports.age = 18;
/// });
///
/// // -------- esm --------
/// var index_ns = _interop_require_wildcard(index_cjs()), name = index_cjs()["name"];
/// export { name };
/// export default index_ns;
///
/// // -------- cjs --------
///
/// var index_ns = _interop_require_wildcard(index_cjs()), name = index_cjs()["name"];
/// module.exports.name = name;
/// Object.defineProperty(exports, "__esModule", {
///     value: true
/// });
/// _export_star(index_ns, module.exports);
/// ```
///
///
pub fn try_reexport_entry_module(
  resource_pot_type: ResourcePotType,
  bundle_reference: &mut BundleReference,
  bundle_variable: &BundleVariable,
  module_id: &ModuleId,
  module_analyzer_manager: &ModuleAnalyzerManager,
  is_format_to_commonjs: bool,
  module_system: ModuleSystem,
) -> Result<()> {
  let reference_kind = ReferenceKind::Module((*module_id).clone());

  if matches!(resource_pot_type, ResourcePotType::Runtime) {
    bundle_reference.execute_module_for_cjs(reference_kind);
    return Ok(());
  }

  // already export default in entry module
  if bundle_reference
    .export
    .as_ref()
    .is_some_and(|e| e.default.is_some())
  {
    return Ok(());
  };

  let Some(ns) = module_analyzer_manager
    .module_global_uniq_name
    .namespace_name(module_id)
  else {
    return Ok(());
  };

  bundle_reference.add_declare_commonjs_import(
    &ImportSpecifierInfo::Namespace(ns),
    reference_kind.clone(),
    bundle_variable,
  )?;

  if is_format_to_commonjs {
    bundle_reference.add_reference_export(
      &ExportSpecifierInfo::All(None),
      reference_kind.clone(),
      module_system,
    );
  } else {
    bundle_reference.add_local_export(&ExportSpecifierInfo::Default(ns), module_system);
  }

  Ok(())
}
