use std::{cell::RefCell, rc::Rc};

use farmfe_core::{
  config::{Config, ModuleFormat},
  error::{CompilationError, Result},
  module::{ModuleId, ModuleSystem, ModuleType},
  resource::resource_pot::ResourcePotId,
  HashMap,
};

use crate::resource_pot_to_bundle::{
  common::with_bundle_reference_slot_name,
  modules_analyzer::module_analyzer::{ExportSpecifierInfo, ImportSpecifierInfo, ModuleAnalyzer},
  uniq_name::BundleVariable,
  ShareBundleContext,
};

use super::ModuleAnalyzerManager;

#[derive(Debug, Clone)]
pub struct ExternalReferenceImport {
  pub named: HashMap<String, usize>,
  pub namespace: Option<usize>,
  pub default: Option<usize>,
  pub source: Option<ReferenceKind>,
}

impl ExternalReferenceImport {
  pub fn new() -> Self {
    Self {
      named: HashMap::default(),
      namespace: None,
      default: None,
      source: None,
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
        // TODO: use index
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

  pub fn extend(&mut self, other: &Self) {
    self
      .named
      .extend(other.named.iter().map(|(k, v)| (k.clone(), *v)));

    if self.namespace.is_none() {
      self.namespace = other.namespace;
    }

    if self.default.is_none() {
      self.default = other.default;
    }
  }
}

#[derive(Debug, Clone)]
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
      named: HashMap::default(),
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

  pub fn extend(&mut self, other: &Self) {
    self.named.extend(other.named.clone());

    if self.default.is_none() {
      self.default = other.default;
    }

    self.all.0 = self.all.0 || other.all.0;

    if self.namespace.is_none() {
      self.namespace = other.namespace;
    }
  }
}

// TODO: remove it
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub enum ReferenceKind {
  Bundle(String),
  Module(ModuleId),
}

impl ReferenceKind {
  pub fn is_module_id(&self) -> bool {
    matches!(self, ReferenceKind::Module(_))
  }

  /// remove it
  pub fn to_module_id(&self) -> ModuleId {
    match self {
      ReferenceKind::Bundle(name) => ModuleId::from(with_bundle_reference_slot_name(name, true)),
      ReferenceKind::Module(id) => id.clone(),
    }
  }
}

impl ToString for ReferenceKind {
  fn to_string(&self) -> String {
    match self {
      ReferenceKind::Bundle(m) => m.to_string(),
      ReferenceKind::Module(m) => m.to_string(),
    }
  }
}

impl From<ModuleId> for ReferenceKind {
  fn from(id: ModuleId) -> Self {
    ReferenceKind::Module(id)
  }
}

impl From<&ModuleId> for ReferenceKind {
  fn from(id: &ModuleId) -> Self {
    ReferenceKind::Module(id.clone())
  }
}

impl From<String> for ReferenceKind {
  fn from(name: String) -> Self {
    ReferenceKind::Bundle(name)
  }
}
impl From<&String> for ReferenceKind {
  fn from(name: &String) -> Self {
    ReferenceKind::Bundle(name.clone())
  }
}

///
/// {
///   import: {
///     source: xxx
///   },
///   commonjs: {
///     source: xxx
///   },
///   external: {
///     source: xxx
///   },
///   local_export: {
///     aa: bb
///   }
/// }
///

pub type CommonJsImportMap = farmfe_core::HashMap<ReferenceKind, ExternalReferenceImport>;

#[derive(Debug, Default)]
pub struct BundleReference {
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
}

impl CombineBundleReference {
  /// import "./cjs"
  pub fn execute_module_for_cjs(&mut self, import_kind: ReferenceKind) {
    self
      .redeclare_commonjs_import
      .entry(import_kind)
      .or_insert_with(ExternalReferenceImport::new);
  }

  pub fn add_execute_module(&mut self, import_kind: ReferenceKind) {
    self
      .import_map
      .entry(import_kind)
      .or_insert_with(ExternalReferenceImport::new);
  }

  pub fn add_local_export(
    &mut self,
    specify: &ExportSpecifierInfo,
    module_system: ModuleSystem,
    is_entry: bool,
  ) {
    let map = if is_entry {
      &mut self.reexport_raw.export
    } else {
      &mut self.bundle_reference1.export
    };
    if map.is_none() {
      *map = Some(ExternalReferenceExport::new(module_system));
    }

    if let Some(ref mut export) = map {
      export.insert(specify.clone())
    };
  }

  pub fn add_reference_export(
    &mut self,
    specify: &ExportSpecifierInfo,
    source: ReferenceKind,
    module_system: ModuleSystem,
    is_entry: bool,
  ) {
    let external_export_map = if is_entry {
      &mut self.reexport_raw.external_export_map
    } else {
      &mut self.bundle_reference1.external_export_map
    };
    if external_export_map.contains_key(&source) {
      let map = external_export_map.get_mut(&source).unwrap();
      map.insert(specify.clone());
    } else {
      let mut map = ExternalReferenceExport::new(module_system);
      map.insert(specify.clone());
      external_export_map.insert(source, map);
    }
  }

  pub fn change_to_hybrid_dynamic(&mut self, source: ReferenceKind) {
    if let Some(map) = self.bundle_reference1.external_export_map.get_mut(&source) {
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

  pub fn reexport_commonjs(
    &mut self,
    module_id: &ModuleId,
    module_analyzer_manager: &ModuleAnalyzerManager,
    bundle_variable: &BundleVariable,
  ) -> Result<()> {
    self.change_to_hybrid_dynamic(module_id.clone().into());

    self.add_declare_commonjs_import(
      &ImportSpecifierInfo::Namespace(
        module_analyzer_manager
          .module_global_uniq_name
          .namespace_name(module_id)
          .unwrap(),
      ),
      module_id.clone().into(),
      bundle_variable,
    )?;

    self.add_reference_export(
      &ExportSpecifierInfo::All(None),
      module_id.clone().into(),
      ModuleSystem::CommonJs,
      false,
    );

    Result::<()>::Ok(())
  }

  pub fn add_reexport_all(&mut self, reference_builder: ReferenceBuilder) -> Result<()> {
    if !reference_builder.is_reference_by_another_bundle {
      return Ok(());
    }

    let reexport_commonjs =
      |module_id: &ModuleId, bundle_reference: &mut CombineBundleReference| {
        bundle_reference.change_to_hybrid_dynamic(module_id.clone().into());

        bundle_reference.add_declare_commonjs_import(
          &ImportSpecifierInfo::Namespace(
            reference_builder
              .module_analyzer_manager
              .module_global_uniq_name
              .namespace_name(module_id)
              .unwrap(),
          ),
          module_id.clone().into(),
          reference_builder.bundle_variable,
        )?;

        bundle_reference.add_reference_export(
          &ExportSpecifierInfo::All(None),
          module_id.clone().into(),
          ModuleSystem::CommonJs,
          reference_builder.is_entry,
        );

        Result::<()>::Ok(())
      };
    let is_external = reference_builder.is_external(reference_builder.source);

    let is_commonjs = reference_builder
      .module_analyzer_manager
      .is_commonjs(reference_builder.source);
    let is_format_to_cjs = matches!(
      reference_builder.config.options.format,
      ModuleFormat::CommonJs
    );

    if is_external {
      // export * from "node:fs"
      // => commonjs
      // const node_fs = require("node:fs")
      // _export_star(node_fs, module.exports);

      if is_format_to_cjs {
        if let Some(ns) = reference_builder
          .module_analyzer_manager
          .module_global_uniq_name
          .namespace_name(reference_builder.source)
        {
          self.add_import(
            &ImportSpecifierInfo::Namespace(ns),
            reference_builder.source.clone().into(),
            &reference_builder.bundle_variable,
          )?;
        }
      }

      self.add_reference_export(
        &ExportSpecifierInfo::All(None),
        reference_builder.source.clone().into(),
        reference_builder.module_system.clone(),
        reference_builder.is_entry,
      );
    }
    // will be format commonjs, esm cannot reexport fields, should export as many fields as possible
    else if is_commonjs && is_format_to_cjs {
      reexport_commonjs(reference_builder.source, self)?;
    } else {
      let export_names = &*reference_builder
        .module_analyzer_manager
        .get_export_names(reference_builder.source);

      let export_type = export_names
        .export_type
        .merge(reference_builder.module_system.clone());

      let is_hybrid_dynamic = matches!(export_type, ModuleSystem::Hybrid);

      // local export
      {
        // export named
        for (export_as, local) in &export_names.export.named {
          let is_default_key = reference_builder.bundle_variable.is_default_key(*export_as);
          // reference_builder.bundle_variable.name(from)

          self.add_local_export(
            &if is_default_key {
              ExportSpecifierInfo::Default((*local).into())
            } else {
              ExportSpecifierInfo::Named((*local, Some(*export_as)).into())
            },
            export_type.clone(),
            reference_builder.is_entry,
          );

          if is_commonjs {
            let imported = if is_default_key {
              reference_builder
                .module_analyzer_manager
                .module_global_uniq_name
                .default_name_result(reference_builder.module_id.to_string())?
            } else {
              *export_as
            };

            self.add_declare_commonjs_import(
              &ImportSpecifierInfo::Named {
                local: *local,
                imported: Some(imported),
              },
              reference_builder.source.clone().into(),
              &reference_builder.bundle_variable,
            )?;
          }
        }

        // export default
        if let Some(item) = &export_names.export.default {
          let is_default_key = reference_builder.bundle_variable.is_default_key(*item);

          self.add_local_export(
            &ExportSpecifierInfo::Default(if is_default_key {
              reference_builder
                .module_analyzer_manager
                .module_global_uniq_name
                .default_name_result(reference_builder.source)?
            } else {
              *item
            }),
            export_type.clone(),
            reference_builder.is_entry,
          );

          if is_commonjs {
            self.add_declare_commonjs_import(
              &ImportSpecifierInfo::Default(if is_default_key {
                reference_builder
                  .module_analyzer_manager
                  .module_global_uniq_name
                  .default_name_result(reference_builder.source)?
              } else {
                *item
              }),
              reference_builder.source.clone().into(),
              &reference_builder.bundle_variable,
            )?;
          }
        }
      }

      // reexport external | bundle
      {
        for (module_id, reference) in &export_names.reexport_map {
          let is_external_source = reference_builder.is_external(module_id);
          let is_commonjs_source = reference_builder.is_commonjs(module_id);

          if is_external_source {
            // export named
            for (from, export_as) in &reference.named {
              self.add_reference_export(
                &ExportSpecifierInfo::Named((*export_as, Some(*from)).into()),
                module_id.clone().into(),
                export_type.clone(),
                reference_builder.is_entry,
              );
            }

            // export default
            if let Some(item) = &reference.default {
              self.add_reference_export(
                &ExportSpecifierInfo::Default(*item),
                module_id.clone().into(),
                export_type.clone(),
                reference_builder.is_entry,
              );
            }

            // reexport all
            if reference.all {
              if is_hybrid_dynamic && is_format_to_cjs {
                self.add_import(
                  &ImportSpecifierInfo::Namespace(
                    reference_builder
                      .module_analyzer_manager
                      .module_global_uniq_name
                      .namespace_name(module_id)
                      .unwrap(),
                  ),
                  module_id.clone().into(),
                  &reference_builder.bundle_variable,
                )?;
              }

              self.add_reference_export(
                &ExportSpecifierInfo::All(None),
                module_id.clone().into(),
                export_type.clone(),
                reference_builder.is_entry,
              );
            }
          } else if is_commonjs_source {
            reexport_commonjs(module_id, self)?
          }
        }
      }
    }

    Ok(())
  }

  // TODO: refactor module relation analyze
  pub fn add_export_named(&mut self) {}

  pub fn add_reexport_named(&mut self) {}

  pub fn add_export_namespace(&mut self) {}

  pub fn add_reexport_namespace(&mut self) {}

  pub fn add_export_default(&mut self) {}

  pub fn add_reexport_default(&mut self) {}

  pub fn add_import_default(&mut self) {}

  pub fn add_import_named(&mut self) {}

  pub fn add_import_namespace(&mut self) {}
}

#[derive(Debug, Default)]
pub struct BundleReferenceManager {
  /// why use [ResourcePotId] as key?
  ///
  /// because use [ResourcePotId] as key can zip same export or import at Bundle
  ///
  /// but there is one place to be careful that entry module/bundle, it should export raw named
  bundle_reference: HashMap<ResourcePotId, Rc<RefCell<CombineBundleReference>>>,
  // bundle_reference1: HashMap<ModuleId, Rc<RefCell<BundleReference>>>,
}

#[derive(Debug, Default)]
pub struct CombineBundleReference {
  pub reexport_raw: BundleReference,
  pub bundle_reference1: BundleReference,

  /// import { xxx } from './external_bundle_module' | './other_bundle_module'
  pub import_map: HashMap<ReferenceKind, ExternalReferenceImport>,

  ///
  /// ```ts
  /// export { } from "./cjs_module";
  /// export * as ns from "./cjs_module";
  /// export { default } ns from "./cjs_module";
  /// // =>
  /// const cjs_module_cjs = cjs_module()["default"];
  ///
  /// {
  ///   "cjs_module": {
  ///     default: None,
  ///     named: {}
  ///   }
  /// }
  /// ```
  ///
  pub redeclare_commonjs_import: CommonJsImportMap,
}

impl CombineBundleReference {}

// TODO: improve logic
impl CombineBundleReference {
  pub fn new() -> Self {
    Self {
      reexport_raw: BundleReference::new(),
      bundle_reference1: BundleReference::new(),
      import_map: HashMap::default(),
      redeclare_commonjs_import: HashMap::default(),
    }
  }
  pub fn fetch(
    &self,
    module_id: &ModuleId,
    module_analyzer_manager: &ModuleAnalyzerManager,
  ) -> &BundleReference {
    if module_analyzer_manager.is_entry(module_id) {
      &self.reexport_raw
    } else {
      &self.bundle_reference1
    }
  }

  pub fn fetch_mut(
    &mut self,
    module_id: &ModuleId,
    module_analyzer_manager: &ModuleAnalyzerManager,
  ) -> &mut BundleReference {
    if module_analyzer_manager.is_entry(module_id) {
      &mut self.reexport_raw
    } else {
      &mut self.bundle_reference1
    }
  }
}

impl BundleReferenceManager {
  // import compress
  pub fn reference_mut(&mut self, group_id: &ResourcePotId) -> Rc<RefCell<CombineBundleReference>> {
    Rc::clone(if self.bundle_reference.contains_key(group_id) {
      self.bundle_reference.get(group_id).unwrap()
    } else {
      self
        .bundle_reference
        .entry(group_id.clone())
        .or_insert_with(|| Rc::new(RefCell::new(CombineBundleReference::default())))
    })
  }

  pub fn reference_mut_by_module(
    &mut self,
    module_id: &ModuleId,
    module_analyzer_manager: &ModuleAnalyzerManager,
  ) -> Rc<RefCell<CombineBundleReference>> {
    let group_id = module_analyzer_manager.group_id(module_id).unwrap();
    self.reference_mut(group_id)
  }

  // pub fn reference1_mut(&mut self, module_id: &ModuleId) -> Rc<RefCell<BundleReference>> {
  //   Rc::clone(if self.bundle_reference1.contains_key(module_id) {
  //     self.bundle_reference1.get(module_id).unwrap()
  //   } else {
  //     self
  //       .bundle_reference1
  //       .entry(module_id.clone())
  //       .or_insert_with(|| Rc::new(RefCell::new(BundleReference::new())))
  //   })
  // }
}

pub struct ReferenceBuilder<'a> {
  pub is_reference_by_another_bundle: bool,
  pub module_analyzer_manager: &'a ModuleAnalyzerManager<'a>,
  pub module_analyzer: &'a ModuleAnalyzer,
  pub bundle_variable: &'a mut BundleVariable,
  pub source: &'a ModuleId,
  pub module_system: ModuleSystem,
  pub config: &'a ShareBundleContext,
  pub module_id: &'a ModuleId,
  pub is_entry: bool,
}

impl<'a> ReferenceBuilder<'a> {
  fn is_external(&self, module_id: &ModuleId) -> bool {
    self.module_analyzer_manager.is_external(module_id)
      || !self.module_analyzer_manager.contain(module_id)
  }

  fn is_commonjs(&self, module_id: &ModuleId) -> bool {
    self.module_analyzer_manager.is_commonjs(module_id)
  }

  ///
  /// ```ts
  /// // moduleA.ts
  /// export default 'a';
  /// ```
  ///
  /// ```ts
  /// // moduleB.ts
  /// export default 'b';
  /// ```
  /// ---
  ///
  /// ```ts
  ///
  /// // entry bundle (entry module moduleA.ts)
  /// export default module_a_default;
  ///
  /// // normal_bundle
  /// export { module_a_default, module_b_default }
  ///
  /// // import bundle
  /// import { module_a_default, module_b_default } from './normal_bundle'
  /// ```
  pub fn reexport_name(&self, raw_export: usize, export_as: usize) -> usize {
    if self.module_analyzer.entry {
      raw_export
    } else {
      export_as
    }
  }
}
